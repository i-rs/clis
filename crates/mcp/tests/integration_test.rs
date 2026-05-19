//! Integration tests for i-rs-mcp server.
//!
//! Spawns the actual binary as a child process and communicates via
//! newline-delimited JSON-RPC 2.0 messages over stdio.
//!
//! NOTE: The binary must be built before running these tests.
//! `cargo test -p i-rs-mcp` handles this automatically.

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn binary_path() -> String {
    let profile = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    format!(
        "{}/../../target/{}/i-rs-mcp",
        std::env!("CARGO_MANIFEST_DIR"),
        profile
    )
}

fn jsonrpc(id: u64, method: &str, params: Option<Value>) -> Value {
    let mut req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
    });
    if let Some(p) = params {
        req["params"] = p;
    }
    req
}

async fn send(
    stdin: &mut (impl AsyncWriteExt + Unpin),
    msg: &Value,
) {
    let json = serde_json::to_string(msg).unwrap();
    stdin.write_all(json.as_bytes()).await.unwrap();
    stdin.write_all(b"\n").await.unwrap();
}

async fn recv(
    reader: &mut (impl AsyncBufReadExt + Unpin),
    line: &mut String,
) -> Value {
    line.clear();
    reader.read_line(line).await.unwrap();
    if line.is_empty() {
        panic!("unexpected EOF from child process — server may have crashed");
    }
    serde_json::from_str(line.trim()).unwrap()
}

fn assert_result(resp: &Value, step: &str) {
    assert!(
        resp.get("result").is_some(),
        "{step}: expected 'result' in response, got error: {:?}",
        resp.get("error")
    );
}

// ── Tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_initialize_success() {
    let binary = binary_path();
    let mut child = tokio::process::Command::new(&binary)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn i-rs-mcp");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    // Initialize request
    let req = jsonrpc(
        1,
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"},
        })),
    );
    send(&mut stdin, &req).await;
    let resp = recv(&mut reader, &mut line).await;
    assert_result(&resp, "initialize");

    // Verify server info
    let server_info = &resp["result"]["serverInfo"];
    assert_eq!(server_info["name"], "i-rs-mcp");
    assert!(!server_info["version"].as_str().unwrap_or("").is_empty());

    // Verify capabilities
    assert!(
        resp["result"]["capabilities"]["tools"].is_object(),
        "expected capabilities.tools to be an object"
    );

    child.kill().await.unwrap();
    child.wait().await.unwrap();
}

#[tokio::test]
async fn test_tools_list_count() {
    let mut child = tokio::process::Command::new(&binary_path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn i-rs-mcp");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    // Initialize
    let req = jsonrpc(
        1,
        "initialize",
        Some(serde_json::json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"},
        })),
    );
    send(&mut stdin, &req).await;
    let _ = recv(&mut reader, &mut line).await;

    // Initialized notification
    send(
        &mut stdin,
        &serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    )
    .await;

    // tools/list
    let req = jsonrpc(2, "tools/list", None);
    send(&mut stdin, &req).await;
    let resp = recv(&mut reader, &mut line).await;
    assert_result(&resp, "tools/list");

    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 260, "expected 260 tools (65 stores × 4 ops)");

    let first = &tools[0];
    assert!(first["name"].as_str().unwrap_or("").ends_with("_list"));
    assert!(
        first.get("inputSchema").or_else(|| first.get("input_schema")).is_some_and(|v| v.is_object()),
        "expected tool to have inputSchema object"
    );

    child.kill().await.unwrap();
    child.wait().await.unwrap();
}

#[tokio::test]
async fn test_tools_call_weight_list() {
    let binary = binary_path();
    let mut child = tokio::process::Command::new(&binary)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn i-rs-mcp");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    // Initialize
    send(
        &mut stdin,
        &jsonrpc(
            1,
            "initialize",
            Some(serde_json::json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"},
            })),
        ),
    )
    .await;
    let _ = recv(&mut reader, &mut line).await;

    // Initialized notification
    send(
        &mut stdin,
        &serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    )
    .await;

    // tools/call weight_list
    send(
        &mut stdin,
        &jsonrpc(
            2,
            "tools/call",
            Some(serde_json::json!({"name": "weight_list", "arguments": {}})),
        ),
    )
    .await;
    let resp = recv(&mut reader, &mut line).await;
    assert_result(&resp, "weight_list");

    let content_text = &resp["result"]["content"][0]["text"];
    let data: Value = serde_json::from_str(content_text.as_str().unwrap()).unwrap();
    assert!(data["count"].is_number());
    assert!(data["entries"].is_array());

    child.kill().await.unwrap();
    child.wait().await.unwrap();
}

#[tokio::test]
async fn test_tools_call_weight_add_and_delete() {
    let mut child = tokio::process::Command::new(&binary_path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn i-rs-mcp");

    let mut stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();

    // Initialize
    send(
        &mut stdin,
        &jsonrpc(
            1,
            "initialize",
            Some(serde_json::json!({
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"},
            })),
        ),
    )
    .await;
    let _ = recv(&mut reader, &mut line).await;

    // Initialized notification
    send(
        &mut stdin,
        &serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    )
    .await;

    // Add a weight entry
    send(
        &mut stdin,
        &jsonrpc(
            2,
            "tools/call",
            Some(serde_json::json!({
                "name": "weight_add",
                "arguments": {"weight": 75.5, "date": "2026-05-19"},
            })),
        ),
    )
    .await;
    let resp = recv(&mut reader, &mut line).await;
    assert_result(&resp, "weight_add");
    let add_content = &resp["result"]["content"][0]["text"];
    let add_data: Value = serde_json::from_str(add_content.as_str().unwrap()).unwrap();
    let added_date = add_data["date"].as_str().unwrap().to_string();
    assert_eq!(add_data["weight"], 75.5);

    // Verify by listing
    send(
        &mut stdin,
        &jsonrpc(
            3,
            "tools/call",
            Some(serde_json::json!({"name": "weight_list", "arguments": {}})),
        ),
    )
    .await;
    let resp = recv(&mut reader, &mut line).await;
    assert_result(&resp, "weight_list after add");

    // Delete the entry by date (weight store uses NaiveDate keys)
    send(
        &mut stdin,
        &jsonrpc(
            4,
            "tools/call",
            Some(serde_json::json!({
                "name": "weight_delete",
                "arguments": {"id": &added_date},
            })),
        ),
    )
    .await;
    let resp = recv(&mut reader, &mut line).await;
    assert_result(&resp, "weight_delete");

    // Verify deletion — weight_list should not contain the entry
    send(
        &mut stdin,
        &jsonrpc(
            5,
            "tools/call",
            Some(serde_json::json!({"name": "weight_list", "arguments": {}})),
        ),
    )
    .await;
    let resp = recv(&mut reader, &mut line).await;
    assert_result(&resp, "weight_list after delete");
    let content_text = &resp["result"]["content"][0]["text"];
    let list_data: Value = serde_json::from_str(content_text.as_str().unwrap()).unwrap();
    let entries = list_data["entries"].as_array().unwrap();
    assert!(
        !entries.iter().any(|e| e["date"] == added_date),
        "expected deleted entry to no longer appear in list"
    );

    child.kill().await.unwrap();
    child.wait().await.unwrap();
}
