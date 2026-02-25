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
        match self {
            Some(duration) => Some(SystemTime::now() + duration),
            None => None,
        }
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
        key: &str,
        value: &str,
        expiry: Option<SystemTime>,
    ) -> anyhow::Result<()> {
        self.data.insert(
            key.to_string(),
            RedisValue {
                value: value.to_string(),
                expiry,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_set_and_get() {
        let mut db = Db::new();
        db.set("foo", "bar", None).unwrap();
        assert_eq!(db.get("foo"), Some("bar".to_string()));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let db = Db::new();
        assert_eq!(db.get("missing"), None);
    }

    #[test]
    fn test_overwrite_key() {
        let mut db = Db::new();
        db.set("key", "value1", None).unwrap();
        db.set("key", "value2", None).unwrap();
        assert_eq!(db.get("key"), Some("value2".to_string()));
    }

    #[test]
    fn test_set_with_expiry_not_expired() {
        let mut db = Db::new();
        db.set("key", "value", Some(Duration::from_secs(10)))
            .unwrap();
        assert_eq!(db.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_set_with_expiry_expired() {
        let mut db = Db::new();
        db.set("key", "value", Some(Duration::from_millis(10)))
            .unwrap();

        sleep(Duration::from_millis(20));

        assert_eq!(db.get("key"), None);
    }

    #[test]
    fn test_multiple_keys() {
        let mut db = Db::new();
        db.set("a", "1", None).unwrap();
        db.set("b", "2", None).unwrap();
        db.set("c", "3", None).unwrap();

        assert_eq!(db.get("a"), Some("1".to_string()));
        assert_eq!(db.get("b"), Some("2".to_string()));
        assert_eq!(db.get("c"), Some("3".to_string()));
    }

    #[test]
    fn test_keys_fn() {
        let mut db = Db::new();
        db.set("foo", "1", None).unwrap();
        db.set("bar", "2", None).unwrap();
        db.set("cat", "3", None).unwrap();

        let mut result = db.keys("*");
        result.sort();
        assert_eq!(result, vec!["bar", "cat", "foo"]);

        let mut result = db.keys("f*");
        result.sort();
        assert_eq!(result, vec!["foo"]);

        let mut result = db.keys("*a*");
        result.sort();
        assert_eq!(result, vec!["bar", "cat"]);
    }
}
