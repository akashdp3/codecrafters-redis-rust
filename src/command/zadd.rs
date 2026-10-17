use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "ZADD";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let score = next_arg(args, CMD_NAME, "score")?.parse()?;
    let member = next_arg(args, CMD_NAME, "member")?;

    Ok(Command::Zadd { key, score, member })
}

pub(crate) fn invoke(
    store: &mut Store,
    key: String,
    score: f32,
    member: String,
) -> anyhow::Result<Resp> {
    match store.db.get_mut(&key) {
        Some(entry) => match entry.value_mut() {
            RedisValue::SortedSet(val) => {
                val.push((member, score));
                val.sort_by(|a, b| a.1.total_cmp(&b.1));

                Ok(Resp::integer(val.len()))
            }
            _ => anyhow::bail!("Invalid Operation!"),
        },
        None => {
            store
                .db
                .set(key, RedisValue::SortedSet(vec![(member, score)]), None)?;

            Ok(Resp::integer(1))
        }
    }
}
