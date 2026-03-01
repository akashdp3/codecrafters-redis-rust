use crate::{Command, Resp, Store};

pub(crate) enum InfoKind {
    All,
    Replication,
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let section = args.next();

    let kind = match section {
        Some(kind) => {
            match kind.as_str() {
                "replication" => InfoKind::Replication,
                _ => return Err(anyhow::anyhow!("Invalid section for INFO command")),
            }
        }
        None => InfoKind::All,
    };

    Ok(Command::Info { kind })
}

pub(crate) fn invoke(store: &mut Store, kind: InfoKind) -> anyhow::Result<Resp> {
    let value = match kind {
        InfoKind::Replication => replication_info(store.config.replica_of()),
        // TODO: for InfoKind::All, need to parse all values
        InfoKind::All => replication_info(store.config.replica_of())
    };

    Ok(Resp::bulk(value))
}

fn replication_info(replica_info: &str) -> String {
    let mut result = vec![];

    let role = match replica_info {
        "" => "master",
        _info => "slave"
    };
    result.push(format!("role:{role}"));

    let master_replid = "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb";
    result.push(format!("master_replid:{master_replid}"));

    let master_repl_offset = 0;
    result.push(format!("master_repl_offset:{master_repl_offset}"));

    result.join("\r\n")
}
