mod config;
mod db;

use anyhow::Context;
use config::Config;
use db::Db;
use tokio::fs;

use crate::{rdb_parser::RDBParser, Conn};

pub(crate) use db::IntoSystemTime;

#[derive(Debug)]
pub(crate) struct ReplicaState {
    pub(crate) conn: Conn,
    pub(crate) ack_offset: usize,
}

#[derive(Debug)]
pub(crate) struct Store {
    pub(crate) config: Config,
    pub(crate) db: Db,
    pub(crate) replicas: Vec<ReplicaState>,
    pub(crate) master_repl_offset: usize,
    offset: usize,
}

impl Store {
    pub(crate) async fn init(
        dir: &str,
        db_file_name: &str,
        replica_of: &str,
    ) -> anyhow::Result<Self> {
        let config = Config::new(dir, db_file_name, replica_of);
        let db = match Self::load_data(&config).await {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to load data from .rdb file, Err: {}", err);
                Db::new()
            }
        };

        Ok(Self {
            config,
            db,
            replicas: vec![],
            master_repl_offset: 0,
            offset: 0,
        })
    }

    async fn load_data(config: &Config) -> anyhow::Result<Db> {
        // Check if dir and db_file_name are passed
        if config.dir().is_empty() || config.db_file_name().is_empty() {
            return Ok(Db::new());
        }

        let dir_path = fs::canonicalize(config.dir())
            .await
            .with_context(|| format!("Failed to canonicalize dir_path: {}", config.dir()))?;
        let rdb_file_path = fs::canonicalize(dir_path.join(config.db_file_name())).await?;

        let mut parser = RDBParser::new(rdb_file_path).await?;
        let rdb = parser.parse().await?;

        let mut db = Db::new();

        for (key, redis_value) in rdb.data.iter() {
            db.set(key, &redis_value.value, redis_value.expiry)?;
        }

        Ok(db)
    }

    pub(crate) fn add_replica(&mut self, conn: Conn) {
        self.replicas.push(ReplicaState {
            conn,
            ack_offset: 0,
        });
    }

    pub(crate) fn increment_offset(&mut self, frame_len: usize) {
        self.offset += frame_len;
    }

    pub(crate) fn get_offset(&mut self) -> usize {
        self.offset
    }
}
