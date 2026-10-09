use anyhow::Context;

use crate::{store::RedisValue, utils, Command, Resp, Store};

const CMD_NAME: &str = "XREAD";

#[derive(Debug)]
pub(crate) enum Entity {
    Streams,
    Block(u64),
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let entity = match utils::next_arg(args, CMD_NAME, "ENTITY")?
        .to_uppercase()
        .as_str()
    {
        "STREAMS" => Entity::Streams,
        "BLOCK" => {
            let timeout = utils::next_arg(args, CMD_NAME, "TIMEOUT")?
                .parse()
                .context("Failed to parse timeout")?;
            let _ = utils::next_arg(args, CMD_NAME, "STREAMS")?;

            Entity::Block(timeout)
        }
        _ => anyhow::bail!("Invalid entity for XREAD"),
    };

    let args: Vec<String> = args.collect();
    if args.is_empty() || args.len() % 2 != 0 {
        anyhow::bail!("Invalid arguments for XREAD");
    }

    let mid = args.len() / 2;
    let keys = args[..mid].to_vec();
    let ids = args[mid..].to_vec();

    Ok(Command::Xread { entity, keys, ids })
}

/// Replaces each `$` id with the id of the latest entry currently in that stream
/// (or `0-0` if the stream is empty/missing), so later reads only see newer entries.
pub(crate) fn resolve_ids(store: &Store, keys: &[String], ids: Vec<String>) -> Vec<String> {
    keys.iter()
        .zip(ids)
        .map(|(key, id)| {
            if id != "$" {
                return id;
            }
            match store.db.get(key) {
                Some(RedisValue::Stream(stream)) => stream
                    .last()
                    .map(|item| item.id().to_string())
                    .unwrap_or_else(|| "0-0".to_string()),
                _ => "0-0".to_string(),
            }
        })
        .collect()
}

fn stream(store: &Store, key: String, id: String) -> anyhow::Result<Resp> {
    let (start_ms, start_seq) = utils::parse_stream_id(&id)?;

    let entries = match store.db.get(&key) {
        Some(RedisValue::Stream(stream)) => stream
            .iter()
            .filter(|item| match utils::parse_stream_id(item.id()) {
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

    if entries.is_empty() {
        return Ok(Resp::null());
    }

    let stream_data = vec![Resp::bulk(key), Resp::Array(entries)];
    Ok(Resp::Array(stream_data))
}

pub(crate) fn invoke(store: &Store, keys: Vec<String>, ids: Vec<String>) -> anyhow::Result<Resp> {
    let mut streams = vec![];
    let ids = resolve_ids(store, &keys, ids);

    for (key, id) in keys.into_iter().zip(ids.into_iter()) {
        let stream_data = stream(store, key, id)?;
        if !stream_data.is_null() {
            streams.push(stream_data);
        }
    }

    Ok(Resp::Array(streams))
}
