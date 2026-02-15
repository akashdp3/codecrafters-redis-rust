use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

struct RedisValue {
    value: String,
    expiry: Option<SystemTime>,
}

pub(crate) struct Store {
    data: HashMap<String, RedisValue>,
}

impl Store {
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub(crate) fn set(
        &mut self,
        key: &str,
        value: &str,
        expiry: Option<Duration>,
    ) -> anyhow::Result<()> {
        let expiry = match expiry {
            Some(duration) => SystemTime::now().checked_add(duration),
            _ => None,
        };

        self.data.insert(
            key.to_string(),
            RedisValue {
                value: value.to_string(),
                expiry: expiry,
            },
        );

        Ok(())
    }

    pub(crate) fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).and_then(|item| match &item.expiry {
            Some(exp) if exp < &SystemTime::now() => None,
            _ => Some(item.value.clone()),
        })
    }
}
