use irc_network::wrapper::*;
use irc_network::modes::*;

pub fn format_cmode_changes(added: &ChannelModeSet, removed: &ChannelModeSet) -> String
{
    let mut changes = String::new();
    if ! added.is_empty()
    {
        changes += "+";
        changes += &added.to_chars();
    }
    if ! removed.is_empty()
    {
        changes += "-";
        changes += &removed.to_chars();
    }

    changes
}

pub fn format_channel_perm_changes(target: &User, added: &MembershipFlagSet, removed: &MembershipFlagSet) -> (String, Vec<String>)
{
    let mut changes = String::new();
    let mut args = Vec::new();

    let nick = target.nick();

    if ! added.is_empty()
    {
        changes += "+";
        for (flag,modechar,_) in MembershipFlagSet::all()
        {
            if added.is_set(flag) {
                changes += &modechar.to_string();
                args.push(nick.to_string());
            }
        }
    }
    if ! removed.is_empty()
    {
        changes += "-";
        for (flag,modechar,_) in MembershipFlagSet::all()
        {
            if removed.is_set(flag) {
                changes += &modechar.to_string();
                args.push(nick.to_string());
            }
        }
    }

    (changes, args)
}