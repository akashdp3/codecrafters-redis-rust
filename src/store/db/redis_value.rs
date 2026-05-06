use super::StreamValue;

#[derive(Debug)]
pub(crate) enum RedisValue {
    String(String),
    Stream(Vec<StreamValue>),
}

impl RedisValue {
    pub(crate) fn type_of(&self) -> &str {
        match self {
            RedisValue::String(..) => "string",
            RedisValue::Stream(..) => "stream",
        }
    }
}
