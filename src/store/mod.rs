pub(crate) mod acl_user;
mod config;
mod db;

use anyhow::Context;
use config::Config;
use db::Db;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use tokio::{
    fs,
    sync::{mpsc::UnboundedSender, Notify},
};

use crate::{rdb_parser::RdbParser, store::acl_user::AclUser, Conn, Resp};

pub(crate) use db::RedisValue;

#[derive(Debug)]
pub(crate) struct ReplicaState {
    pub(crate) conn: Conn,
    pub(crate) ack_offset: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Subscriber {
    id: Uuid,
    pub(crate) sender: UnboundedSender<String>,
}

#[derive(Debug)]
pub(crate) struct Store {
    pub(crate) config: Config,
    pub(crate) db: Db,
    pub(crate) replicas: Vec<ReplicaState>,
    pub(crate) master_repl_offset: usize,
    pub(crate) notify: Arc<Notify>,
    pub(crate) to_be_notified_items: Vec<String>,
    pub(crate) pubsub: HashMap<String, Vec<Subscriber>>,
    pub(crate) users: HashMap<String, AclUser>,
    offset: usize,
}

impl Store {
    pub(crate) async fn init(
        dir: &str,
        db_file_name: &str,
        replica_of: &str,
    ) -> anyhow::Result<Self> {
        let config = Config::new(dir, db_file_name, replica_of);
        let db = match Self::load_data(&config).await {
            Ok(data) => data,
            Err(err) => {
                eprintln!("Failed to load data from .rdb file, Err: {}", err);
                Db::new()
            }
        };

        Ok(Self {
            config,
            db,
            replicas: vec![],
            master_repl_offset: 0,
            notify: Arc::new(Notify::new()),
            to_be_notified_items: vec![],
            pubsub: HashMap::new(),
            users: HashMap::new(),
            offset: 0,
        })
    }

    async fn load_data(config: &Config) -> anyhow::Result<Db> {
        // Check if dir and db_file_name are passed
        if config.dir().is_empty() || config.db_file_name().is_empty() {
            return Ok(Db::new());
        }

        let dir_path = fs::canonicalize(config.dir())
            .await
            .with_context(|| format!("Failed to canonicalize dir_path: {}", config.dir()))?;
        let rdb_file_path = fs::canonicalize(dir_path.join(config.db_file_name())).await?;

        let mut parser = RdbParser::new(rdb_file_path).await?;
        let rdb = parser.parse().await?;

        let mut db = Db::new();

        for (key, redis_value) in rdb.data.into_iter() {
            db.set(
                key,
                RedisValue::String(redis_value.value),
                redis_value.expiry,
            )?;
        }

        Ok(db)
    }

    pub(crate) fn add_replica(&mut self, conn: Conn) {
        self.replicas.push(ReplicaState {
            conn,
            ack_offset: 0,
        });
    }

    pub(crate) fn increment_offset(&mut self, frame_len: usize) {
        self.offset += frame_len;
    }

    pub(crate) fn get_offset(&mut self) -> usize {
        self.offset
    }

    pub(crate) fn add_subscriber(
        &mut self,
        channel: String,
        client_id: Uuid,
        tx: UnboundedSender<String>,
    ) {
        let subs = self.pubsub.entry(channel).or_default();
        if !subs.iter().any(|s| s.id == client_id) {
            subs.push(Subscriber {
                id: client_id,
                sender: tx,
            })
        }
    }

    pub(crate) fn remove_subscriber(&mut self, id: Uuid) {
        for subs in self.pubsub.values_mut() {
            subs.retain(|s| s.id != id);
        }
    }

    pub(crate) async fn publish(&mut self, channel: String, msg: String) -> usize {
        let mut len = 0;
        let subscribers = match self.pubsub.get(&channel) {
            Some(e) => e,
            None => {
                eprintln!("Invalid Operations");
                &vec![]
            }
        };

        let push = Resp::Array(vec![
            Resp::bulk("message"),
            Resp::bulk(channel.clone()),
            Resp::bulk(msg),
        ])
        .encode();

        for subscriber in subscribers {
            if subscriber.sender.send(push.clone()).is_ok() {
                len += 1;
            }
        }

        len
    }
}
