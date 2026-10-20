use crate::{
    utils::{next_arg, sha256_hex},
    Command, Resp, Store,
};

const CMD_NAME: &str = "AUTH";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let username = next_arg(args, CMD_NAME, "username")?;
    let password = next_arg(args, CMD_NAME, "password")?;

    Ok(Command::Auth { username, password })
}

pub(crate) fn invoke(
    store: &mut Store,
    username: String,
    password: String,
) -> anyhow::Result<Resp> {
    let user = store.users.entry(username).or_default();
    let password = sha256_hex(&password);

    if user.passwords().contains(&password) {
        Ok(Resp::ok())
    } else {
        Ok(Resp::error(
            "WRONGPASS invalid username-password pair or user is disabled.",
        ))
    }
}
