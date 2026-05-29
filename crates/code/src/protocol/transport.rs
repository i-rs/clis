use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

/// Send a JSON event to claw (write to stdout)
pub fn send_event(event: &impl serde::Serialize) -> anyhow::Result<()> {
    let line = serde_json::to_string(event)?;
    println!("{}", line);
    Ok(())
}

/// Read a JSON line from stdin (blocking in spawned task)
pub async fn read_line() -> anyhow::Result<String> {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    if let Some(line) = lines.next_line().await? {
        Ok(line)
    } else {
        anyhow::bail!("stdin closed")
    }
}

/// Read a JSON-RPC message using Content-Length headers (shared by LSP and MCP).
pub async fn read_content_length_message(
    reader: &mut BufReader<tokio::process::ChildStdout>,
    server_name: &str,
) -> anyhow::Result<serde_json::Value> {
    let mut content_len: Option<usize> = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            anyhow::bail!("{} server closed connection", server_name);
        }
        let trimmed = line.trim();
        if trimmed.is_empty() { break; }
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            content_len = Some(len_str.parse()?);
        }
    }
    let len = content_len.ok_or_else(|| anyhow::anyhow!("missing Content-Length header"))?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    let content = String::from_utf8(buf)?;
    Ok(serde_json::from_str(&content)?)
}
