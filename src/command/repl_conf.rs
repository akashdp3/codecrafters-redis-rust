use std::str::FromStr;

use crate::{Command, Resp, Store};
use anyhow::Context;

pub(crate) enum Kind {
    BindingPort,
    Capabilities,
}

impl FromStr for Kind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binding-port" => Ok(Kind::BindingPort),
            "capa" => Ok(Kind::Capabilities),
            _ => Err(anyhow::anyhow!("Invalid REPLCONF key: {}", s)),
        }
    }
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key: Kind = args
        .next()
        .context("Failed to parse 'KEY' for REPLCONF")?
        .parse()?;
    let value = args
        .next()
        .context("Failed to parse 'VALUE' for REPLCONF")?;

    Ok(Command::ReplConf { key, value })
}

pub(crate) fn invoke(_store: &mut Store, _key: Kind, _value: &str) -> anyhow::Result<Resp> {
    Ok(Resp::ok())
}
