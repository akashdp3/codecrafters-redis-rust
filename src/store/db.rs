use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

use glob::Pattern;

#[derive(Debug)]
pub(crate) enum Value {
    String(String),
    Stream(String, Vec<(String, String)>),
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<(String, Vec<(String, String)>)> for Value {
    fn from((id, value): (String, Vec<(String, String)>)) -> Self {
        Value::Stream(id, value)
    }
}

#[derive(Debug)]
struct Entry {
    value: Value,
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
    data: HashMap<String, Entry>,
}

impl Db {
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub(crate) fn set<V: Into<Value>>(
        &mut self,
        key: String,
        value: V,
        expiry: Option<SystemTime>,
    ) -> anyhow::Result<()> {
        let value = value.into();
        self.data.insert(key, Entry { value, expiry });

        Ok(())
    }

    pub(crate) fn get(&self, key: &str) -> Option<&String> {
        let value = self.data.get(key).and_then(|item| match &item.expiry {
            Some(exp) if exp < &SystemTime::now() => None,
            _ => Some(&item.value),
        });

        match value {
            Some(Value::String(val)) => Some(val),
            _ => None,
        }
    }

    pub(crate) fn get_type(&self, key: &str) -> &str {
        let value = self.data.get(key).and_then(|item| match &item.expiry {
            Some(exp) if exp < &SystemTime::now() => None,
            _ => Some(&item.value),
        });

        match value {
            Some(Value::String(..)) => "string",
            Some(Value::Stream(..)) => "stream",
            None => "none",
        }
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
