use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "WATCH";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;

    Ok(Command::Watch { key })
}

pub(crate) fn invoke(_store: &mut Store, _key: String) -> anyhow::Result<Resp> {
    Ok(Resp::ok())
}
