use std::collections::HashMap;
use std::process::Stdio;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

pub struct McpConnection {
    child: Option<Child>,
    stdin: Option<tokio::process::ChildStdin>,
    reader: Option<BufReader<tokio::process::ChildStdout>>,
    next_id: u32,
    tools: Vec<McpToolDef>,
}

#[derive(Debug, Clone)]
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl McpConnection {
    pub async fn connect(command: &str, args: &[&str]) -> anyhow::Result<Self> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = cmd.spawn()?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("no stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("no stdout"))?;
        let reader = BufReader::new(stdout);

        let mut conn = Self {
            child: Some(child),
            stdin: Some(stdin),
            reader: Some(reader),
            next_id: 1,
            tools: Vec::new(),
        };

        conn.send_request("initialize", serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "i-rs-code", "version": "0.1" },
        })).await?;

        let tools_result: Value = conn.send_request("tools/list", serde_json::json!({})).await?;
        if let Some(tools_array) = tools_result.get("tools").and_then(|v| v.as_array()) {
            for tool_val in tools_array {
                let name = tool_val.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let description = tool_val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let input_schema = tool_val.get("inputSchema").cloned().unwrap_or(Value::Null);
                conn.tools.push(McpToolDef { name, description, input_schema });
            }
        }

        Ok(conn)
    }

    pub fn server_count(&self) -> usize {
        0 // async, can't lock from sync context; shown via status instead
    }

    pub fn discovered_tools(&self) -> &[McpToolDef] {
        &self.tools
    }

    pub async fn call_tool(&mut self, name: &str, args: Value) -> anyhow::Result<Value> {
        self.send_request("tools/call", serde_json::json!({
            "name": name,
            "arguments": args,
        })).await
    }

    async fn send_request(&mut self, method: &str, params: Value) -> anyhow::Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        self.write_message(&request).await?;

        let reader = self.reader.as_mut().ok_or_else(|| anyhow::anyhow!("no reader"))?;
        loop {
            let msg = read_mcp_message(reader).await?;
            if msg["id"].as_u64() == Some(id as u64) {
                if let Some(result) = msg.get("result") {
                    return Ok(result.clone());
                }
                if let Some(error) = msg.get("error") {
                    let code = error.get("code").and_then(|v| v.as_u64()).unwrap_or(0);
                    anyhow::bail!("MCP error: {}: {}",
                        code, error.get("message").and_then(|v| v.as_str()).unwrap_or("unknown"));
                }
                anyhow::bail!("invalid MCP response");
            }
        }
    }

    async fn write_message(&mut self, msg: &Value) -> anyhow::Result<()> {
        let content = serde_json::to_string(msg)?;
        let header = format!("Content-Length: {}\r\n\r\n", content.len());
        if let Some(stdin) = &mut self.stdin {
            stdin.write_all(header.as_bytes()).await?;
            stdin.write_all(content.as_bytes()).await?;
            stdin.flush().await?;
        }
        Ok(())
    }
}

impl Drop for McpConnection {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
        }
    }
}

async fn read_mcp_message(reader: &mut BufReader<tokio::process::ChildStdout>) -> anyhow::Result<Value> {
    let mut content_len: Option<usize> = None;
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            anyhow::bail!("MCP server closed");
        }
        let trimmed = line.trim();
        if trimmed.is_empty() { break; }
        if let Some(len_str) = trimmed.strip_prefix("Content-Length: ") {
            content_len = Some(len_str.parse()?);
        }
    }
    let len = content_len.ok_or_else(|| anyhow::anyhow!("MCP: missing Content-Length"))?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf).await?;
    let content = String::from_utf8(buf)?;
    Ok(serde_json::from_str(&content)?)
}

pub struct McpManager {
    connections: Mutex<HashMap<String, McpConnection>>,
}

impl McpManager {
    pub fn new() -> Self {
        Self { connections: Mutex::new(HashMap::new()) }
    }

    pub async fn connect(&self, name: &str, command: &str, args: &[String]) -> anyhow::Result<()> {
        let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let conn = McpConnection::connect(command, &args_refs).await?;
        let mut map = self.connections.lock().await;
        map.insert(name.to_string(), conn);
        Ok(())
    }

    pub async fn discover_tools(&self, name: &str) -> anyhow::Result<Vec<McpToolDef>> {
        let map = self.connections.lock().await;
        if let Some(conn) = map.get(name) {
            Ok(conn.discovered_tools().to_vec())
        } else {
            Err(anyhow::anyhow!("no MCP connection: {}", name))
        }
    }

    pub async fn call_tool(&self, server_name: &str, tool_name: &str, args: Value) -> anyhow::Result<Value> {
        let mut map = self.connections.lock().await;
        if let Some(conn) = map.get_mut(server_name) {
            conn.call_tool(tool_name, args).await
        } else {
            Err(anyhow::anyhow!("no MCP connection: {}", server_name))
        }
    }
}
