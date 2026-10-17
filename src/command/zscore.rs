use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "ZSCORE";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let member = next_arg(args, CMD_NAME, "member")?;

    Ok(Command::Zscore { key, member })
}

pub(crate) fn invoke(store: &mut Store, key: String, member: String) -> anyhow::Result<Resp> {
    let result = match store.db.get(&key) {
        Some(value) => match value {
            RedisValue::SortedSet(items) => match items.iter().find(|(s, _)| *s == member) {
                Some((_, score)) => Resp::bulk(score.to_string()),
                None => Resp::null(),
            },
            _ => anyhow::bail!("Invalid Operation"),
        },
        None => Resp::null(),
    };

    Ok(result)
}
