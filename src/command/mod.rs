use std::sync::Arc;

use anyhow::{Context, Ok};
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};

use crate::store::RedisValue;
use crate::{resp::Resp, store::Store, utils::next_arg};

mod blpop;
mod config;
mod get;
mod incr;
mod info;
mod keys;
mod llen;
mod lpop;
mod lrange;
mod psync;
mod repl_conf;
mod rpush;
mod set;
mod type_cmd;
mod wait;
mod xadd;
mod xrange;
mod xread;

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
    Xadd {
        key: String,
        id: String,
        fields: Vec<(String, String)>,
    },
    Xrange {
        key: String,
        start_id: String,
        end_id: String,
    },
    Xread {
        entity: xread::Entity,
        keys: Vec<String>,
        ids: Vec<String>,
    },
    Incr {
        key: String,
    },
    Multi,
    Exec,
    Discard,
    LPush {
        key: String,
        elements: Vec<String>,
    },
    RPush {
        key: String,
        elements: Vec<String>,
    },
    LRange {
        key: String,
        start: isize,
        end: isize,
    },
    LLen {
        key: String,
    },
    LPop {
        key: String,
        count: usize,
    },
    BLPop {
        key: String,
        timeout: f32,
    },
}

impl Command {
    pub(crate) fn parse(args: Vec<String>) -> anyhow::Result<Command> {
        let mut args = args.into_iter();
        let command = args.next().context("Invalid command")?;

        match command.to_lowercase().as_str() {
            "ping" => Ok(Command::Ping),
            "echo" => Ok(Command::Echo {
                name: next_arg(&mut args, "ECHO", "name")?,
            }),
            "get" => get::parse(&mut args),
            "set" => set::parse(&mut args),
            "config" => config::parse(&mut args),
            "keys" => keys::parse(&mut args),
            "info" => info::parse(&mut args),
            "replconf" => repl_conf::parse(&mut args),
            "psync" => psync::parse(&mut args),
            "wait" => wait::parse(&mut args),
            "type" => type_cmd::parse(&mut args),
            "xadd" => xadd::parse(&mut args),
            "xrange" => xrange::parse(&mut args),
            "xread" => xread::parse(&mut args),
            "incr" => incr::parse(&mut args),
            "multi" => Ok(Command::Multi),
            "exec" => Ok(Command::Exec),
            "discard" => Ok(Command::Discard),
            "lpush" => {
                let key = next_arg(&mut args, "LPUSH", "key")?;
                let elements = args.collect();

                Ok(Command::LPush { key, elements })
            }
            "rpush" => rpush::parse(&mut args),
            "lrange" => lrange::parse(&mut args),
            "llen" => llen::parse(&mut args),
            "lpop" => lpop::parse(&mut args),
            "blpop" => blpop::parse(&mut args),
            _ => anyhow::bail!("Unknown command encountered: {}", command),
        }
    }

    pub(crate) async fn execute(self, store: Arc<Mutex<Store>>) -> anyhow::Result<Vec<u8>> {
        let result: Vec<u8> = match self {
            Command::Ping => Resp::SimpleString("PONG".to_string()).encode().into_bytes(),
            Command::Echo { name } => Resp::bulk(name).encode().into_bytes(),
            Command::Get { key } => respond(&store, |s| get::invoke(s, &key)).await?,
            Command::Set { key, value, expiry } => {
                respond(&store, |s| {
                    set::invoke(s, key, RedisValue::String(value), expiry)
                })
                .await?
            }
            Command::Config { op, name } => {
                respond(&store, |s| config::invoke(s, op, name)).await?
            }
            Command::Keys { pattern } => respond(&store, |s| keys::invoke(s, &pattern)).await?,
            Command::Info { kind } => respond(&store, |s| info::invoke(s, kind)).await?,
            Command::ReplConf { key, value } => {
                respond(&store, |s| repl_conf::invoke(s, key, &value)).await?
            }
            Command::Psync { repl_id, offset } => {
                let mut s = store.lock().await;
                psync::invoke(&mut s, &repl_id, &offset)?
            }
            Command::Wait {
                numreplicas,
                timeout,
            } => wait::invoke(store, numreplicas, timeout).await?,
            Command::Type { key } => respond(&store, |s| type_cmd::invoke(s, &key)).await?,
            Command::Xadd { key, id, fields } => {
                respond(&store, |s| xadd::invoke(s, key, id, fields)).await?
            }
            Command::Xrange {
                key,
                start_id,
                end_id,
            } => respond(&store, |s| xrange::invoke(s, key, start_id, end_id)).await?,
            Command::Xread { entity, keys, ids } => {
                match entity {
                    xread::Entity::Streams => {
                        respond(&store, |s| xread::invoke(s, keys, ids)).await?
                    }
                    xread::Entity::Block(timeout_ms) => {
                        let (notified, ids) = {
                            let s = store.lock().await;
                            // Pin `$` to the latest id *before* waiting, so we only
                            // return entries added after this XREAD started.
                            let ids = xread::resolve_ids(&s, &keys, ids);
                            let result = xread::invoke(&s, keys.clone(), ids.clone())?;
                            if !matches!(&result, Resp::Array(v) if v.is_empty()) {
                                return Ok(result.encode().into_bytes());
                            }
                            (s.notify.clone(), ids)
                        };
                        let notified = notified.notified();

                        if timeout_ms == 0 {
                            notified.await
                        } else {
                            let timeout = Instant::now() + Duration::from_millis(timeout_ms);
                            if tokio::time::timeout_at(timeout, notified).await.is_err() {
                                return Ok(Resp::NullArray.encode().into_bytes());
                            }
                        }

                        respond(&store, |s| xread::invoke(s, keys, ids)).await?
                    }
                }
            }
            Command::Incr { key } => respond(&store, |s| incr::invoke(s, &key)).await?,
            Command::LPush { key, elements } => {
                respond(&store, |s| rpush::invoke(s, key, elements, true)).await?
            }
            Command::RPush { key, elements } => {
                respond(&store, |s| rpush::invoke(s, key, elements, false)).await?
            }
            Command::LRange { key, start, end } => {
                respond(&store, |s| lrange::invoke(s, &key, start, end)).await?
            }
            Command::LLen { key } => respond(&store, |s| llen::invoke(s, &key)).await?,
            Command::LPop { key, count } => {
                respond(&store, |s| lpop::invoke(s, &key, count)).await?
            }
            Command::BLPop { key, timeout } => {
                let notify = {
                    let mut s = store.lock().await;
                    s.to_be_notified_items.push(key.clone());

                    s.notify.clone()
                };
                let notified = notify.notified();

                if timeout == 0.0 {
                    notified.await
                } else {
                    let timeout = Instant::now() + Duration::from_secs_f32(timeout);
                    if tokio::time::timeout_at(timeout, notified).await.is_err() {
                        return Ok(Resp::null_array().encode().into_bytes());
                    }
                }

                respond(&store, |s| blpop::invoke(s, key)).await?
            }
            _ => {
                anyhow::bail!("Invalid command")
            }
        };

        Ok(result)
    }
}

async fn respond(
    store: &Arc<Mutex<Store>>,
    f: impl FnOnce(&mut Store) -> anyhow::Result<Resp>,
) -> anyhow::Result<Vec<u8>> {
    let mut s = store.lock().await;
    Ok(f(&mut s)?.encode().into_bytes())
}
