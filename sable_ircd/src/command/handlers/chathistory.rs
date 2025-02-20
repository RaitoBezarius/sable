use super::*;
use crate::utils;
use messages::send_history::SendHistoryItem;
use sable_network::network::update::HistoricMessageTarget;

use std::cmp::{
    max,
    min,
};

#[command_handler("CHATHISTORY")]
fn handle_chathistory(source: UserSource, server: &ClientServer, cmd: &dyn Command,
                      subcommand: &str, arg_1: &str, arg_2: &str, arg_3: &str, arg_4: Option<&str>) -> CommandResult
{
    let source = source.deref();

    match subcommand.to_ascii_uppercase().as_str()
    {
        "TARGETS" =>
        {
            let from_ts = utils::parse_timestamp(arg_1);
            let to_ts = utils::parse_timestamp(arg_2);
            let limit = arg_3.parse().ok();

            if from_ts.is_none() || to_ts.is_none()
            {
                cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid timestamp"));
                return Ok(());
            }
            if limit.is_none()
            {
                cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid limit"));
                return Ok(());
            }

            // The spec allows the from and to timestamps in either order; list_targets requires from < to
            list_targets(server, cmd, source, min(from_ts, to_ts), max(from_ts, to_ts), limit);
        }
        "LATEST" =>
        {
            let target = arg_1.clone();
            let from_ts = match arg_2
            {
                "*" => None,
                _ => match utils::parse_timestamp(arg_2)
                {
                    Some(ts) => Some(ts),
                    None => {
                        cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid timestamp"));
                        return Ok(());
                    }
                }
            };

            let limit = arg_3.parse().ok();
            if limit.is_none()
            {
                cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid limit"));
                return Ok(());
            }

            send_history_for_target_reverse(server, cmd, source, &target, from_ts, None, limit)?;
        }
        "BEFORE" =>
        {
            let target = arg_1.to_string();
            let end_ts = match utils::parse_timestamp(arg_2)
            {
                Some(ts) => ts,
                None => {
                    cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid timestamp"));
                    return Ok(());
                }
            };

            let limit = arg_3.parse().ok();
            if limit.is_none()
            {
                cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid limit"));
                return Ok(());
            }

            send_history_for_target_reverse(server, cmd, source, &target, None, Some(end_ts), limit)?;
        }
        "AFTER" =>
        {
            let target = arg_1.clone();
            let start_ts = match utils::parse_timestamp(arg_2)
            {
                Some(ts) => ts,
                None => {
                    cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid timestamp"));
                    return Ok(());
                }
            };

            let limit = arg_3.parse().ok();
            if limit.is_none()
            {
                cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid limit"));
                return Ok(());
            }

            send_history_for_target_forward(server, cmd, source, &target, Some(start_ts), None, limit)?;
        }
        "AROUND" =>
        {
            let target = arg_1.clone();
            let around_ts = match utils::parse_timestamp(arg_2)
            {
                Some(ts) => ts,
                None => {
                    cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid timestamp"));
                    return Ok(());
                }
            };

            let limit = match arg_3.parse::<usize>().ok()
            {
                Some(limit) => limit,
                None =>
                {
                    cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid limit"));
                    return Ok(());
                }
            };

            send_history_for_target_reverse(server, cmd, source, &target, Some(around_ts), None, Some(limit/2))?;
            send_history_for_target_forward(server, cmd, source, &target, Some(around_ts), None, Some(limit/2))?;
        }
        "BETWEEN" =>
        {
            let target = arg_1.clone();
            let start_ts = match utils::parse_timestamp(arg_2)
            {
                Some(ts) => ts,
                None => {
                    cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid timestamp"));
                    return Ok(());
                }
            };
            let end_ts = match utils::parse_timestamp(arg_3)
            {
                Some(ts) => ts,
                None => {
                    cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid timestamp"));
                    return Ok(());
                }
            };

            let limit = arg_4.and_then(|arg| arg.parse().ok());
            if limit.is_none()
            {
                cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", "", "Invalid limit"));
                return Ok(());
            }

            send_history_for_target_forward(server, cmd, source.deref(), &target, Some(start_ts), Some(end_ts), limit)?;
        }
        _ =>
        {
            cmd.response(&message::Fail::new("CHATHISTORY", "INVALID_PARAMS", subcommand, "Invalid subcommand"));
        }
    }

    Ok(())
}

