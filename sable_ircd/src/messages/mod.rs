use sable_network::prelude::*;
use crate::command::*;

/// Trait describing an object that can be the source of a client protocol message
pub trait MessageSource
{
    fn format(&self) -> String;
}

/// Trait describing an object that can be the target of a client protocol message
pub trait MessageTarget
{
    fn format(&self) -> String;
}

/// Placeholder type to denote that a message is being sent to a target whose name we
/// don't know - e.g. a pre-registration client, or a snote being sent to multiple users
pub struct UnknownTarget;

impl MessageSource for &crate::ClientServer
{
    fn format(&self) -> String { self.node().name().to_string() }
}

impl MessageSource for crate::ClientServer
{
    fn format(&self) -> String { self.node().name().to_string() }
}

impl MessageSource for ServerName
{
    fn format(&self) -> String { self.to_string() }
}

impl MessageSource for String
{
    fn format(&self) -> String { self.clone() }
}

impl MessageSource for wrapper::User<'_>
{
    fn format(&self) -> String { format!("{}!{}@{}", self.nick(), self.user(), self.visible_host()) }
}

impl MessageSource for update::HistoricMessageSource
{
    fn format(&self) -> String
    {
        match self
        {
            Self::User(historic_user) => {
                <update::HistoricUser as MessageSource>::format(historic_user)
            }
            Self::Server(server) => {
                server.name.to_string()
            }
            Self::Unknown => {
                "*".to_string()
            }
        }
    }
}

impl MessageSource for update::HistoricUser
{
    fn format(&self) -> String
    {
        format!("{}!{}@{}", self.nickname, self.user.user, self.user.visible_host)
    }
}

impl <T: MessageSource> MessageSource for std::sync::Arc<T>
{
    fn format(&self) -> String
    {
        use std::ops::Deref;
        self.deref().format()
    }
}

impl MessageTarget for wrapper::User<'_>
{
    fn format(&self) -> String { self.nick().to_string() }
}

impl MessageTarget for wrapper::Channel<'_>
{
    fn format(&self) -> String { self.name().to_string() }
}

impl MessageTarget for state::Channel
{
    fn format(&self) -> String { self.name.to_string() }
}

impl MessageTarget for UnknownTarget
{
    fn format(&self) -> String { "*".to_string() }
}

impl MessageTarget for Nickname
{
    fn format(&self) -> String { self.value().to_string() }
}

impl MessageTarget for update::HistoricUser
{
    fn format(&self) -> String { self.nickname.to_string() }
}

impl MessageTarget for update::HistoricMessageTarget
{
    fn format(&self) -> String
    {
        match self
        {
            Self::Channel(c) => c.name.to_string(),
            Self::User(hu) => hu.nickname.to_string(),
            Self::Unknown => "*".to_string()
        }
    }
}

// This may seem counter-intuitive, but there are times we need to
// format a message source as if it were a target
impl MessageTarget for update::HistoricMessageSource
{
    fn format(&self) -> String
    {
        match self
        {
            Self::Server(s) => s.name.to_string(),
            Self::User(u) => u.nickname.to_string(),
            Self::Unknown => "*".to_string()
        }
    }
}

impl MessageTarget for wrapper::MessageTarget<'_>
{
    fn format(&self) -> String
    {
        match self
        {
            Self::Channel(c) => c.format(),
            Self::User(u) => MessageTarget::format(u)
        }
    }
}

// Used when command parsing/processing fails
impl MessageTarget for CommandSource<'_>
{
    fn format(&self) -> String
    {
        match self
        {
            Self::User(u) => <wrapper::User as MessageTarget>::format(u),
            Self::PreClient(_) => "*".to_string()
        }
    }
}

/// Trait describing a client protocol message type
pub trait MessageType : std::fmt::Debug + MessageTypeFormat
{ }

/// Trait that determines how to format a client message
pub trait MessageTypeFormat
{
    fn format_for_client_caps(&self, caps: &super::capability::ClientCapabilitySet) -> Option<String>;
}

impl<T: std::fmt::Display> MessageTypeFormat for T
{
    fn format_for_client_caps(&self, _caps: &super::capability::ClientCapabilitySet) -> Option<String>
    {
        Some(self.to_string())
    }
}

/// A `Numeric` that has been formatted for a specific source and target
#[derive(Debug)]
pub struct TargetedNumeric(String);

impl std::fmt::Display for TargetedNumeric
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
    {
        self.0.fmt(f)
    }
}

impl MessageType for TargetedNumeric { }

/// Trait describing a numeric message
pub trait Numeric : std::fmt::Debug
{
    fn format_for(&self, source: &dyn MessageSource, target: &dyn MessageTarget) -> TargetedNumeric;
    fn message(&self) -> &str;
}

pub mod message;
pub mod numeric;
pub mod send_history;
pub mod send_realtime;

mod message_sink;
pub use message_sink::*;