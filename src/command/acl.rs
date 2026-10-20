use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "ACL";

#[derive(Debug, Clone)]
pub(crate) enum SubCmd {
    WhoAmI,
    GetUser(String),
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let sub_cmd = next_arg(args, CMD_NAME, "sub_cmd")?;

    let sub_cmd = match sub_cmd.to_lowercase().as_str() {
        "whoami" => SubCmd::WhoAmI,

        "getuser" => {
            let name = next_arg(args, CMD_NAME, "username")?;
            SubCmd::GetUser(name)
        }

        _ => anyhow::bail!("Invalid sub command for ACL"),
    };

    Ok(Command::Acl { sub_cmd })
}

pub(crate) fn invoke(_store: &mut Store, sub_cmd: SubCmd) -> anyhow::Result<Resp> {
    match sub_cmd {
        SubCmd::WhoAmI => Ok(Resp::bulk("default")),
        SubCmd::GetUser(_) => Ok(Resp::Array(vec![Resp::bulk("flags"), Resp::Array(vec![])])),
    }
}
