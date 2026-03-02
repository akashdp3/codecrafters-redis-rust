use crate::{command::Command, resp::Resp, store::Store};
use anyhow::Context;
use bytes::Bytes;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

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
    server_addr: &str,
    store: Arc<Mutex<Store>>,
) -> anyhow::Result<()> {
    println!("Listening on server: {}", server_addr);
    let listener = TcpListener::bind(server_addr).await?;

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

pub(crate) async fn send_connection(master_addr: &str) -> anyhow::Result<()> {
    println!("Connecting to master: {}", master_addr);
    let ping = "*1\r\n$4\r\nPING\r\n";
    let mut buf = [0; 1024];

    println!("Sending PING");
    let mut stream = TcpStream::connect(master_addr).await?;
    stream.write_all(ping.as_bytes()).await?;
    stream.read(&mut buf).await?;
    println!(
        "Got Response: {}",
        std::str::from_utf8(&buf).unwrap_or("<invalid UTF-8>")
    );

    Ok(())
}
