use anyhow::{Context, Ok};

use crate::{store, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "INCR";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;

    Ok(Command::Incr { key })
}

pub(crate) fn invoke(store: &mut Store, key: &str) -> anyhow::Result<Resp> {
    match store.db.get(key) {
        Some(store::RedisValue::String(value)) => {
            let val: usize = value
                .parse()
                .context("Error while performing INCR on string value")?;
            let new_val = val + 1;

            store.db.update(key, new_val.to_string())?;
            Ok(Resp::Integer(new_val))
        }
        None => {
            let new_val = 1;
            store.db.set(key.to_string(), new_val.to_string(), None)?;
            Ok(Resp::Integer(new_val))
        }
        _ => anyhow::bail!("Error while performing INCR on non-numeric value"),
    }
}
