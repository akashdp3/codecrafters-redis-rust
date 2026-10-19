use crate::{
    store::RedisValue,
    utils::{self, next_arg},
    Command, Resp, Store,
};

const CMD_NAME: &str = "GEOSEARCH";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let option_1 = next_arg(args, CMD_NAME, "key")?;
    let lon = next_arg(args, CMD_NAME, "lon")?.parse()?;
    let lat = next_arg(args, CMD_NAME, "lat")?.parse()?;
    let option_2 = next_arg(args, CMD_NAME, "option_2")?;
    let radius = next_arg(args, CMD_NAME, "radius")?.parse()?;
    let unit = next_arg(args, CMD_NAME, "unit")?;

    Ok(Command::GeoSearch {
        key,
        option_1,
        lon,
        lat,
        option_2,
        radius,
        unit,
    })
}

pub(crate) fn invoke(
    store: &mut Store,
    key: String,
    _option_1: String,
    lon: f64,
    lat: f64,
    _option_2: String,
    radius: f64,
    _unit: String,
) -> anyhow::Result<Resp> {
    let result = match store.db.get(&key) {
        Some(value) => {
            let mut results: Vec<String> = vec![];
            match value {
                RedisValue::SortedSet(items) => {
                    for (member, score) in items.iter() {
                        let coordidate = utils::geo::decode(*score as u64);

                        let distance = utils::geo::calculate_distance(
                            lon,
                            lat,
                            coordidate.longitude,
                            coordidate.latitude,
                        );

                        if radius >= distance {
                            results.push(member.clone());
                        }
                    }
                }
                _ => anyhow::bail!("Invalid Operation"),
            };

            results
        }
        None => anyhow::bail!("Invalid Operation"),
    };

    Ok(Resp::array(result))
}
