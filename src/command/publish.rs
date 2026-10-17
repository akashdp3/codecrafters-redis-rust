use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "PUBLISH";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let channel = next_arg(args, CMD_NAME, "channel")?;
    let msg = next_arg(args, CMD_NAME, "msg")?;

    Ok(Command::Publish { channel, msg })
}

pub(crate) async fn invoke(
    store: &mut Store,
    channel: String,
    msg: String,
) -> anyhow::Result<Resp> {
    let sub_len = store.publish(channel, msg).await;
    Ok(Resp::integer(sub_len))
}
