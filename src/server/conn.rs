use anyhow::Context;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::Resp;

#[derive(Debug)]
pub struct Conn {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Conn {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(1024),
        }
    }

    pub fn _clear_buffer(&mut self) {
        self.buffer.clear();
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        self.stream
            .flush()
            .await
            .context("Failed to flush stream")?;

        Ok(())
    }

    pub async fn read_raw(&mut self) -> anyhow::Result<usize> {
        let n = self
            .stream
            .read_buf(&mut self.buffer)
            .await
            .context("Failed to read from stream to buf")?;

        Ok(n)
    }

    pub async fn write_raw(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.stream.write_all(bytes).await?;
        self.flush().await?;

        Ok(())
    }

    pub async fn _write_frame(&mut self, resp: Resp) -> anyhow::Result<()> {
        let resp_str = resp.encode();
        self.write_raw(resp_str.as_bytes()).await?;

        Ok(())
    }

    pub async fn read_frame(&mut self) -> anyhow::Result<(usize, Vec<String>)> {
        loop {
            // Try to parse a complete frame from what's already buffered
            if let Some(frame_len) = self.find_frame_end() {
                let data = self.buffer.split_to(frame_len).freeze();
                return Ok((frame_len, Resp::decode(data)?));
            }
            // Not enough data yet, read more
            let n = self.read_raw().await?;
            if n == 0 {
                anyhow::bail!("Connection closed");
            }
        }
    }

    fn find_frame_end(&self) -> Option<usize> {
        match self.buffer.first()? {
            b'+' | b'-' => {
                // SimpleString/Error: ends at \r\n
                let pos = self.buffer.iter().position(|&b| b == b'\n')?;
                Some(pos + 1)
            }
            b'$' => {
                // BulkString: $<len>\r\n<data>\r\n
                let crlf = self.buffer.iter().position(|&b| b == b'\n')?;
                let len: usize = std::str::from_utf8(&self.buffer[1..crlf - 1])
                    .ok()?
                    .parse()
                    .ok()?;
                let total = crlf + 1 + len;
                if self.buffer.len() >= total {
                    Some(total)
                } else {
                    None
                }
            }
            b'*' => {
                // Array: need to scan all elements
                // Simple approach: find end by counting \r\n pairs
                let crlf = self.buffer.iter().position(|&b| b == b'\n')?;
                let count: usize = std::str::from_utf8(&self.buffer[1..crlf - 1])
                    .ok()?
                    .parse()
                    .ok()?;
                let mut pos = crlf + 1;
                for _ in 0..count {
                    // skip $<len>\r\n
                    let lf = self.buffer[pos..].iter().position(|&b| b == b'\n')? + pos;
                    let len: usize = std::str::from_utf8(&self.buffer[pos + 1..lf - 1])
                        .ok()?
                        .parse()
                        .ok()?;
                    pos = lf + 1 + len + 2;
                }
                if self.buffer.len() >= pos {
                    Some(pos)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
