mod config;
mod db;

use anyhow::Context;
use config::Config;
use db::Db;
use tokio::fs;

use crate::rdb_parser::RDBParser;

pub(crate) use db::IntoSystemTime;

#[derive(Debug)]
pub(crate) struct Store {
    pub(crate) config: Config,
    pub(crate) db: Db,
}

impl Store {
    pub(crate) async fn init(dir: &str, db_file_name: &str) -> anyhow::Result<Self> {
        let config = Config::set(dir, db_file_name);
        let loaded_data = match Self::load_data(&config).await {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to load data from .rdb file, Err: {}", err);
                Db::new()
            }
        };

        Ok(Self {
            config: Config::set(dir, db_file_name),
            db: loaded_data,
        })
    }

    async fn load_data(config: &Config) -> anyhow::Result<Db> {
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
}
