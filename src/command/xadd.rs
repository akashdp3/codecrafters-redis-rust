use crate::{Command, Resp, Store};
use anyhow::Context;

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = args
        .next()
        .context("Missing argument 'key' for XADD command")?;
    let id = args
        .next()
        .context("Missing argument 'id' for XADD command")?;
    let mut fields = vec![];

    while let Some(field) = args.next() {
        let value = args
            .next()
            .context("Missing value for 'field' for XADD command")?;

        fields.push((field, value));
    }

    Ok(Command::Xadd { key, id, fields })
}

pub(crate) fn invoke(
    store: &mut Store,
    key: String,
    id: String,
    fields: Vec<(String, String)>,
) -> anyhow::Result<Resp> {
    store.db.set(key, (id.clone(), fields), None)?;

    Ok(Resp::BulkString(Some(id)))
}
