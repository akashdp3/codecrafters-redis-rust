use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "RPUSH";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let elements = args.collect();

    Ok(Command::RPush { key, elements })
}

pub(crate) fn invoke(
    store: &mut Store,
    key: String,
    elements: Vec<String>,
    is_inverse: bool,
) -> anyhow::Result<Resp> {
    let elements = match store.db.get_mut(&key) {
        Some(entry) if entry.is_expired() => elements,
        Some(entry) => match entry.value_mut() {
            RedisValue::List(values) => {
                let mut elements = elements;

                if is_inverse {
                    elements.reverse();
                }
                elements.append(values);

                elements
            }
            _ => anyhow::bail!("Invalid Operation! Can't RPUSH into non-list type of value"),
        },
        None => elements,
    };
    let count = elements.len();
    let value = RedisValue::List(elements);

    super::set::invoke(store, key.clone(), value, None)?;
    if store.to_be_notified_items.contains(&key) {
        store.notify.notify_last();
    }

    Ok(Resp::integer(count))
}
