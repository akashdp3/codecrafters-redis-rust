use crate::{
    store::RedisValue,
    utils::{self, next_arg},
    Command, Resp, Store,
};

const CMD_NAME: &str = "GEOPOS";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;

    Ok(Command::Geopos {
        key,
        members: args.collect(),
    })
}

pub(crate) fn invoke(store: &mut Store, key: String, members: Vec<String>) -> anyhow::Result<Resp> {
    let result = match store.db.get(&key) {
        Some(value) => match value {
            RedisValue::SortedSet(items) => {
                let mut result = vec![];
                for member in members {
                    let value = match items.iter().find(|(s, _)| *s == member) {
                        Some((_, score)) => {
                            let coordinates = utils::geo::decode(*score as u64);

                            Resp::array(vec![
                                coordinates.longitude.to_string(),
                                coordinates.latitude.to_string(),
                            ])
                        }
                        None => Resp::null_array(),
                    };

                    result.push(value);
                }

                Resp::Array(result)
            }
            _ => anyhow::bail!("Invalid Operation"),
        },
        None => Resp::Array(members.iter().map(|_| Resp::null_array()).collect()),
    };

    Ok(result)
}
