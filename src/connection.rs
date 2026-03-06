use crate::store::Store;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

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
