use anyhow::{Context, Ok};

use crate::{resp::Resp, store::Store};

pub(crate) enum Command {
    Ping,
    Echo { name: String },
    Get { key: String },
    Set { key: String, value: String },
}

impl Command {
    pub(crate) fn parse(args: Vec<String>) -> anyhow::Result<Command> {
        let command = args.first().context("Invalid command")?;

        match command.to_lowercase().as_str() {
            "ping" => Ok(Command::Ping),
            "echo" => {
                let name = args
                    .get(1)
                    .context("Missing argument 'name' for ECHO command")?;

                Ok(Command::Echo { name: name.clone() })
            }
            "get" => {
                let key = args
                    .get(1)
                    .context("Missing argument 'key' for GET command")?;

                Ok(Command::Get { key: key.clone() })
            }
            "set" => {
                let key = args
                    .get(1)
                    .context("Missing argument 'key' for SET command")?;
                let value = args
                    .get(2)
                    .context("Missing argument 'key' for SET command")?;

                Ok(Command::Set {
                    key: key.clone(),
                    value: value.clone(),
                })
            }
            _ => anyhow::bail!("Unknown command encountered"),
        }
    }

    pub(crate) async fn execute(self, store: &mut Store) -> anyhow::Result<String> {
        let result: Resp = match self {
            Command::Ping => Resp::SimpleString("PONG".to_string()),
            Command::Echo { name } => Resp::BulkString(Some(name)),
            Command::Get { key } => match store.get(&key) {
                Some(value) => Resp::BulkString(Some(value)),
                None => Resp::null(),
            },
            Command::Set { key, value } => {
                store
                    .set(&key, &value)
                    .with_context(|| format!("Failed to write data to store: {}", key))?;

                Resp::ok()
            }
        };

        Ok(result.encode())
    }
}
