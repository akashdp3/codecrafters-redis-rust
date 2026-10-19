use crate::{
    store::RedisValue,
    utils::{self, next_arg},
    Command, Resp, Store,
};

const CMD_NAME: &str = "GEODIST";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let member_1 = next_arg(args, CMD_NAME, "member_1")?;
    let member_2 = next_arg(args, CMD_NAME, "member_2")?;

    Ok(Command::GeoDistance {
        key,
        member_1,
        member_2,
    })
}

pub(crate) fn invoke(
    store: &mut Store,
    key: String,
    member_1: String,
    member_2: String,
) -> anyhow::Result<Resp> {
    let cor_1 = get_cordinate(store, key.clone(), member_1)?;
    let cor_2 = get_cordinate(store, key, member_2)?;

    let distance = utils::geo::calculate_distance(cor_1.0, cor_1.1, cor_2.0, cor_2.1);

    Ok(Resp::BulkString(Some(distance.to_string())))
}

fn get_cordinate(store: &Store, key: String, member: String) -> anyhow::Result<(f64, f64)> {
    match store.db.get(&key) {
        Some(value) => match value {
            RedisValue::SortedSet(items) => match items.iter().find(|(s, _)| *s == member) {
                Some((_, score)) => {
                    let coordinates = utils::geo::decode(*score as u64);

                    Ok((coordinates.longitude, coordinates.latitude))
                }
                None => anyhow::bail!("Invalid Operation"),
            },
            _ => anyhow::bail!("Invalid Operation"),
        },
        None => anyhow::bail!("Invalid Operation"),
    }
}
