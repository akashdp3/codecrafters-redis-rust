use std::sync::Arc;

use tokio::{net::TcpStream, sync::Mutex};

use crate::{handler::handle_replication, server::Conn, Store};

pub(crate) async fn init(store: &Arc<Mutex<Store>>) -> anyhow::Result<()> {
    let master_addr = store.lock().await.config.master_addr().to_string();

    let stream = TcpStream::connect(master_addr).await?;
    let mut conn = Conn::new(stream);

    // PING command to master
    let msg = "*1\r\n$4\r\nping\r\n";
    conn.write_raw(msg.as_bytes()).await?;
    conn.read_frame().await?;

    // REPL_CONF command to send listening_port and capa to master
    let msg = "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n";
    conn.write_raw(msg.as_bytes()).await?;
    conn.read_frame().await?;

    let msg = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n";
    conn.write_raw(msg.as_bytes()).await?;
    conn.read_frame().await?;

    // PSYNC command to master
    // TODO: PSYNC and RDB file is read in single frame. Fix this.
    let msg = "*3\r\n$5\r\npsync\r\n$1\r\n?\r\n$2\r\n-1\r\n";
    conn.write_raw(msg.as_bytes()).await?;
    conn.read_frame().await?;
    conn.read_frame().await?;
    // conn.clear_buffer();

    let store = store.clone();
    tokio::spawn(async move {
        if let Err(e) = handle_replication(conn, store).await {
            eprintln!("Failed to handle replication; Err = {:?}", e);
        }
    });

    Ok(())
}
