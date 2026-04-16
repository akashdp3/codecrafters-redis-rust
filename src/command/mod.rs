use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Ok};
use tokio::sync::Mutex;

use crate::{resp::Resp, store::Store};

mod config;
mod get;
mod info;
mod keys;
mod psync;
mod repl_conf;
mod set;
mod wait;

#[derive(Debug)]
pub(crate) enum Command {
    Ping,
    Echo {
        name: String,
    },
    Get {
        key: String,
    },
    Set {
        key: String,
        value: String,
        expiry: Option<Duration>,
    },
    Config {
        op: config::Op,
        name: config::Name,
    },
    Keys {
        pattern: String,
    },
    Info {
        kind: info::InfoKind,
    },
    ReplConf {
        key: repl_conf::Kind,
        value: String,
    },
    Psync {
        repl_id: String,
        offset: String,
    },
    Wait {
        numreplicas: u8,
        timeout: u16,
    },
    Type {
        key: String,
    },
}

impl Command {
    pub(crate) fn parse(args: Vec<String>) -> anyhow::Result<Command> {
        let mut args = args.into_iter();
        let command = args.next().context("Invalid command")?;

        match command.to_lowercase().as_str() {
            "ping" => Ok(Command::Ping),
            "echo" => {
                let name = args
                    .next()
                    .context("Missing argument 'name' for ECHO command")?;

                Ok(Command::Echo { name })
            }
            "get" => get::parse(&mut args),
            "set" => set::parse(&mut args),
            "config" => config::parse(&mut args),
            "keys" => keys::parse(&mut args),
            "info" => info::parse(&mut args),
            "replconf" => repl_conf::parse(&mut args),
            "psync" => psync::parse(&mut args),
            "wait" => wait::parse(&mut args),
            "type" => {
                let key = args
                    .next()
                    .context("Missing argument 'key' for TYPE command")?;

                Ok(Command::Type { key })
            }
            _ => anyhow::bail!("Unknown command encountered: {}", command),
        }
    }

    pub(crate) async fn execute(self, store: Arc<Mutex<Store>>) -> anyhow::Result<Vec<u8>> {
        let result: Vec<u8> = match self {
            Command::Ping => Resp::SimpleString("PONG".to_string()).encode().into_bytes(),
            Command::Echo { name } => Resp::bulk(name).encode().into_bytes(),
            Command::Get { key } => {
                let mut s = store.lock().await;
                get::invoke(&mut s, &key)?.encode().into_bytes()
            }
            Command::Set { key, value, expiry } => {
                let mut s = store.lock().await;
                set::invoke(&mut s, key, value, expiry)?
                    .encode()
                    .into_bytes()
            }
            Command::Config { op, name } => {
                let s = store.lock().await;
                config::invoke(&s, op, name)?.encode().into_bytes()
            }
            Command::Keys { pattern } => {
                let mut s = store.lock().await;
                keys::invoke(&mut s, &pattern)?.encode().into_bytes()
            }
            Command::Info { kind } => {
                let mut s = store.lock().await;
                info::invoke(&mut s, kind)?.encode().into_bytes()
            }
            Command::ReplConf { key, value } => {
                let mut s = store.lock().await;
                repl_conf::invoke(&mut s, key, &value)?
                    .encode()
                    .into_bytes()
            }
            Command::Psync { repl_id, offset } => {
                let mut s = store.lock().await;
                psync::invoke(&mut s, &repl_id, &offset)?
            }
            Command::Wait {
                numreplicas,
                timeout,
            } => wait::invoke(store, numreplicas, timeout).await?,
            Command::Type { key } => {
                let store = store.lock().await;
                let response = match store.db.get(&key) {
                    Some(_) => Ok(Resp::SimpleString("string".to_string())),
                    None => Ok(Resp::SimpleString("none".to_string())),
                }?;

                response.encode().into_bytes()
            }
        };

        Ok(result)
    }
}
