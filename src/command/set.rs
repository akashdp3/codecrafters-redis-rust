use crate::{store::IntoSystemTime, Command, Resp, Store};
use anyhow::Context;
use std::time::Duration;

enum ExpiryUnit {
    PX,
    EX,
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let key = args
        .next()
        .context("Missing argument 'key' for SET command")?;
    let value = args
        .next()
        .context("Missing argument 'value' for SET command")?;
    let unit = match args.next() {
        Some(unit) => match unit.as_str() {
            "PX" => Some(ExpiryUnit::PX),
            "EX" => Some(ExpiryUnit::EX),
            _ => None,
        },
        _ => None,
    };
    let expiry = match args.next() {
        Some(expiry_time) => {
            let expiry_time = expiry_time
                .parse::<u64>()
                .context("Failed to parse expiry")?;

            match unit {
                Some(ExpiryUnit::PX) => Some(Duration::from_millis(expiry_time)),
                Some(ExpiryUnit::EX) => Some(Duration::from_secs(expiry_time)),
                _ => None,
            }
        }
        None => None,
    };

    Ok(Command::Set { key, value, expiry })
}

pub(crate) fn invoke(
    store: &mut Store,
    key: &str,
    value: &str,
    expiry: Option<Duration>,
) -> anyhow::Result<Resp> {
    let expiry = expiry.into_system_time();
    store
        .db
        .set(key, value, expiry)
        .with_context(|| format!("Failed to write data to store: {}", key))?;

    Ok(Resp::ok())
}
