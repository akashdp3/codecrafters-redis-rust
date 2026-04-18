use crate::{store, Command, Resp, Store};
use anyhow::Context;

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = args
        .next()
        .context("Missing argument 'key' for GET command")?;

    Ok(Command::Get { key })
}

pub(crate) fn invoke(store: &mut Store, key: &str) -> anyhow::Result<Resp> {
    match store.db.get(key) {
        Some(store::RedisValue::String(value)) => Ok(Resp::bulk(value)),
        Some(store::RedisValue::Stream(..)) => Ok(Resp::error("INVALID TYPE")),
        _ => Ok(Resp::null()),
    }
}
