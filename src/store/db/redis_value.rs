use super::StreamValue;

#[derive(Debug, Clone)]
pub(crate) enum RedisValue {
    String(String),
    Stream(Vec<StreamValue>),
    List(Vec<String>),
    SortedSet(Vec<(String, f64)>),
}

impl RedisValue {
    pub(crate) fn type_of(&self) -> &str {
        match self {
            RedisValue::String(..) => "string",
            RedisValue::Stream(..) => "stream",
            RedisValue::List(..) => "list",
            RedisValue::SortedSet(..) => "sorted-set",
        }
    }
}
