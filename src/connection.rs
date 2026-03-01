use anyhow::Context;
use bytes::Bytes;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::{command::Command, resp::Resp, store::Store};

async fn handle_client(socket: &mut TcpStream, store: &Arc<Mutex<Store>>) -> anyhow::Result<()> {
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
                Resp::error("Something went wrong").encode()
            }
        };
        let result = Bytes::from(result);

        socket
            .write_all(&result)
            .await
            .context("Failed to write to socket")?;
    }
}

pub(crate) async fn handle_connection(
    dir: &str,
    db_file_name: &str,
    server_addr: &str,
    replica_of: &str,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(server_addr).await?;
    let store = Store::init(dir, db_file_name, replica_of).await?;
    let store = Arc::new(Mutex::new(store));

    loop {
        let (mut socket, _) = listener.accept().await?;
        let store = store.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket, &store).await {
                eprintln!("Failed to handle client; err = {:?}", e);
            }
        });
    }
}
