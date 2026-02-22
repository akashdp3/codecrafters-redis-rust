use std::env;
use std::sync::Arc;

use anyhow::Context;
use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use crate::command::Command;
use crate::resp::Resp;
use crate::store::Store;

const SERVER_ADDR: &str = "127.0.0.1:6379";
const DEFAULT_DIR: &str = "";
const DEFAULT_FILE_NAME: &str = "";

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

pub(crate) async fn handle_connection() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let dir_name = args.get(2).map(|s| s.as_str()).unwrap_or(DEFAULT_DIR);
    let db_file_name = args.get(4).map(|s| s.as_str()).unwrap_or(DEFAULT_FILE_NAME);

    let listener = TcpListener::bind(SERVER_ADDR).await?;
    let store = Store::init(dir_name, db_file_name).await?;
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
