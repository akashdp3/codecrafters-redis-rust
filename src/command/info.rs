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
        InfoKind::Replication => {
            match store.config.replica_of() {
                "" => "role:master",
                _replica_info => "role:slave"
            }
        }
        InfoKind::All => "role:master"
    };

    Ok(Resp::bulk(value))
}
