mod command;
mod connection;
mod rdb_parser;
mod resp;
mod store;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Logs from your program will appear here!");
    connection::handle_connection().await
}
