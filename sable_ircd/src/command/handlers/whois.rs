use super::*;

#[sable_macros::command_handler("WHOIS")]
fn whois_handler(cmd: &dyn Command, _source: UserSource, target: wrapper::User) -> CommandResult
{
    cmd.numeric(make_numeric!(WhoisUser, &target));

    if let Ok(server) = target.server()
    {
        cmd.numeric(make_numeric!(WhoisServer, &target, &server));
    }

    if let Ok(Some(account)) = target.account()
    {
        cmd.numeric(make_numeric!(WhoisAccount, &target, &account.name()));
    }

    cmd.numeric(make_numeric!(EndOfWhois, &target));
    Ok(())
}
