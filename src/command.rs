use crate::{resp::Resp, store::Store};
use anyhow::{Context, Ok};
use std::time::Duration;

mod config;
mod info;
mod keys;
mod set;

pub(crate) enum Command {
    Ping,
    Echo {
        name: String,
    },
    Get {
        key: String,
    },
    Set {
        key: String,
        value: String,
        expiry: Option<Duration>,
    },
    Config {
        op: config::Op,
        name: config::Name,
    },
    Keys {
        pattern: String,
    },
    Info {
        kind: info::InfoKind,
    },
}

impl Command {
    pub(crate) fn parse(args: Vec<String>) -> anyhow::Result<Command> {
        let mut args = args.into_iter();
        let command = args.next().context("Invalid command")?;

        match command.to_lowercase().as_str() {
            "ping" => Ok(Command::Ping),
            "echo" => {
                let name = args
                    .next()
                    .context("Missing argument 'name' for ECHO command")?;

                Ok(Command::Echo { name })
            }
            "get" => {
                let key = args
                    .next()
                    .context("Missing argument 'key' for GET command")?;

                Ok(Command::Get { key })
            }
            "set" => set::parse(&mut args),
            "config" => config::parse(&mut args),
            "keys" => keys::parse(&mut args),
            "info" => info::parse(&mut args),
            _ => anyhow::bail!("Unknown command encountered"),
        }
    }

    pub(crate) async fn execute(self, store: &mut Store) -> anyhow::Result<String> {
        let result: Resp = match self {
            Command::Ping => Resp::SimpleString("PONG".to_string()),
            Command::Echo { name } => Resp::bulk(name),
            Command::Get { key } => match store.db.get(&key) {
                Some(value) => Resp::BulkString(Some(value)),
                None => Resp::null(),
            },
            Command::Set { key, value, expiry } => set::invoke(store, &key, &value, expiry)?,
            Command::Config { op, name } => config::invoke(store, op, name)?,
            Command::Keys { pattern } => keys::invoke(store, &pattern)?,
            Command::Info { kind } => info::invoke(store, kind)?,
        };

        Ok(result.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping() {
        let args = vec!["PING".to_string()];
        let cmd = Command::parse(args).unwrap();
        assert!(matches!(cmd, Command::Ping));
    }

    #[test]
    fn test_parse_ping_lowercase() {
        let args = vec!["ping".to_string()];
        let cmd = Command::parse(args).unwrap();
        assert!(matches!(cmd, Command::Ping));
    }

    #[test]
    fn test_parse_echo() {
        let args = vec!["ECHO".to_string(), "hello".to_string()];
        let cmd = Command::parse(args).unwrap();
        assert!(matches!(cmd, Command::Echo { name } if name == "hello"));
    }

    #[test]
    fn test_parse_echo_missing_arg() {
        let args = vec!["ECHO".to_string()];
        let result = Command::parse(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_get() {
        let args = vec!["GET".to_string(), "mykey".to_string()];
        let cmd = Command::parse(args).unwrap();
        assert!(matches!(cmd, Command::Get { key } if key == "mykey"));
    }

    #[test]
    fn test_parse_set_without_expiry() {
        let args = vec!["SET".to_string(), "foo".to_string(), "bar".to_string()];
        let cmd = Command::parse(args).unwrap();
        assert!(matches!(cmd, Command::Set { key, value, expiry }
            if key == "foo" && value == "bar" && expiry.is_none()));
    }

    #[test]
    fn test_parse_set_with_px() {
        let args = vec![
            "SET".to_string(),
            "foo".to_string(),
            "bar".to_string(),
            "PX".to_string(),
            "1000".to_string(),
        ];
        let cmd = Command::parse(args).unwrap();
        assert!(matches!(cmd, Command::Set { expiry: Some(d), .. }
            if d == Duration::from_millis(1000)));
    }

    #[test]
    fn test_parse_set_with_ex() {
        let args = vec![
            "SET".to_string(),
            "foo".to_string(),
            "bar".to_string(),
            "EX".to_string(),
            "10".to_string(),
        ];
        let cmd = Command::parse(args).unwrap();
        assert!(matches!(cmd, Command::Set { expiry: Some(d), .. }
            if d == Duration::from_secs(10)));
    }

    #[test]
    fn test_parse_unknown_command() {
        let args = vec!["UNKNOWN".to_string()];
        let result = Command::parse(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_args() {
        let args: Vec<String> = vec![];
        let result = Command::parse(args);
        assert!(result.is_err());
    }
}
