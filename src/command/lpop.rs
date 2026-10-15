use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "LPOP";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;

    Ok(Command::LPop { key })
}

pub(crate) fn invoke(store: &mut Store, key: &str) -> anyhow::Result<Resp> {
    match store.db.get_mut(key) {
        Some(entry) if !entry.is_expired() => match entry.value_mut() {
            RedisValue::List(values) => {
                let values: Vec<String> = values.drain(0..1).collect();
                let first_element = values[0].clone();

                Ok(Resp::bulk(first_element))
            }
            _ => anyhow::bail!("Invalid Operation! Can't LPOP on non-list type"),
        },
        _ => Ok(Resp::null()),
    }
}
