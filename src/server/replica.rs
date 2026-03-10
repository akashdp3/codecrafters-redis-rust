use tokio::net::TcpStream;

use crate::{server::Conn, Store};

pub(crate) async fn init(store: &Store) -> anyhow::Result<()> {
    let master_addr = store.config.master_addr();
    let stream = TcpStream::connect(master_addr).await?;
    let mut conn = Conn::new(stream);

    // PING command to master
    let msg = "*1\r\n$4\r\nping\r\n";
    conn.write_raw(msg.as_bytes()).await?;
    conn.read_raw().await?;

    // REPL_CONF command to send listening_port and capa to master
    let msg = "*3\r\n$8\r\nREPLCONF\r\n$14\r\nlistening-port\r\n$4\r\n6380\r\n";
    conn.write_raw(msg.as_bytes()).await?;
    conn.read_raw().await?;

    let msg = "*3\r\n$8\r\nREPLCONF\r\n$4\r\ncapa\r\n$6\r\npsync2\r\n";
    conn.write_raw(msg.as_bytes()).await?;
    conn.read_raw().await?;

    // PSYNC command to master
    let msg = "*3\r\n$5\r\npsync\r\n$1\r\n?\r\n$2\r\n-1\r\n";
    conn.write_raw(msg.as_bytes()).await?;
    conn.read_raw().await?;

    Ok(())
}
