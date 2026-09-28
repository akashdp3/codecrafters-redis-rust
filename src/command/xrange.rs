use crate::{store::RedisValue, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "XRANGE";

pub(super) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let start_id = next_arg(args, CMD_NAME, "start_id")?;
    let end_id = next_arg(args, CMD_NAME, "end_id")?;

    Ok(Command::Xrange {
        key,
        start_id,
        end_id,
    })
}

pub(super) fn invoke(
    store: &Store,
    key: String,
    start_id: String,
    end_id: String,
) -> anyhow::Result<Resp> {
    let mut result = vec![];
    let redis_stream = match store.db.get(&key) {
        Some(RedisValue::Stream(stream)) => stream,
        Some(RedisValue::String(..)) => anyhow::bail!("Invalid Operation!!!"),
        None => anyhow::bail!("Stream not found"),
    };

    let start_id = parse_stream_id(&start_id, true);
    let end_id = parse_stream_id(&end_id, false);

    for item in redis_stream {
        let stream_id = parse_stream_id(&item.id(), false);

        if start_id <= stream_id && stream_id <= end_id {
            result.push(item);
        }
    }

    let result = result
        .iter()
        .map(|item| {
            let id = Resp::bulk(item.id());
            let mut fields = vec![];

            item.fields().iter().for_each(|(key, value)| {
                fields.push(Resp::bulk(key));
                fields.push(Resp::bulk(value));
            });

            Resp::Array(vec![id, Resp::Array(fields)])
        })
        .collect();

    Ok(Resp::Array(result))
}

fn parse_stream_id(stream_id: &str, is_start: bool) -> (u128, u128) {
    if stream_id == "-" {
        return (0, 0);
    } else if stream_id == "+" {
        return (u128::MAX, u128::MAX);
    }

    if let Some((ms, seq)) = stream_id.split_once("-") {
        return (ms.parse().unwrap(), seq.parse().unwrap());
    }

    if is_start {
        (stream_id.parse().unwrap(), 0)
    } else {
        (stream_id.parse().unwrap(), u128::MAX)
    }
}
