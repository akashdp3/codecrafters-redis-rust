use std::sync::Arc;

use clap::Parser;

mod command;
mod handler;
mod rdb_parser;
mod resp;
mod server;
mod store;

pub(crate) use command::Command;
pub(crate) use resp::Resp;
pub(crate) use server::{replica, Conn};
pub(crate) use store::Store;
use tokio::sync::Mutex;

const HOST_URL: &str = "127.0.0.1";

#[derive(Debug, Parser)]
pub(crate) struct Args {
    #[arg(long = "dir", default_value = "")]
    dir: String,

    #[arg(long = "dbfilename", default_value = "")]
    dbfilename: String,

    #[arg(long = "port", default_value = "6379")]
    port: String,

    #[arg(long = "replicaof", default_value = "")]
    replica_of: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Logs from your program will appear here!\n");

    let args = Args::parse();

    let store = Store::init(&args.dir, &args.dbfilename, &args.replica_of).await?;
    let is_replica = store.config.is_replica();
    let store = Arc::new(Mutex::new(store));

    // Handshake with master server
    if is_replica {
        replica::init(&store).await?;
    }

    // Handle incoming requests
    let server_addr = format!("{}:{}", HOST_URL, args.port);
    server::listen(&server_addr, store).await
}
