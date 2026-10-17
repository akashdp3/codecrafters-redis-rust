use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "ZRANGE";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let start = next_arg(args, CMD_NAME, "start")?.parse()?;
    let end = next_arg(args, CMD_NAME, "end")?.parse()?;

    Ok(Command::Zrange { key, start, end })
}

pub(crate) fn invoke(
    store: &mut Store,
    key: String,
    start: isize,
    end: isize,
) -> anyhow::Result<Resp> {
    let values = match store.db.get(&key) {
        Some(val) => match val {
            RedisValue::SortedSet(elements) => {
                let len = elements.len() as isize;

                let start = if start < 0 { len + start } else { start }.max(0);
                let end = if end < 0 { len + end } else { end }.min(len - 1);

                if start <= end {
                    elements[start as usize..=end as usize]
                        .iter()
                        .map(|s| s.0.clone())
                        .collect()
                } else {
                    vec![]
                }
            }
            _ => anyhow::bail!("Invalid Operation!"),
        },
        None => vec![],
    };

    Ok(Resp::array(values))
}
