use std::time::{SystemTime, UNIX_EPOCH};

use crate::{store::RedisValue, Command, Resp, Store};
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
    // Get latest sequence_number
    let (latest_ms_time, latest_seq_num) = {
        let value = store.db.get(&key);

        match value {
            Some(RedisValue::Stream(values)) => {
                let last_value = values.last();

                match last_value {
                    Some(val) => {
                        let (ms_time, seq_num) = parse_stream_id(&val.id)?;

                        (ms_time, seq_num)
                    }
                    None => (0, 0),
                }
            }
            Some(RedisValue::String(..)) => {
                anyhow::bail!("Invalid Operation: Appending stream data to string type")
            }
            None => (0, 0),
        }
    };

    // id validation
    let id = {
        let (ms_time, seq_num) = get_stream_id(&id, latest_ms_time, latest_seq_num)?;

        if ms_time == 0 && seq_num == 0 {
            anyhow::bail!("The ID specified in XADD must be greater than 0-0")
        }

        if (latest_ms_time == ms_time && latest_seq_num >= seq_num) || latest_ms_time > ms_time {
            anyhow::bail!(
                "The ID specified in XADD is equal or smaller than the target stream top item"
            )
        }

        format!("{}-{}", ms_time, seq_num)
    };

    store.db.append_stream(key, id.clone(), fields)?;
    Ok(Resp::BulkString(Some(id)))
}

fn parse_stream_id(id: &str) -> anyhow::Result<(u128, u128)> {
    let (ms_time, seq_num) = match id.split_once("-") {
            Some((m, s)) => (m, s),
            _ => anyhow::bail!("Invalid stream id format. It should be in format '<millisecond_time>-<sequence_number>'"),
        };

    let ms_time: u128 = ms_time.parse()?;
    let seq_num: u128 = seq_num.parse()?;

    Ok((ms_time, seq_num))
}

fn get_stream_id(
    id: &str,
    latest_ms_time: u128,
    latest_seq_num: u128,
) -> anyhow::Result<(u128, u128)> {
    if id == "*" {
        let current_time = SystemTime::now();
        let current_time = current_time
            .duration_since(UNIX_EPOCH)
            .context("Failed to find current time")?
            .as_millis();
        return Ok((current_time, 0));
    }
    let (ms_time, seq_num) = match id.split_once("-") {
        Some((m, s)) => (m, s),
        _ => anyhow::bail!("Invalid stream id format. It should be in format '<millisecond_time>-<sequence_number>'"),
    };

    let ms_time: u128 = ms_time.parse()?;
    if seq_num == "*" {
        if ms_time == latest_ms_time as u128 {
            return Ok((ms_time, latest_seq_num + 1));
        } else {
            return Ok((ms_time, 0));
        }
    }

    let seq_num: u128 = seq_num.parse()?;

    Ok((ms_time, seq_num))
}
