use crate::{handler::handle_client, server::Conn, Store};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

pub async fn listen(addr: &str, store: Arc<Mutex<Store>>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _addr) = listener.accept().await?;
        let store = store.clone();

        tokio::spawn(async move {
            let conn = Conn::new(stream);

            if let Err(e) = handle_client(conn, &store).await {
                eprintln!("Failed to handle client; err = {}", e);
            }
        });
    }
}
