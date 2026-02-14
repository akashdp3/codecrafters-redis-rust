use bytes::Bytes;

pub(crate) async fn execute(args: Vec<String>) -> anyhow::Result<Bytes> {
    let result = match args[0].as_str() {
        "PING" => Bytes::from_static(b"+PONG\r\n"),
        "ECHO" => {
            let value = args.get(1).unwrap();
            Bytes::from(format!("${}\r\n{}\r\n", value.len(), value))
        },
        _ => anyhow::bail!("Unsupported command"),
    };

    Ok(result)
}
