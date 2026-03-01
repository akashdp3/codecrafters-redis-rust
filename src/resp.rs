use anyhow::{Context, Ok};
use bytes::Bytes;

pub(crate) enum Resp {
    SimpleString(String),
    SimpleError(String),
    BulkString(Option<String>),
    Array(Vec<Resp>),
}

impl Resp {
    pub(crate) fn encode(&self) -> String {
        match self {
            Resp::SimpleString(msg) => format!("+{}\r\n", msg),
            Resp::SimpleError(msg) => format!("-{}\r\n", msg),
            Resp::BulkString(Some(msg)) => format!("${}\r\n{}\r\n", msg.len(), msg),
            Resp::BulkString(None) => format!("$-1\r\n"),
            Resp::Array(msgs) => {
                let mut encoded = format!("*{}\r\n", msgs.len());
                for msg in msgs {
                    encoded.push_str(&msg.encode());
                }
                encoded
            }
        }
    }

    pub(crate) fn decode(buf: Bytes) -> anyhow::Result<Vec<String>> {
        let args = match buf.first() {
            Some(b'*') => parse_array(buf.slice(1..))?,
            _ => anyhow::bail!("Unsupported RESP type"),
        };

        Ok(args)
    }

    pub(crate) fn bulk(msg: impl Into<String>) -> Resp {
        Resp::BulkString(Some(msg.into()))
    }

    pub(crate) fn ok() -> Resp {
        Resp::SimpleString("OK".to_string())
    }

    pub(crate) fn null() -> Resp {
        Resp::BulkString(None)
    }

    pub(crate) fn error(msg: &str) -> Resp {
        Resp::SimpleError(msg.to_string())
    }
}

fn parse_array(buf: Bytes) -> anyhow::Result<Vec<String>> {
    let mut pos = buf
        .iter()
        .position(|&b| b == b'\r')
        .ok_or_else(|| anyhow::anyhow!("Invalid RESP array: missing CRLF"))?;

    let param_count = std::str::from_utf8(&buf[..pos])?
        .parse::<usize>()
        .context("Failed to parse array length")?;

    let mut result = vec![];

    pos += 2;
    for _ in 0..param_count {
        let word = buf.slice(pos..);

        let crlf_pos = word
            .iter()
            .position(|&b| b == b'\r')
            .ok_or_else(|| anyhow::anyhow!("Invalid RESP string; missing CRLF"))?;
        let data_len = std::str::from_utf8(&word[1..crlf_pos])?
            .parse::<usize>()
            .context("Failed to parse RESP string")?;

        let data_start = crlf_pos + 2;
        let data_end = data_start + data_len;
        result.push(
            std::str::from_utf8(&word[data_start..data_end])
                .context("Failed parse UTF-8")?
                .to_string(),
        );

        pos = pos + data_end + 2;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_simple_string() {
        let resp = Resp::SimpleString("OK".to_string());
        assert_eq!(resp.encode(), "+OK\r\n");
    }

    #[test]
    fn test_encode_simple_error() {
        let resp = Resp::SimpleError("ERR unknown".to_string());
        assert_eq!(resp.encode(), "-ERR unknown\r\n");
    }

    #[test]
    fn test_encode_bulk_string() {
        let resp = Resp::BulkString(Some("hello".to_string()));
        assert_eq!(resp.encode(), "$5\r\nhello\r\n");
    }

    #[test]
    fn test_encode_null_bulk_string() {
        let resp = Resp::BulkString(None);
        assert_eq!(resp.encode(), "$-1\r\n");
    }

    #[test]
    fn test_ok_helper() {
        assert_eq!(Resp::ok().encode(), "+OK\r\n");
    }

    #[test]
    fn test_null_helper() {
        assert_eq!(Resp::null().encode(), "$-1\r\n");
    }

    #[test]
    fn test_decode_single_element_array() {
        let input = Bytes::from("*1\r\n$4\r\nPING\r\n");
        let result = Resp::decode(input).unwrap();
        assert_eq!(result, vec!["PING"]);
    }

    #[test]
    fn test_decode_two_element_array() {
        let input = Bytes::from("*2\r\n$4\r\nECHO\r\n$5\r\nhello\r\n");
        let result = Resp::decode(input).unwrap();
        assert_eq!(result, vec!["ECHO", "hello"]);
    }

    #[test]
    fn test_decode_set_command() {
        let input = Bytes::from("*3\r\n$3\r\nSET\r\n$3\r\nfoo\r\n$3\r\nbar\r\n");
        let result = Resp::decode(input).unwrap();
        assert_eq!(result, vec!["SET", "foo", "bar"]);
    }

    #[test]
    fn test_decode_invalid_type() {
        let input = Bytes::from("+OK\r\n");
        let result = Resp::decode(input);
        assert!(result.is_err());
    }
}
