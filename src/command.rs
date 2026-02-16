use std::time::Duration;

use anyhow::{Context, Ok};

use crate::{resp::Resp, store::Store};

enum ExpiryUnit {
    PX,
    EX,
}

enum ConfigOp {
    GET,
}

enum ConfigName {
    Dir,
    DbFileName,
}

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
        op: ConfigOp,
        name: ConfigName,
    },
}

impl Command {
    pub(crate) fn parse(args: Vec<String>) -> anyhow::Result<Command> {
        let command = args.first().context("Invalid command")?;

        match command.to_lowercase().as_str() {
            "ping" => Ok(Command::Ping),
            "echo" => {
                let name = args
                    .get(1)
                    .context("Missing argument 'name' for ECHO command")?;

                Ok(Command::Echo { name: name.clone() })
            }
            "get" => {
                let key = args
                    .get(1)
                    .context("Missing argument 'key' for GET command")?;

                Ok(Command::Get { key: key.clone() })
            }
            "set" => {
                let key = args
                    .get(1)
                    .context("Missing argument 'key' for SET command")?;
                let value = args
                    .get(2)
                    .context("Missing argument 'key' for SET command")?;
                let unit = match args.get(3).map(|s| s.as_str()) {
                    Some("PX") => Some(ExpiryUnit::PX),
                    Some("EX") => Some(ExpiryUnit::EX),
                    _ => None,
                };
                let expiry = match args.get(4) {
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

                Ok(Command::Set {
                    key: key.clone(),
                    value: value.clone(),
                    expiry: expiry,
                })
            }
            "config" => {
                let op = args
                    .get(1)
                    .map(|s| s.as_str())
                    .context("Missing argument 'GET' for CONFIG command")?;
                let name = args
                    .get(2)
                    .map(|s| s.as_str())
                    .context("Missing argument 'name' for CONFIG command")?;

                let op = match op {
                    "GET" => ConfigOp::GET,
                    _ => anyhow::bail!(format!("Invalid arguemnt '{}' for CONFIG command", op)),
                };
                let name = match name {
                    "dir" => ConfigName::Dir,
                    "dbfilename" => ConfigName::DbFileName,
                    _ => anyhow::bail!(format!("Invalid argument '{}' in CONFIG command", name)),
                };

                Ok(Command::Config { op, name })
            }
            _ => anyhow::bail!("Unknown command encountered"),
        }
    }

    pub(crate) async fn execute(self, store: &mut Store) -> anyhow::Result<String> {
        let result: Resp = match self {
            Command::Ping => Resp::SimpleString("PONG".to_string()),
            Command::Echo { name } => Resp::BulkString(Some(name)),
            Command::Get { key } => match store.db.get(&key) {
                Some(value) => Resp::BulkString(Some(value)),
                None => Resp::null(),
            },
            Command::Set { key, value, expiry } => {
                store
                    .db
                    .set(&key, &value, expiry)
                    .with_context(|| format!("Failed to write data to store: {}", key))?;

                Resp::ok()
            }
            Command::Config { op, name } => {
                let result = match name {
                    ConfigName::Dir => store.config.dir(),
                    ConfigName::DbFileName => store.config.db_file_name(),
                }
                .to_string();

                Resp::BulkString(Some(result))
            }
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
