use crate::{Command, Resp, Store};
use anyhow::Context;

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let numreplicas: u8 = args
        .next()
        .context("Missing argument 'numreplicas' for WAIT command")?
        .parse()?;
    let timeout: u16 = args
        .next()
        .context("Missing argument 'timeout' for WAIT command")?
        .parse()?;

    Ok(Command::Wait {
        numreplicas,
        timeout,
    })
}

pub(crate) fn invoke(store: &mut Store, _numreplicas: u8, _timeout: u16) -> anyhow::Result<Resp> {
    let replica_count = store.replicas.len();
    Ok(Resp::integer(replica_count))
}
