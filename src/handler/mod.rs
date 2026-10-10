use std::sync::Arc;

use anyhow::Context;
use tokio::sync::Mutex;

use crate::{Command, Conn, Resp, Store};

pub async fn handle_client(mut conn: Conn, store: &Arc<Mutex<Store>>) -> anyhow::Result<()> {
    let mut queue: Option<Vec<Command>> = None;

    loop {
        let (_, args) = match conn.read_frame().await {
            Ok((frame_len, args)) => (frame_len, args),
            Err(e) if e.to_string().contains("Connection closed") => {
                break;
            }
            Err(e) => {
                eprintln!("Failed to read frame; Err: {e}");
                break;
            }
        };

        let cmd = Command::parse(args).context("Failed to parse command")?;
        let is_psync = matches!(&cmd, Command::Psync { .. });

        sync(&cmd, store).await?;

        let response = match cmd {
            Command::Multi => {
                queue = Some(vec![]);
                Resp::ok().encode().into_bytes()
            }
            Command::Discard => match queue {
                Some(_) => {
                    queue = None;

                    Resp::ok().encode().into_bytes()
                }
                None => Resp::error("DISCARD without MULTI").encode().into_bytes(),
            },
            Command::Exec => match queue.take() {
                Some(cmds) => {
                    let mut results = format!("*{}\r\n", cmds.len()).into_bytes();

                    for cmd in cmds {
                        match cmd.execute(Arc::clone(store)).await {
                            Ok(r) => {
                                results.extend(r);
                            }
                            Err(err) => {
                                let msg = Resp::error(&err.to_string()).encode().into_bytes();
                                results.extend(msg);
                            }
                        }
                    }

                    results
                }
                None => Resp::error("EXEC without MULTI").encode().into_bytes(),
            },
            cmd => match queue.as_mut() {
                Some(cmds) => {
                    cmds.push(cmd);
                    Resp::SimpleString("QUEUED".to_string())
                        .encode()
                        .into_bytes()
                }
                None => match cmd.execute(Arc::clone(store)).await {
                    Ok(r) => r,
                    Err(err) => {
                        let msg = Resp::error(&err.to_string()).encode().into_bytes();
                        conn.write_raw(&msg).await?;
                        continue;
                    }
                },
            },
        };

        // let result = match cmd.execute(Arc::clone(store)).await {
        //     Ok(result) => result,
        //     Err(err) => {
        //         let error_msg = Resp::error(&err.to_string()).encode().into_bytes();
        //         conn.write_raw(&error_msg).await?;
        //         continue;
        //     }
        // };

        conn.write_raw(&response).await?;

        if is_psync {
            store.lock().await.add_replica(conn);
            return Ok(());
        }
    }
    Ok(())
}

pub async fn handle_replication(mut conn: Conn, store: Arc<Mutex<Store>>) -> anyhow::Result<()> {
    loop {
        let (frame_len, args) = match conn.read_frame().await {
            Ok((frame_len, args)) => (frame_len, args),
            Err(e) if e.to_string().contains("Connection closed") => {
                break;
            }
            Err(e) => {
                eprintln!("Failed to read frame; Err: {e}");
                break;
            }
        };

        let cmd = Command::parse(args).context("Failed to parse command")?;
        let is_replconf_cmd = matches!(&cmd, Command::ReplConf { .. });
        let result = cmd.execute(Arc::clone(&store)).await?;

        if is_replconf_cmd {
            conn.write_raw(&result).await?;
        }

        store.lock().await.increment_offset(frame_len);
    }

    Ok(())
}

async fn sync(cmd: &Command, store: &Arc<Mutex<Store>>) -> anyhow::Result<()> {
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
        let byte_count = encoded.len();
        let mut s = store.lock().await;
        for replica in s.replicas.iter_mut() {
            if let Err(e) = replica.conn.write_raw(encoded.as_bytes()).await {
                eprintln!("Failed to write to replica; Err = {:?}", e);
            }
        }
        s.master_repl_offset += byte_count;
    }

    Ok(())
}
