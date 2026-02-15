use anyhow::{Context, Ok};
use bytes::Bytes;

pub(crate) enum Resp {
    SimpleString(String),
    SimpleError(String),
    BulkString(Option<String>),
}

impl Resp {
    pub(crate) fn encode(&self) -> String {
        match self {
            Resp::SimpleString(msg) => format!("+{}\r\n", msg),
            Resp::SimpleError(msg) => format!("-{}\r\n", msg),
            Resp::BulkString(Some(msg)) => format!("${}\r\n{}\r\n", msg.len(), msg),
            Resp::BulkString(None) => format!("$-1\r\n"),
        }
    }

    pub(crate) fn decode(buf: Bytes) -> anyhow::Result<Vec<String>> {
        let args = match buf.first() {
            Some(b'*') => parse_array(buf.slice(1..))?,
            _ => anyhow::bail!("Unsupported RESP type"),
        };

        Ok(args)
    }

    pub(crate) fn ok() -> Resp {
        Resp::SimpleString("OK".to_string())
    }

    pub(crate) fn null() -> Resp {
        Resp::BulkString(None)
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
