use crate::{handler::handle_client, server::Conn, Client, Store};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

pub async fn listen(addr: &str, store: Arc<Mutex<Store>>) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _addr) = listener.accept().await?;
        let mut client = Client::new();
        {
            let mut s = store.lock().await;
            let user = s.users.entry("default".to_string()).or_default();
            if user.flags().iter().any(|flag| flag == "nopass") {
                client.authenticated_user = Some("default".to_string());
            }
        }
        let store = store.clone();

        tokio::spawn(async move {
            let conn = Conn::new(stream);

            if let Err(e) = handle_client(conn, &store, &mut client).await {
                eprintln!("Failed to handle client; err = {}", e);
            }
        });
    }
}
