use crate::{
    utils::{next_arg, sha256_hex},
    Command, Resp, Store,
};

const CMD_NAME: &str = "AUTH";

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let first = next_arg(args, CMD_NAME, "password")?;
    let (username, password) = match args.next() {
        Some(password) => (first, password),
        None => ("default".to_string(), first),
    };
    anyhow::ensure!(
        args.next().is_none(),
        "ERR wrong number of arguments for 'auth' command"
    );

    Ok(Command::Auth { username, password })
}

pub(crate) fn invoke(
    store: &mut Store,
    username: String,
    password: String,
) -> anyhow::Result<Resp> {
    let password = sha256_hex(&password);

    if store.users.get(&username).is_some_and(|user| {
        user.enabled()
            && (user.flags().iter().any(|flag| flag == "nopass")
                || user.passwords().contains(&password))
    }) {
        Ok(Resp::ok())
    } else {
        Ok(Resp::error(
            "WRONGPASS invalid username-password pair or user is disabled.",
        ))
    }
}
