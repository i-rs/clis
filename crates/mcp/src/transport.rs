/// MCP (Model Context Protocol) stdio transport for i-rs
///
/// Implements JSON-RPC 2.0 over stdio as specified by the MCP protocol.
/// Reads JSON-RPC requests from stdin, writes JSON-RPC responses to stdout,
/// and uses stderr for diagnostic logging.

use std::io::{self, BufRead, Write};
use serde_json::Value;

/// Represents a JSON-RPC 2.0 request received from stdin.
#[derive(Debug, serde::Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,       // number or string
    pub method: Option<String>,
    pub params: Option<Value>,
}

/// A JSON-RPC 2.0 response to send to stdout.
#[derive(Debug, serde::Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, serde::Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Standard JSON-RPC error codes
pub const METHOD_NOT_FOUND: i32 = -32601;
pub const INVALID_PARAMS: i32 = -32602;
pub const INTERNAL_ERROR: i32 = -32603;

/// Read the next JSON-RPC request from stdin.
/// Returns `Ok(msg)` on success, `Ok(None)` on EOF, `Err` on parse error.
pub fn read_request() -> Result<Option<JsonRpcRequest>, String> {
    let mut line = String::new();
    let n = io::stdin().lock().read_line(&mut line)
        .map_err(|e| format!("Failed to read stdin: {e}"))?;

    if n == 0 {
        return Ok(None); // EOF
    }

    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    match serde_json::from_str::<JsonRpcRequest>(trimmed) {
        Ok(req) => Ok(Some(req)),
        Err(e) => Err(format!("Invalid JSON-RPC request: {e}")),
    }
}

/// Send a JSON-RPC response to stdout.
pub fn send_response(resp: &JsonRpcResponse) {
    let json = serde_json::to_string(resp).unwrap_or_default();
    let mut stdout = io::stdout().lock();
    let _ = writeln!(stdout, "{json}");
    let _ = stdout.flush();
}

/// Log diagnostic information to stderr (visible to the user).
pub fn log(msg: impl std::fmt::Display) {
    let _ = writeln!(io::stderr().lock(), "[i-rs-mcp] {msg}");
}
