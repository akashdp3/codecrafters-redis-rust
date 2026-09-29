use std::str::FromStr;

use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "CONFIG";

/**
 * Enum Operation
 */
#[derive(Debug)]
pub(crate) enum Op {
    Get,
    Set,
}

impl FromStr for Op {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "get" => Ok(Op::Get),
            "set" => Ok(Op::Set),
            _ => anyhow::bail!(format!("Invalid argument '{}' for CONFIG command", s)),
        }
    }
}

/**
 * Enum Name
 */
#[derive(Debug)]
pub(crate) enum Name {
    Dir,
    DbFileName,
}

impl FromStr for Name {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dir" => Ok(Name::Dir),
            "dbfilename" => Ok(Name::DbFileName),
            _ => anyhow::bail!(format!("Invalid argument '{}' for CONFIG command", s)),
        }
    }
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let op = next_arg(args, CMD_NAME, "GET")?.parse()?;
    let name = next_arg(args, CMD_NAME, "name")?.parse()?;

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
