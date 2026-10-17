use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "ZRANK";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let member = next_arg(args, CMD_NAME, "member")?;

    Ok(Command::Zrank { key, member })
}

pub(crate) fn invoke(store: &mut Store, key: String, member: String) -> anyhow::Result<Resp> {
    let score: isize = match store.db.get(&key) {
        Some(entry) => match entry {
            RedisValue::SortedSet(val) => match val.iter().position(|(m, _)| *m == member) {
                Some(v) => v as isize,
                None => -1,
            },
            _ => anyhow::bail!("Invalid Operation!"),
        },
        None => -1,
    };

    if score >= 0 {
        Ok(Resp::integer(score as usize))
    } else {
        Ok(Resp::null())
    }
}
