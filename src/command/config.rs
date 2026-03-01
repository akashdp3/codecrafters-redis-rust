use crate::{Command, Resp, Store};
use anyhow::Context;

pub(crate) enum Op {
    Get,
    Set,
}

pub(crate) enum Name {
    Dir,
    DbFileName,
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let op = args
        .next()
        .context("Missing argument 'GET' for CONFIG command")?;
    let name = args
        .next()
        .context("Missing argument 'name' for CONFIG command")?;

    let op = match op.as_str() {
        "GET" => Op::Get,
        "SET" => Op::Set,
        _ => anyhow::bail!(format!("Invalid arguemnt '{}' for CONFIG command", op)),
    };
    let name = match name.as_str() {
        "dir" => Name::Dir,
        "dbfilename" => Name::DbFileName,
        _ => anyhow::bail!(format!("Invalid argument '{}' in CONFIG command", name)),
    };

    Ok(Command::Config { op, name })
}

pub(crate) fn invoke(store: &Store, op: Op, name: Name) -> anyhow::Result<Resp> {
    let (key, val) = match name {
        Name::Dir => (String::from("dir"), store.config.dir().to_string()),
        Name::DbFileName => (
            String::from("dbfilename"),
            store.config.db_file_name().to_string(),
        ),
    };

    match op {
        Op::Get => Ok(Resp::array(vec![key, val])),
        _ => Ok(Resp::null()),
    }
}
