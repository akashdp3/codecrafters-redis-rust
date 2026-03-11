use std::str::FromStr;

use crate::{Command, Resp, Store};
use anyhow::Context;

pub(crate) enum Kind {
    ListeningPort,
    Capabilities,
    GetAck,
    Ack,
}

impl FromStr for Kind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "listening-port" => Ok(Kind::ListeningPort),
            "capa" => Ok(Kind::Capabilities),
            "getack" => Ok(Kind::GetAck),
            "ack" => Ok(Kind::Ack),
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

pub(crate) fn invoke(_store: &mut Store, key: Kind, value: &str) -> anyhow::Result<Resp> {
    let result = match key {
        Kind::GetAck => Resp::array(vec![
            "REPLCONF".to_string(),
            "ACK".to_string(),
            "0".to_string(),
        ]),
        _ => Resp::ok(),
    };

    Ok(result)
}
