use crate::{command::zadd, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "GEOADD";
const MAX_LON: f32 = 180.0;
const MIN_LON: f32 = -180.0;
const MAX_LAT: f32 = 85.05112878;
const MIN_LAT: f32 = -85.05112878;

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = next_arg(args, CMD_NAME, "key")?;
    let lon = next_arg(args, CMD_NAME, "lon")?.parse()?;
    let lat = next_arg(args, CMD_NAME, "lat")?.parse()?;
    let member = next_arg(args, CMD_NAME, "member")?;

    Ok(Command::Geoadd {
        key,
        lon,
        lat,
        member,
    })
}

pub(crate) fn invoke(
    store: &mut Store,
    key: String,
    lon: f32,
    lat: f32,
    member: String,
) -> anyhow::Result<Resp> {
    if (lon <= MAX_LON && lon >= MIN_LON) && (lat <= MAX_LAT && lat >= MIN_LAT) {
        return zadd::invoke(store, key, 0.0, member);
    }

    let err_msg = format!("ERR invalid longitude,latitude pair {},{}", lon, lat);
    Ok(Resp::error(&err_msg))
}