// Helper to extract the target name for chathistory purposes from a given event.
// This might be the source or target of the actual event, or might be None if it's
// an event type that we don't include in history playback
fn target_name_for_entry(for_user: UserId, entry: &HistoryLogEntry) -> Option<String>
{
    match &entry.details
    {
        NetworkStateChange::NewMessage(message) =>
        {
            if matches!(&message.target, HistoricMessageTarget::User(user) if user.user.id == for_user)
            {
                Some(messages::MessageTarget::format(&message.source))
            }
            else
            {
                Some(message.target.format())
            }
        }
        _ => None
    }
}

// For listing targets, we iterate backwards through time; this allows us to just collect the
// first timestamp we see for each target and know that it's the most recent one
fn list_targets(server: &ClientServer, into: &(impl MessageSink + ?Sized), source: &wrapper::User, from_ts: Option<i64>, to_ts: Option<i64>, limit: Option<usize>)
{
    let log = server.node().history();
    let mut found_targets = HashMap::new();

    for entry in log.entries_for_user_reverse(source.id())
    {
        if matches!(to_ts, Some(ts) if entry.timestamp >= ts)
        {
            // Skip over until we hit the timestamp window we're interested in
            continue;
        }
        if matches!(from_ts, Some(ts) if entry.timestamp <= ts)
        {
            // We're iterating backwards through time; if we hit this then we've
            // passed the requested window and should stop
            break;
        }


        if let Some(target_name) = target_name_for_entry(source.id(), entry)
        {
            found_targets.entry(target_name).or_insert(entry.timestamp);
        }

        // If this pushes us past the the requested limit, stop
        if matches!(limit, Some(limit) if limit <= found_targets.len())
        {
            break;
        }
    }

    for (target, timestamp) in found_targets
    {
        into.send(&message::ChatHistoryTarget::new(&target, &utils::format_timestamp(timestamp)))
    }
}

fn send_history_for_target_forward(server: &ClientServer, into: &(impl MessageSink + ?Sized), source: &wrapper::User, target: &str, from_ts: Option<i64>, to_ts: Option<i64>, limit: Option<usize>) -> CommandResult
{
    let log = server.node().history();
    let mut entries = Vec::new();

    for entry in log.entries_for_user(source.id())
    {
        if matches!(from_ts, Some(ts) if entry.timestamp <= ts)
        {
            // Skip over until we hit the timestamp window we're interested in
            continue;
        }
        if matches!(to_ts, Some(ts) if entry.timestamp >= ts)
        {
            // If we hit this then we've passed the requested window and should stop
            break;
        }

        if let Some(event_target) = target_name_for_entry(source.id(), entry)
        {
            if event_target == target
            {
                entries.push(entry);
            }
        }

        if matches!(limit, Some(limit) if limit <= entries.len())
        {
            break;
        }
    }

    for entry in entries
    {
        entry.send_to(into, entry)?;
    }

    Ok(())
}

// As above, but work backwards
fn send_history_for_target_reverse(server: &ClientServer, into: &(impl MessageSink + ?Sized), source: &wrapper::User, target: &str, from_ts: Option<i64>, to_ts: Option<i64>, limit: Option<usize>) -> CommandResult
{
    let log = server.node().history();
    let mut entries = Vec::new();

    for entry in log.entries_for_user_reverse(source.id())
    {
        if matches!(from_ts, Some(ts) if entry.timestamp <= ts)
        {
            // Skip over until we hit the timestamp window we're interested in
            continue;
        }
        if matches!(to_ts, Some(ts) if entry.timestamp >= ts)
        {
            // If we hit this then we've passed the requested window and should stop
            break;
        }

        if let Some(event_target) = target_name_for_entry(source.id(), entry)
        {
            if event_target == target
            {
                entries.push(entry);
            }
        }

        if matches!(limit, Some(limit) if limit <= entries.len())
        {
            break;
        }
    }

    for entry in entries.into_iter().rev()
    {
        entry.send_to(into, entry)?;
    }

    Ok(())
}
