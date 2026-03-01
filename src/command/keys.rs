use anyhow::Context;

use crate::{Command, Resp, Store};

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let pattern = args.next().context("Missing argument 'pattern for KEYS command")?;

    Ok(Command::Keys { pattern })
}

pub(crate) fn invoke(store: &mut Store, pattern: &str) -> anyhow::Result<Resp> {
    let matching_keys = store.db.keys(pattern);

    Ok(Resp::array(matching_keys))
}
