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

pub(crate) async fn handle_connection(server_addr: &str, store: Store) -> anyhow::Result<()> {
    println!("Listening on server: {}", server_addr);
    let listener = TcpListener::bind(server_addr).await?;
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

pub(crate) async fn send_connection(store: &Store) -> anyhow::Result<()> {
    let master_addr = store.config.master_addr();

    println!("Connecting to master: {}", master_addr);
    let mut buf = [0; 1024];
    let mut stream = TcpStream::connect(master_addr).await?;

    // PING command
    let msg = "*1\r\n$4\r\nPING\r\n";
    stream.write_all(msg.as_bytes()).await?;
    stream.read(&mut buf).await?;
    println!(
        "Sending PING. Got response: {}",
        std::str::from_utf8(&buf).unwrap_or("<invalid UTF-8>")
    );

    // Informing master regarding replica's config i.e. listening-port
    let mut buf = [0; 1024];
    let msg = "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n";
    stream.write_all(msg.as_bytes()).await?;
    stream.read(&mut buf).await?;
    println!(
        "Sending REPLCONF for port. Got response: {}",
        std::str::from_utf8(&buf).unwrap_or("<invalid UTD-8>")
    );

    // Informing master regarding replica's config i.e. capabilities
    let mut buf = [0; 1024];
    let msg = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n";
    stream.write_all(msg.as_bytes()).await?;
    stream.read(&mut buf).await?;
    println!(
        "Sending REPLCONF for capabilities. Got response: {}",
        std::str::from_utf8(&buf).unwrap_or("<invalid UTD-8>")
    );

    // Sync data between replica and master
    let mut buf = [0; 1024];
    let msg = "*3\r\n$5\r\nPSYNC\r\n$1\r\n?\r\n$2\r\n-1\r\n";
    stream.write_all(msg.as_bytes()).await?;
    stream.read(&mut buf).await?;
    println!(
        "Sending PSYNC. Got response: {}",
        std::str::from_utf8(&buf).unwrap_or("<invalid UTD-8>")
    );
    Ok(())
}
