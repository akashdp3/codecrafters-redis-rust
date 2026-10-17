use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "ZREM";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let member = next_arg(args, CMD_NAME, "member")?;

    Ok(Command::Zrem { key, member })
}

pub(crate) fn invoke(store: &mut Store, key: String, member: String) -> anyhow::Result<Resp> {
    let result = match store.db.get_mut(&key) {
        Some(entry) => match entry.value_mut() {
            RedisValue::SortedSet(items) => {
                let before_len = items.len();
                items.retain(|(m, _)| *m != member);
                let after_len = items.len();

                before_len - after_len
            }
            _ => anyhow::bail!("Invalid Operation"),
        },
        None => 0,
    };

    Ok(Resp::integer(result))
}
