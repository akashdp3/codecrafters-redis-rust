#[derive(Debug)]
pub(crate) struct Config {
    dir: String,
    db_file_name: String,
    replica_of: String
}

impl Config {
    pub(crate) fn new(dir: &str, db_file_name: &str, replica_of: &str) -> Self {
        Self {
            dir: dir.to_string(),
            db_file_name: db_file_name.to_string(),
            replica_of: replica_of.to_string()
        }
    }

    pub(crate) fn dir(&self) -> &str {
        self.dir.as_str()
    }

    pub(crate) fn db_file_name(&self) -> &str {
        self.db_file_name.as_str()
    }

    pub(crate) fn replica_of(&self) -> &str {
        self.replica_of.as_str()
    }
}
