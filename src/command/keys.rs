use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "KEYS";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let pattern = next_arg(args, CMD_NAME, "pattern")?;

    Ok(Command::Keys { pattern })
}

pub(crate) fn invoke(store: &mut Store, pattern: &str) -> anyhow::Result<Resp> {
    let matching_keys = store.db.keys(pattern);

    Ok(Resp::array(matching_keys))
}
