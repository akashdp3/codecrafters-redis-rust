use crate::{Command, Resp, Store};
use anyhow::Context;
use bytes::Bytes;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

pub async fn handle(socket: &mut TcpStream, store: &Arc<Mutex<Store>>) -> anyhow::Result<()> {
    let mut buf = [0; 1024];

    loop {
        let n = socket
            .read(&mut buf)
            .await
            .context("Failed to read from socket")?;

        if n == 0 {
            return Ok(());
        }

        let args =
            Resp::decode(Bytes::from(buf[..n].to_vec())).context("Failed to parse RESP message")?;

        let mut store = store.lock().await;
        let cmd = Command::parse(args).context("Failed to parse command")?;
        let result = match cmd.execute(&mut store).await {
            Ok(result) => result,
            Err(err) => {
                eprintln!("Error: {err}");
                Resp::error("Something went wrong").encode().into_bytes()
            }
        };
        let result = Bytes::from(result);

        socket
            .write_all(&result)
            .await
            .context("Failed to write to socket")?;
    }
}
