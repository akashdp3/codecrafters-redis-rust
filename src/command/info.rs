use crate::{Command, Resp, Store};

#[derive(Debug)]
pub(crate) enum InfoKind {
    All,
    Replication,
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let section = args.next();

    let kind = match section {
        Some(kind) => match kind.as_str() {
            "replication" => InfoKind::Replication,
            _ => return Err(anyhow::anyhow!("Invalid section for INFO command")),
        },
        None => InfoKind::All,
    };

    Ok(Command::Info { kind })
}

pub(crate) fn invoke(store: &mut Store, kind: InfoKind) -> anyhow::Result<Resp> {
    let replication_info = match kind {
        InfoKind::Replication => store.config.get_repl_info(),
        // TODO: for InfoKind::All, need to parse all values
        InfoKind::All => store.config.get_repl_info(),
    };

    let mut result = vec![];
    result.push(format!(
        "role:{}",
        if store.config.is_master() {
            "master"
        } else {
            "slave"
        }
    ));
    result.push(format!("master_replid:{}", replication_info.id));
    result.push(format!("master_repl_offset:{}", replication_info.offset));

    Ok(Resp::bulk(result.join("\r\n")))
}
