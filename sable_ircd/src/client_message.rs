use client_listener::ConnectionId;

/// A tokenised, but not yet processed, message from a client connection
#[derive(Debug)]
pub struct ClientMessage
{
    /// The connection from which the message was received
    pub source: ConnectionId,
    /// The command
    pub command: String,
    /// The list of arguments
    pub args: Vec<String>
}

impl ClientMessage
{
    /// Create a `ClientMessage` from a received message
    pub fn parse(source: ConnectionId, raw: &str) -> Option<Self>
    {
        let mut args = Vec::new();
        let raw = raw.trim_start();
        if raw.is_empty()
        {
            return None;
        }

        let offset = match raw.find(' ')
        {
            Some(offset) => offset,
            None => {
                return Some(Self {
                    source,
                    command: raw.to_string(),
                    args: Vec::new()
                });
            }
        };

        let command = &raw[0 .. offset];
        let mut rest = &raw[offset+1 .. ];

        loop
        {
            if let Some(arg) = rest.strip_prefix(':')
            {
                if !arg.is_empty()
                {
                    args.push(arg.to_string());
                }
                break;
            }

            match rest.find(' ')
            {
                Some(offset) =>
                {
                    let arg = &rest[0 .. offset];

                    if !arg.is_empty()
                    {
                        args.push(arg.to_string());
                    }

                    rest = &rest[offset + 1 ..];
                }
                None =>
                {
                    if !rest.is_empty()
                    {
                        args.push(rest.to_string());
                    }
                    break;
                }
            }
        }

        Some(Self {
            source,
            command: command.to_string(),
            args
        })
    }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use client_listener::*;

    fn get_connid() -> ConnectionId
    {
        let listener_id = ListenerIdGenerator::new(0).next();
        ConnectionIdGenerator::new(listener_id, 0).next()
    }

    #[test]
    fn no_args()
    {
        let msg = ClientMessage::parse(get_connid(), "command").unwrap();
        assert_eq!(msg.command, "command");
        assert_eq!(msg.args.len(), 0);
    }

    #[test]
    fn simple_args()
    {
        let msg = ClientMessage::parse(get_connid(), "command arg1 arg2 :arg three").unwrap();

        assert_eq!(msg.command, "command");
        assert_eq!(msg.args, &["arg1", "arg2", "arg three"]);
    }

    #[test]
    fn ending_space()
    {
        let msg = ClientMessage::parse(get_connid(), "command arg1 arg2 ").unwrap();

        assert_eq!(msg.args, &["arg1", "arg2"]);
    }

    #[test]
    fn ending_colon()
    {
        let msg = ClientMessage::parse(get_connid(), "command arg1 arg2 :").unwrap();

        assert_eq!(msg.args, &["arg1", "arg2"]);

    }

    #[test]
    fn double_space()
    {
        let msg = ClientMessage::parse(get_connid(), "command arg1  arg2").unwrap();

        assert_eq!(msg.args, &["arg1", "arg2"]);
    }

    #[test]
    fn colon_space()
    {
        let msg = ClientMessage::parse(get_connid(), "command arg1 : arg2").unwrap();

        assert_eq!(msg.args, &["arg1", " arg2"]);
    }

    #[test]
    fn empty()
    {
        assert!(ClientMessage::parse(get_connid(), "").is_none());
    }

    #[test]
    fn leading_space()
    {
        let msg = ClientMessage::parse(get_connid(), "    command arg1 arg2 :arg three").unwrap();

        assert_eq!(msg.command, "command");
        assert_eq!(msg.args, &["arg1", "arg2", "arg three"]);
    }
}