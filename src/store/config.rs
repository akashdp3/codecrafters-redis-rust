#[derive(Debug)]
pub(crate) struct Config {
    dir: String,
    db_file_name: String,
    replication: Replication,
}

#[derive(Debug, PartialEq)]
pub(crate) enum Role {
    Master,
    Replica { master_addr: String },
}

#[derive(Debug)]
pub(crate) struct Replication {
    pub(crate) role: Role,
    pub(crate) id: String,
    pub(crate) offset: u64,
}

impl Config {
    pub(crate) fn new(dir: &str, db_file_name: &str, replica_of: &str) -> Self {
        let role = if replica_of.is_empty() {
            Role::Master
        } else {
            Role::Replica {
                master_addr: replica_of
                    .split(" ")
                    .into_iter()
                    .collect::<Vec<_>>()
                    .join(":"),
            }
        };

        Self {
            dir: dir.to_string(),
            db_file_name: db_file_name.to_string(),
            replication: Replication {
                id: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".into(),
                role,
                offset: 0,
            },
        }
    }

    pub(crate) fn dir(&self) -> &str {
        self.dir.as_str()
    }

    pub(crate) fn db_file_name(&self) -> &str {
        self.db_file_name.as_str()
    }

    pub(crate) fn is_master(&self) -> bool {
        matches!(self.replication.role, Role::Master)
    }

    pub(crate) fn is_replica(&self) -> bool {
        matches!(self.replication.role, Role::Replica { .. })
    }

    pub(crate) fn master_addr(&self) -> &str {
        match &self.replication.role {
            Role::Master => {
                panic!("master_addr called on master role");
            }
            Role::Replica { master_addr } => master_addr.as_str(),
        }
    }

    pub(crate) fn get_repl_info(&self) -> &Replication {
        &self.replication
    }
}
