mod redis_entry;
mod redis_value;
mod stream_value;

use glob::Pattern;
use redis_entry::RedisEntry;
pub(crate) use redis_value::RedisValue;
use std::{collections::HashMap, time::SystemTime};
use stream_value::StreamValue;

#[derive(Debug)]
pub(crate) struct Db {
    data: HashMap<String, RedisEntry>,
}

impl Db {
    pub(crate) fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub(crate) fn get(&self, key: &str) -> Option<&RedisValue> {
        match self.data.get(key) {
            Some(entry) if !entry.is_expired() => Some(entry.value()),
            _ => None,
        }
    }

    pub(crate) fn get_type(&self, key: &str) -> &str {
        match self.get(key) {
            Some(val) => val.type_of(),
            None => "none",
        }
    }

    pub(crate) fn set(
        &mut self,
        key: String,
        val: String,
        expiry: Option<SystemTime>,
    ) -> anyhow::Result<()> {
        let entry = RedisEntry::new(RedisValue::String(val), expiry);
        self.data.insert(key, entry);

        Ok(())
    }

    pub(crate) fn append_stream(
        &mut self,
        key: String,
        id: String,
        fields: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        let stream_value = StreamValue::new(id, fields);
        let redis_entry = self.data.get_mut(&key);

        match redis_entry {
            Some(entry) if entry.is_expired() => {
                self.data.insert(
                    key,
                    RedisEntry::new(RedisValue::Stream(vec![stream_value]), None),
                );
            }
            Some(entry) => {
                match entry.value_mut() {
                    RedisValue::Stream(stream) => stream.push(stream_value),
                    _ => anyhow::bail!("Invalid Stream Operation"),
                };
            }
            None => {
                self.data.insert(
                    key,
                    RedisEntry::new(RedisValue::Stream(vec![stream_value]), None),
                );
            }
        };

        Ok(())
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
