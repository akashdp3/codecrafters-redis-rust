use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use glob::Pattern;

#[derive(Debug)]
struct RedisValue {
    value: String,
    expiry: Option<SystemTime>,
}

pub(crate) trait IntoSystemTime {
    fn into_system_time(self) -> Option<SystemTime>;
}

impl IntoSystemTime for Option<Duration> {
    fn into_system_time(self) -> Option<SystemTime> {
        self.map(|duration| SystemTime::now() + duration)
    }
}

#[derive(Debug)]
pub(crate) struct Db {
    data: HashMap<String, RedisValue>,
}

impl Db {
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub(crate) fn set(
        &mut self,
        key: String,
        value: String,
        expiry: Option<SystemTime>,
    ) -> anyhow::Result<()> {
        self.data.insert(key, RedisValue { value, expiry });

        Ok(())
    }

    pub(crate) fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key).and_then(|item| match &item.expiry {
            Some(exp) if exp < &SystemTime::now() => None,
            _ => Some(&item.value),
        })
    }

    pub(crate) fn keys(&self, pattern: &str) -> Vec<String> {
        let ptn = match Pattern::new(pattern) {
            Ok(ptn) => ptn,
            Err(_) => return vec![],
        };

        self.data
            .keys()
            .filter(|s| ptn.matches(s.as_str()))
            .cloned()
            .collect()
    }
}
