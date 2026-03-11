use std::sync::Arc;

use anyhow::Context;
use tokio::sync::Mutex;

use crate::{Command, Conn, Resp, Store};

pub async fn handle_client(mut conn: Conn, store: &Arc<Mutex<Store>>) -> anyhow::Result<()> {
    loop {
        let args = match conn.read_frame().await {
            Ok(args) => args,
            Err(_) => break,
        };

        let mut store = store.lock().await;
        let cmd = Command::parse(args).context("Failed to parse command")?;
        let is_psync = matches!(&cmd, Command::PSYNC { .. });

        sync(&cmd, &mut store).await?;

        let result = cmd.execute(&mut store).await?;
        conn.write_raw(&result).await?;

        if is_psync {
            store.add_replica(conn);
            return Ok(());
        }
    }
    Ok(())
}

pub async fn handle_replication(mut conn: Conn, store: Arc<Mutex<Store>>) -> anyhow::Result<()> {
    loop {
        let args = match conn.read_frame().await {
            Ok(args) => args,
            Err(e) => {
                eprintln!("Faile to read frame; Err: {e}");
                break;
            }
        };

        let mut store = store.lock().await;
        let cmd = Command::parse(args).context("Failed to parse command")?;
        let is_replconf_cmd = matches!(&cmd, Command::ReplConf { .. });
        let result = cmd.execute(&mut store).await?;

        if is_replconf_cmd {
            println!("Hello World");
            conn.write_raw(&result).await?;
        }
    }

    Ok(())
}

async fn sync(cmd: &Command, store: &mut Store) -> anyhow::Result<()> {
    let resp_cmd = match cmd {
        Command::Set {
            key,
            value,
            expiry: _,
        } => Some(Resp::array(vec![
            "SET".to_string(),
            key.to_string(),
            value.to_string(),
        ])),
        _ => None,
    };

    if let Some(resp_cmd) = resp_cmd {
        let encoded = resp_cmd.encode();
        for replica_conn in store.replicas.iter_mut() {
            println!("Sent from master: {}", encoded.clone());
            if let Err(e) = replica_conn.write_raw(encoded.as_bytes()).await {
                eprintln!("Failed to write to raplica; Err = {:?}", e);
            }
        }
    }

    Ok(())
}
