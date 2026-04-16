use crate::{Command, Resp, Store};
use anyhow::Context;

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let repl_id = args.next().context("repl_id not provided")?;
    let offset = args.next().context("offset not provided")?;

    Ok(Command::Psync { repl_id, offset })
}

// Empty RDB file contents (hex-decoded)
const EMPTY_RDB_HEX: &str = "524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfe0d093a76";

pub(crate) fn invoke(store: &mut Store, repl_id: &str, offset: &str) -> anyhow::Result<Vec<u8>> {
    let repl_info = store.config.get_repl_info();

    // Temperory values
    assert_eq!(repl_id, "?");
    assert_eq!(offset, "-1");

    let fullresync = format!("FULLRESYNC {} {}", repl_info.id, repl_info.offset);
    let fullresync_resp = Resp::SimpleString(fullresync).encode();

    let rdb_contents = hex::decode(EMPTY_RDB_HEX)?;
    let rdb_header = format!("${}\r\n", rdb_contents.len());

    let mut result = fullresync_resp.into_bytes();
    result.extend_from_slice(rdb_header.as_bytes());
    result.extend_from_slice(&rdb_contents);

    Ok(result)
}
