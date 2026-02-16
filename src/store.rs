mod config;
mod db;

use config::Config;
use db::Db;

pub(crate) struct Store {
    pub(crate) config: Config,
    pub(crate) db: Db,
}

impl Store {
    pub(crate) fn init(dir: &str, db_file_name: &str) -> Self {
        Self {
            config: Config::set(dir, db_file_name),
            db: Db::new(),
        }
    }
}
