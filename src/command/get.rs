use crate::{store, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "GET";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;

    Ok(Command::Get { key })
}

pub(crate) fn invoke(store: &mut Store, key: &str) -> anyhow::Result<Resp> {
    match store.db.get(key) {
        Some(store::RedisValue::String(value)) => Ok(Resp::bulk(value)),
        Some(store::RedisValue::Stream(..)) => Ok(Resp::error("INVALID TYPE")),
        _ => Ok(Resp::null()),
    }
}
