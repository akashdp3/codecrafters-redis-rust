use anyhow::Context;
use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::resp;

const SERVER_ADDR: &'static str = "127.0.0.1:6379";

async fn handle_client(socket: &mut TcpStream) -> anyhow::Result<()> {
    let mut buf = [0; 1024];

    let n = socket
        .read(&mut buf)
        .await
        .context("Failed to read from socket")?;

    if n == 0 {
        return Err(anyhow::anyhow!("Client disconnected"));
    }

    let result = resp::parse(Bytes::from(buf[..n].to_vec()))
        .await
        .context("Failed to parse RESP message")?;

    socket
        .write_all(&result)
        .await
        .context("Failed to write to socket")
}

pub(crate) async fn handle_connection() -> anyhow::Result<()> {
    let listener = TcpListener::bind(SERVER_ADDR).await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket).await {
                eprintln!("Failed to handle client; err = {:?}", e);
            }
        });
    }
}
