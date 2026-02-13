use anyhow::{Context, Ok};
use bytes::Bytes;

pub(crate) async fn parse(buf: Bytes) -> anyhow::Result<Bytes> {
    let parsed_msg = match buf.first() {
        Some(b'*') => parse_array(buf.slice(1..)).await?,
        _ => anyhow::bail!("Unsupported RESP type"),
    };

    let result = match parsed_msg[0].as_str() {
        "PING" => Bytes::from_static(b"+PONG\r\n"),
        "ECHO" => Bytes::from(format!("+{}\r\n", parsed_msg.get(1).unwrap())),
        _ => anyhow::bail!("Unsupported command"),
    };

    Ok(result)
}

async fn parse_array(buf: Bytes) -> anyhow::Result<Vec<String>> {
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

        let crlf_pos = word.iter().position(|&b| b == b'\r').ok_or_else(|| anyhow::anyhow!("Invalid RESP string; missing CRLF"))?;
        let data_len = std::str::from_utf8(&word[1..crlf_pos])?.parse::<usize>().context("Failed to parse RESP string")?;

        let data_start = crlf_pos + 2;
        let data_end = data_start + data_len;
        result.push(std::str::from_utf8(&word[data_start..data_end]).context("Failed parse UTF-8")?.to_string());

        pos = pos + data_end + 2;
    }
    println!("{:?}", result);

    Ok(result)
}
