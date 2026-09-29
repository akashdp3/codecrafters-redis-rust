use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "TYPE";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;

    Ok(Command::Type { key })
}

pub(crate) fn invoke(store: &mut Store, key: &str) -> anyhow::Result<Resp> {
    Ok(Resp::SimpleString(store.db.get_type(key).to_string()))
}
