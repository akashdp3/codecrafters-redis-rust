use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "LRANGE";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let start: isize = next_arg(args, CMD_NAME, "start")?.parse()?;
    let end: isize = next_arg(args, CMD_NAME, "start")?.parse()?;

    Ok(Command::LRange { key, start, end })
}

pub(crate) fn invoke(
    store: &mut Store,
    key: &str,
    start: isize,
    end: isize,
) -> anyhow::Result<Resp> {
    let values = match store.db.get(&key) {
        Some(RedisValue::List(elements)) => {
            let len = elements.len() as isize;

            let start = if start < 0 { len + start } else { start }.max(0);
            let end = if end < 0 { len + end } else { end }.min(len - 1);

            if start < end {
                elements[start as usize..=end as usize].to_vec()
            } else {
                vec![]
            }
        }
        _ => vec![],
    };

    Ok(Resp::array(values))
}
