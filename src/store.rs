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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_set_and_get() {
        let mut store = Store::new();
        store.set("foo", "bar", None).unwrap();
        assert_eq!(store.get("foo"), Some("bar".to_string()));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let store = Store::new();
        assert_eq!(store.get("missing"), None);
    }

    #[test]
    fn test_overwrite_key() {
        let mut store = Store::new();
        store.set("key", "value1", None).unwrap();
        store.set("key", "value2", None).unwrap();
        assert_eq!(store.get("key"), Some("value2".to_string()));
    }

    #[test]
    fn test_set_with_expiry_not_expired() {
        let mut store = Store::new();
        store
            .set("key", "value", Some(Duration::from_secs(10)))
            .unwrap();
        assert_eq!(store.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_set_with_expiry_expired() {
        let mut store = Store::new();
        store
            .set("key", "value", Some(Duration::from_millis(10)))
            .unwrap();

        sleep(Duration::from_millis(20));

        assert_eq!(store.get("key"), None);
    }

    #[test]
    fn test_multiple_keys() {
        let mut store = Store::new();
        store.set("a", "1", None).unwrap();
        store.set("b", "2", None).unwrap();
        store.set("c", "3", None).unwrap();

        assert_eq!(store.get("a"), Some("1".to_string()));
        assert_eq!(store.get("b"), Some("2".to_string()));
        assert_eq!(store.get("c"), Some("3".to_string()));
    }
}
