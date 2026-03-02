use crate::{Command, Resp, Store};
use anyhow::Context;

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let repl_id = args.next().context("repl_id not provided")?;
    let offset = args.next().context("offset not provided")?;

    Ok(Command::PSYNC { repl_id, offset })
}

pub(crate) fn invoke(store: &mut Store, repl_id: &str, offset: &str) -> anyhow::Result<Resp> {
    let repl_info = store.config.get_repl_info();

    // Temperory values
    assert_eq!(repl_id, "?");
    assert_eq!(offset, "-1");

    let result = format!("FULLRESYNC {} {}\r\n", repl_info.id, repl_info.offset);

    Ok(Resp::SimpleString(result))
}
