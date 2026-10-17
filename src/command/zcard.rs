use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "ZCARD";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;

    Ok(Command::Zcard { key })
}

pub(crate) fn invoke(store: &mut Store, key: String) -> anyhow::Result<Resp> {
    let len = match store.db.get(&key) {
        Some(value) => match value {
            RedisValue::SortedSet(items) => items.len(),
            _ => anyhow::bail!("Invalid Operation"),
        },
        None => 0,
    };

    Ok(Resp::integer(len))
}
