use crate::{utils::next_arg, Command, Resp, Store};

const CMD_NAME: &str = "ACL";

#[derive(Debug, Clone)]
pub(crate) enum SubCmd {
    WhoAmI,
}

impl TryFrom<String> for SubCmd {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "whoami" => Ok(SubCmd::WhoAmI),
            _ => Err("Invalid sub command".to_string()),
        }
    }
}

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let sub_cmd = next_arg(args, CMD_NAME, "sub_cmd")?;
    let sub_cmd = match SubCmd::try_from(sub_cmd) {
        Ok(sub_cmd) => sub_cmd,
        Err(_) => {
            anyhow::bail!("Invalid sub command for ACL");
        }
    };

    Ok(Command::Acl { sub_cmd })
}

pub(crate) fn invoke(_store: &mut Store, sub_cmd: SubCmd) -> anyhow::Result<Resp> {
    match sub_cmd {
        SubCmd::WhoAmI => Ok(Resp::bulk("default")),
    }
}
