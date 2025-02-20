use sable_network::prelude::*;

use sable_macros::define_messages;
use super::*;

define_messages! {
    Cap     => { (source, target, subcmd: &str, text: &str) => ":{source} CAP {target} {subcmd} :{text}" },
    Nick    => { (source, newnick: &Nickname)               => ":{source} NICK {newnick}" },
    Join    => { (source, chan: &ChannelName)               => ":{source} JOIN {chan}" },
    Part    => { (source, chan: &ChannelName, msg: &str)    => ":{source} PART {chan} :{msg}" },
    Invite  => { (source, target, chan: &ChannelName)       => ":{source} INVITE {target} :{chan}" },
    Quit    => { (source, message: &str)                    => ":{source} QUIT :{message}" },
    Topic   => { (source, chan: &ChannelName, text: &str)   => ":{source} TOPIC {chan} :{text}" },

    Mode    => { (source, target, changes: &str)            => ":{source} MODE {target} {changes}" },

    Notice  => { (source, target, message: &str)            => ":{source} NOTICE {target} :{message}" },
    Privmsg => { (source, target, message: &str)            => ":{source} PRIVMSG {target} :{message}" },
    Message => { (source, target, message_type: state::MessageType, message: &str)
                                                            => ":{source} {message_type} {target} :{message}" },

    Ping    => { (source, target, cookie: &str)             => ":{source} PING {target} :{cookie}" },
    Pong    => { (source, cookie: &str)                     => ":{source} PONG {source} :{cookie}" },

    Error   => { (text: &str)   => "ERROR :{text}" },

    // IRCv3 standard reply messages
    Fail    => { (command: &str, code: &str, context: &str, description: &str)
                                => "FAIL {command} {code} {context} :{description}" },
    Warn    => { (command: &str, code: &str, context: &str, description: &str)
                                => "WARN {command} {code} {context} :{description}" },
    Note    => { (command: &str, code: &str, context: &str, description: &str)
                                => "NOTE {command} {code} {context} :{description}" },

    // SASL
    Authenticate    => { (data: &str) => "AUTHENTICATE :{data}" },

    // Extension messages
    ChatHistoryTarget => { (target_name: &str, timestamp: &str) => "CHATHISTORY TARGETS {target_name} {timestamp}" },
    Register => { (status: &str, account: Nickname, message: &str) => "REGISTER {status} {account} {message}" }
}