use crate::{
    utils::{next_arg, sha256_hex},
    Command, Resp, Store,
};

const CMD_NAME: &str = "ACL";

#[derive(Debug, Clone)]
pub(crate) enum SubCmd {
    WhoAmI,
    GetUser(String),
    SetUser(String, String),
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let sub_cmd = next_arg(args, CMD_NAME, "sub_cmd")?;

    let sub_cmd = match sub_cmd.to_lowercase().as_str() {
        "whoami" => SubCmd::WhoAmI,
        "getuser" => {
            let name = next_arg(args, CMD_NAME, "username")?;
            SubCmd::GetUser(name)
        }
        "setuser" => {
            let username = next_arg(args, CMD_NAME, "username")?;
            let value = next_arg(args, CMD_NAME, "value")?;

            SubCmd::SetUser(username, value)
        }
        _ => anyhow::bail!("Invalid sub command for ACL"),
    };

    Ok(Command::Acl { sub_cmd })
}

pub(crate) fn invoke(store: &mut Store, sub_cmd: SubCmd) -> anyhow::Result<Resp> {
    let result = match sub_cmd {
        // WHOAMI needs connection state and is handled by execute_for_client.
        SubCmd::WhoAmI => Resp::error("NOAUTH Authentication required."),
        SubCmd::GetUser(name) => {
            let user = store.users.entry(name).or_default();
            let flags = user.flags().iter().map(Resp::bulk).collect();
            let passwords = user.passwords().iter().map(Resp::bulk).collect();

            Resp::Array(vec![
                Resp::bulk("flags"),
                Resp::Array(flags),
                Resp::bulk("passwords"),
                Resp::Array(passwords),
            ])
        }
        SubCmd::SetUser(username, value) => {
            let user = store.users.entry(username).or_default();

            if let Some(password) = value.strip_prefix(">") {
                user.add_password(sha256_hex(password));
                user.remove_flag("nopass".to_string());
            }

            Resp::ok()
        }
    };

    Ok(result)
}
