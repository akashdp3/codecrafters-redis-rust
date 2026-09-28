use std::str::FromStr;

use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "REPLCONF";

#[derive(Debug)]
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
    let key: Kind = next_arg(args, CMD_NAME, "KEY")?.parse()?;
    let value = next_arg(args, CMD_NAME, "VALUE")?;

    Ok(Command::ReplConf { key, value })
}

pub(crate) fn invoke(store: &mut Store, key: Kind, _value: &str) -> anyhow::Result<Resp> {
    let result = match key {
        Kind::GetAck => Resp::array(vec![
            "REPLCONF".to_string(),
            "ACK".to_string(),
            store.get_offset().to_string(),
        ]),
        _ => Resp::ok(),
    };

    Ok(result)
}
