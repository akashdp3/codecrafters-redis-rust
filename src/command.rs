use anyhow::Context;

use crate::store::Store;

pub(crate) async fn execute(store: &mut Store, args: Vec<String>) -> anyhow::Result<String> {
    let result: String = match args[0].as_str() {
        "PING" => "+PONG\r\n".to_string(),
        "ECHO" => {
            let value = args.get(1).context("Invalid value for ECHO request")?;
            format!("${}\r\n{}\r\n", value.len(), value)
        },
        "GET" => {
            let data_key = args.get(1).context("Invalid key for GET request")?;
            let data_value = store.get(data_key).with_context(|| format!("Failed to read data from store: {}", data_key))?;

            format!("${}\r\n{}\r\n", data_value.len(), data_value)
        },
        "SET" => {
            let data_key = args.get(1).context("Invalid key for PUT request")?;
            let data_value = args.get(2).context("Invalid value for PUT request")?;

            store.set(data_key, data_value).with_context(|| format!("Failed to write data to store: {}", data_key))?;

            format!("+OK\r\n")
        }
        _ => anyhow::bail!("Unsupported command"),
    };

    Ok(result)
}
