use super::RedisValue;
use std::time::SystemTime;

/*
* Redis Entry
*/
#[derive(Debug)]
pub(crate) struct RedisEntry {
    value: RedisValue,
    expiry: Option<SystemTime>,
}

impl RedisEntry {
    pub(super) fn new(value: RedisValue, expiry: Option<SystemTime>) -> Self {
        Self { value, expiry }
    }

    pub(super) fn value(&self) -> &RedisValue {
        &self.value
    }

    pub(crate) fn value_mut(&mut self) -> &mut RedisValue {
        &mut self.value
    }

    pub(crate) fn is_expired(&self) -> bool {
        match self.expiry {
            Some(exp) if exp < SystemTime::now() => true,
            _ => false,
        }
    }
}
