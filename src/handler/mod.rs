use std::sync::Arc;

use anyhow::Context;
use tokio::sync::Mutex;

use crate::{Command, Conn, Store};

pub async fn handle(conn: &mut Conn, store: &Arc<Mutex<Store>>) -> anyhow::Result<()> {
    loop {
        let args = match conn.read_frame().await {
            Ok(args) => args,
            Err(_) => break,
        };

        let mut store = store.lock().await;
        let cmd = Command::parse(args).context("Failed to parse command")?;
        let result = cmd.execute(&mut store).await?;
        conn.write_raw(&result).await?;
    }
    Ok(())
}
