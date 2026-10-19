use crate::{command::zadd, utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "GEOADD";

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
    zadd::invoke(store, key, 0.0, member)
}
