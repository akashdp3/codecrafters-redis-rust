use crate::{Command, Resp, Store};
use anyhow::Context;

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = args
        .next()
        .context("Missing argument 'key' for TYPE command")?;

    Ok(Command::Type { key })
}

pub(crate) async fn invoke(store: &mut Store, key: &str) -> anyhow::Result<Resp> {
    Ok(Resp::SimpleString(store.db.get_type(key).to_string()))
}
