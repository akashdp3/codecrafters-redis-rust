use std::sync::Arc;

use anyhow::Context;
use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::store::Store;
use crate::{command, resp};

const SERVER_ADDR: &'static str = "127.0.0.1:6379";

async fn handle_client(socket: &mut TcpStream, store: &mut Store) -> anyhow::Result<()> {
    let mut buf = [0; 1024];

    loop {
        let n = socket
            .read(&mut buf)
            .await
            .context("Failed to read from socket")?;

        if n == 0 {
            return Ok(())
        }

        let args = resp::parse(Bytes::from(buf[..n].to_vec()))
            .await
            .context("Failed to parse RESP message")?;

        let result = match command::execute(store, args).await {
            Ok(result) => Bytes::from(result),
            Err(err) => Bytes::from(format!("-{}\r\n", err))
        };

        socket
            .write_all(&result)
            .await
            .context("Failed to write to socket")?;
    }
}

pub(crate) async fn handle_connection() -> anyhow::Result<()> {
    let listener = TcpListener::bind(SERVER_ADDR).await?;
    let store = Arc::new(Mutex::new(Store::new()));

    loop {
        let (mut socket, _) = listener.accept().await?;
        let store = store.clone();

        tokio::spawn(async move {
            let mut store = store.lock().await;

            if let Err(e) = handle_client(&mut socket, &mut store).await {
                eprintln!("Failed to handle client; err = {:?}", e);
            }
        });
    }
}
