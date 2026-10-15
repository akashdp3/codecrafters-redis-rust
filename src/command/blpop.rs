use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "BLPOP";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let timeout = next_arg(args, CMD_NAME, "timeout")?.parse()?;

    Ok(Command::BLPop { key, timeout })
}

pub(crate) fn invoke(store: &mut Store, key: String) -> anyhow::Result<Resp> {
    let poped_element = super::lpop::invoke(store, &key, 1)?;

    Ok(Resp::Array(vec![
        Resp::BulkString(Some(key)),
        poped_element,
    ]))
}
