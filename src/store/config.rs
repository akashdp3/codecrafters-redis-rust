pub(crate) struct Config {
    dir: String,
    db_file_name: String,
}

impl Config {
    pub(crate) fn set(dir: &str, db_file_name: &str) -> Self {
        Self {
            dir: dir.to_string(),
            db_file_name: db_file_name.to_string(),
        }
    }

    pub(crate) fn dir(&self) -> &str {
        self.dir.as_str()
    }

    pub(crate) fn db_file_name(&self) -> &str {
        &self.db_file_name.as_str()
    }
}
