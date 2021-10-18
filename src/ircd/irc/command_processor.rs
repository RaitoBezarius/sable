use crate::ircd::*;
use super::*;
use crate::utils::*;
use std::cell::RefCell;

pub struct CommandProcessor<'a>
{
    server: &'a Server,
    actions: Vec<CommandAction>
}

pub enum CommandAction {
    StateChange(event::Event),
    RegisterClient(ConnectionId),
    DisconnectUser(UserId),
}

#[derive(Debug)]
pub enum CommandError
{
    UnderlyingError(Box<dyn std::error::Error>),
    UnknownError(String),
    NoSuchTarget(String),
    Numeric(Box<dyn messages::Message>)
}

pub enum CommandSource<'a>
{
    PreClient(&'a RefCell<PreClient>),
    User(wrapper::User<'a>),
}

pub struct ClientCommand<'a>
{
    pub connection: &'a ClientConnection,
    pub source: CommandSource<'a>,
    pub command: String,
    pub args: Vec<String>,
}

impl<T: messages::Message + 'static> From<T> for CommandError
{
    fn from(t: T) -> Self {
        Self::Numeric(Box::new(t))
    }
}

impl<'a> CommandProcessor<'a>
{
    pub fn new (server: &'a Server) -> Self
    {
        Self {
            server: server,
            actions: Vec::new()
        }
    }

    pub fn actions(self) -> Vec<CommandAction>
    {
        self.actions
    }

    pub async fn process_message(&mut self, message: ClientMessage)
    {
        if let Some(conn) = self.server.find_connection(message.source)
        {
            let command = message.command.clone();
            if let Err(err) = self.do_process_message(conn, message)
            {
                match err {
                    CommandError::UnderlyingError(err) => {
                        panic!("Error occurred handling command {} from {:?}: {}", command, conn.id(), err);
                    },
                    CommandError::UnknownError(desc) => {
                        panic!("Error occurred handling command {} from {:?}: {}", command, conn.id(), desc);
                    }
                    CommandError::NoSuchTarget(t) => {
                        if let Ok(source) = self.translate_message_source(conn) {
                            conn.send(&numeric::NoSuchTarget::new(self.server, &source, &t)).await.or_log("sending error numeric");
                        }
                    }
                    CommandError::Numeric(num) => {
                        conn.send(num.as_ref()).await.or_log("sending error numeric");
                    }
                }
            }
        } else {
            panic!("Got message '{}' from unknown source?", message.command);
        }
    }

    fn do_process_message(&mut self, connection: &ClientConnection, message: ClientMessage) -> Result<(), CommandError>
    {
        let source = self.translate_message_source(connection)?;
        if let Some(handler) = self.server.command_dispatcher().resolve_command(&message.command) {
            let cmd = ClientCommand {
                 connection: connection, 
                 source: source,
                 command: message.command,
                 args: message.args
            };

            handler.validate(self.server, &cmd)?;
            handler.handle(self.server, &cmd, &mut self.actions)
        } else {
            Err(numeric::UnknownCommand::new(self.server, &source, &message.command).into())
        }
    }

    fn translate_message_source(&self, source: &'a ClientConnection) -> Result<CommandSource<'a>, CommandError> {
        if let Some(user_id) = source.user_id {
            Ok(self.server.network().user(user_id).map(|u| CommandSource::User(u))?)
        } else if let Some(pre_client) = &source.pre_client {
            Ok(CommandSource::PreClient(pre_client))
        } else {
            Err(CommandError::unknown("Got message from neither preclient nor client"))
        }
    }
}

impl CommandError
{
    pub fn unknown(desc: impl std::fmt::Display) -> Self
    {
        Self::UnknownError(desc.to_string())
    }

    pub fn inner(err: impl std::error::Error + Clone + 'static) -> CommandError
    {
        CommandError::UnderlyingError(Box::new(err.clone()))
    }
}

impl From<LookupError> for CommandError
{
    fn from(e: LookupError) -> Self
    {
        match e {
            LookupError::NoSuchNick(n) => Self::NoSuchTarget(n),
            LookupError::NoSuchChannelName(n) => Self::NoSuchTarget(n),
            _ => Self::UnknownError(e.to_string())
        }
    }
}