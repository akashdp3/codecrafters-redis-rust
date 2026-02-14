use std::{collections::HashMap};

pub(crate) struct Store {
    data: HashMap<String, String>
}

impl Store {
    pub(crate) fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub(crate) fn set(&mut self, key: &str, value: &str) -> anyhow::Result<()> {
        self.data.insert(key.to_string(), value.to_string());

        Ok(())
    }

    pub(crate) fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).map(|value| value.clone())
    }
}
