use clap::Parser;

mod command;
mod connection;
mod rdb_parser;
mod resp;
mod store;

pub(crate) use command::Command;
pub(crate) use resp::Resp;
pub(crate) use store::Store;

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

    // Handshake with master server
    if store.config.is_replica() {
        connection::send_connection(&store).await?;
    }

    // Handle incoming requests
    let server_addr = format!("{}:{}", HOST_URL, args.port);
    connection::handle_connection(&server_addr, store).await
}
