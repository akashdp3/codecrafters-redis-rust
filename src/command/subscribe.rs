use crate::{Command, Resp, Store};

const _CMD_NAME: &str = "SUBSCRIBE";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    Ok(Command::Subscribe {
        channels: args.collect(),
    })
}

pub(crate) fn _invoke(_store: &mut Store, channel: String) -> anyhow::Result<Resp> {
    Ok(Resp::Array(vec![
        Resp::bulk("subscribe"),
        Resp::bulk(channel),
        Resp::integer(1),
    ]))
}
