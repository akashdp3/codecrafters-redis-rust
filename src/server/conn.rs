use anyhow::Context;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use crate::Resp;

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

    async fn read_raw(&mut self) -> anyhow::Result<usize> {
        let n = self
            .stream
            .read_buf(&mut self.buffer)
            .await
            .context("Failed to read from stream to buf")?;

        Ok(n)
    }

    pub async fn read_frame(&mut self) -> anyhow::Result<Vec<String>> {
        let len = self.read_raw().await?;
        if len == 0 {
            anyhow::bail!("Connection closed");
        }

        let data = self.buffer.split_to(len).freeze();
        let args = Resp::decode(data)?;
        Ok(args)
    }

    pub async fn write_raw(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.stream.write_all(bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }
}
