use crate::{handler::handle, Store};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

pub async fn listen(addr: &str, store: Store) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    let store = Arc::new(Mutex::new(store));

    loop {
        let (mut socket, _addr) = listener.accept().await?;
        let store = store.clone();

        tokio::spawn(async move {
            if let Err(e) = handle(&mut socket, &store).await {
                eprintln!("Failed to handle client; err = {:?}", e);
            }
        });
    }
}
