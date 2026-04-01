use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::{Command, Resp, Store};

pub(crate) fn parse(args: &mut impl Iterator<Item = String>) -> anyhow::Result<Command> {
    let numreplicas: u8 = args
        .next()
        .context("Missing argument 'numreplicas' for WAIT command")?
        .parse()?;
    let timeout: u16 = args
        .next()
        .context("Missing argument 'timeout' for WAIT command")?
        .parse()?;

    Ok(Command::Wait {
        numreplicas,
        timeout,
    })
}

pub(crate) async fn invoke(
    store: Arc<Mutex<Store>>,
    numreplicas: u8,
    timeout: u16,
) -> anyhow::Result<Vec<u8>> {
    let deadline = Instant::now() + Duration::from_millis(timeout as u64);

    let (master_offset, replica_count) = {
        let s = store.lock().await;
        (s.master_repl_offset, s.replicas.len())
    };

    // Fast path: no writes have been propagated yet
    if master_offset == 0 {
        return Ok(Resp::integer(replica_count).encode().into_bytes());
    }

    // Send REPLCONF GETACK * to all replicas
    let getack = Resp::array(vec![
        "REPLCONF".to_string(),
        "GETACK".to_string(),
        "*".to_string(),
    ])
    .encode();
    {
        let mut s = store.lock().await;
        for replica in s.replicas.iter_mut() {
            if let Err(e) = replica.conn.write_raw(getack.as_bytes()).await {
                eprintln!("Failed to send GETACK to replica; Err = {:?}", e);
            }
        }
    }

    // Poll for ACK responses until enough replicas confirm or timeout expires
    loop {
        let now = Instant::now();
        if now >= deadline {
            break;
        }

        {
            let mut s = store.lock().await;
            for replica in s.replicas.iter_mut() {
                if replica.ack_offset >= master_offset {
                    continue;
                }
                match tokio::time::timeout(Duration::from_millis(5), replica.conn.read_frame())
                    .await
                {
                    Ok(Ok((_, args))) => {
                        // Expect: REPLCONF ACK <offset>
                        if args.get(0).map(|s| s.to_uppercase()) == Some("REPLCONF".to_string())
                            && args.get(1).map(|s| s.to_uppercase()) == Some("ACK".to_string())
                        {
                            if let Some(offset) = args.get(2).and_then(|s| s.parse::<usize>().ok())
                            {
                                replica.ack_offset = offset;
                            }
                        }
                    }
                    _ => {}
                }
            }

            let acked = s
                .replicas
                .iter()
                .filter(|r| r.ack_offset >= master_offset)
                .count();
            if acked >= numreplicas as usize {
                return Ok(Resp::integer(acked).encode().into_bytes());
            }
        }

        tokio::time::sleep(Duration::from_millis(5)).await;
    }

    let s = store.lock().await;
    let acked = s
        .replicas
        .iter()
        .filter(|r| r.ack_offset >= master_offset)
        .count();
    Ok(Resp::integer(acked).encode().into_bytes())
}
