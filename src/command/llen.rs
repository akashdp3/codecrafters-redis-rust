use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "LLEN";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;

    Ok(Command::LLen { key })
}

pub(crate) fn invoke(store: &mut Store, key: &str) -> anyhow::Result<Resp> {
    let len = match store.db.get(key) {
        Some(RedisValue::List(value)) => value.len(),
        Some(..) => anyhow::bail!("Invalid Operation ! Can't perform LLEN on non-list type value"),
        None => 0,
    };

    Ok(Resp::integer(len))
}
