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
                let added = match val.iter().position(|(m, _)| *m == member) {
                    Some(i) => {
                        val.remove(i);
                        0
                    }
                    None => 1,
                };
                let pos = val
                    .binary_search_by(|(m, s)| s.total_cmp(&score).then_with(|| m.cmp(&member)))
                    .unwrap_or_else(|p| p);
                val.insert(pos, (member, score));

                Ok(Resp::integer(added))
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
