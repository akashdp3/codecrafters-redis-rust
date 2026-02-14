mod connection;
mod resp;
mod command;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Logs from your program will appear here!");
    connection::handle_connection().await
}
