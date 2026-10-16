use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "SUBSCRIBE";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let channel = next_arg(args, CMD_NAME, "channel")?;

    Ok(Command::Subscribe { channel })
}

pub(crate) fn invoke(_store: &mut Store, channel: String) -> anyhow::Result<Resp> {
    Ok(Resp::Array(vec![
        Resp::bulk("subscribe"),
        Resp::bulk(channel),
        Resp::integer(1),
    ]))
}
