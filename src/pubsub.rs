use crate::{utils::next_arg, Client, Resp};

enum Command {
    Ping,
    Subscribe { channel: String },
    Unsubscribe,
    Psubscribe,
    Punsubscribe,
    Quit,
}

impl TryFrom<Vec<String>> for Command {
    type Error = String;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        let mut args = value.into_iter();
        let command = args.next().ok_or("No command found")?;
        let command = command.to_ascii_lowercase();
        let channel = args.next().ok_or("No channel found")?;

        let cmd = match command.as_str() {
            "ping" => Command::Ping,
            "subscribe" => Command::Subscribe { channel },
            "unsubscribe" => Command::Unsubscribe,
            "psubscribe" => Command::Psubscribe,
            "punsubscribe" => Command::Punsubscribe,
            "quit" => Command::Quit,
            _ => {
                let err_msg = format!("Can't execute '{}' in subscribed mode", command);
                return Err(err_msg);
            }
        };

        Ok(cmd)
    }
}

pub(crate) struct PubSub;

impl PubSub {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn handle(&self, args: Vec<String>, client: &mut Client) -> Resp {
        let cmd = match Command::try_from(args) {
            Ok(cmd) => cmd,
            Err(msg) => return Resp::error(&msg),
        };

        match cmd {
            Command::Ping => {
                todo!()
            }
            Command::Subscribe { channel } => {
                client.subscribe(channel.clone());
                Resp::Array(vec![
                    Resp::bulk("subscribe"),
                    Resp::bulk(channel),
                    Resp::integer(client.subscriber_count()),
                ])
            }
            Command::Unsubscribe => todo!(),
            Command::Psubscribe => todo!(),
            Command::Punsubscribe => todo!(),
            Command::Quit => todo!(),
        }
    }
}
