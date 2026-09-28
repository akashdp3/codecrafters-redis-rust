use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "XREAD";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let entity = next_arg(args, CMD_NAME, "ENTITY")?;
    let key = next_arg(args, CMD_NAME, "key")?;
    let id = next_arg(args, CMD_NAME, "id")?;

    Ok(Command::Xread { entity, key, id })
}

pub(crate) fn invoke(store: &Store, key: String, id: String) -> anyhow::Result<Resp> {
    let (start_ms, start_seq) = parse_stream_id(&id)?;

    let entries = match store.db.get(&key) {
        Some(RedisValue::Stream(stream)) => stream
            .iter()
            .filter(|item| match parse_stream_id(item.id()) {
                Ok(stream_id) => stream_id > (start_ms, start_seq),
                Err(_) => false,
            })
            .map(|item| {
                let mut fields = vec![];
                item.fields().iter().for_each(|(field, value)| {
                    fields.push(Resp::bulk(field));
                    fields.push(Resp::bulk(value));
                });

                Resp::Array(vec![Resp::bulk(item.id()), Resp::Array(fields)])
            })
            .collect(),
        Some(RedisValue::String(..)) => anyhow::bail!("Invalid Operation!!!"),
        None => vec![],
    };

    Ok(Resp::Array(vec![Resp::Array(vec![
        Resp::bulk(key),
        Resp::Array(entries),
    ])]))
}

fn parse_stream_id(id: &str) -> anyhow::Result<(u128, u128)> {
    let (ms_time, seq_num) = id.split_once('-').ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid stream id format. It should be in format '<millisecond_time>-<sequence_number>'"
        )
    })?;

    let ms_time: u128 = ms_time.parse()?;
    let seq_num: u128 = seq_num.parse()?;

    Ok((ms_time, seq_num))
}
