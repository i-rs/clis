//! hello-mcp — MCP Server 示例：问候工具
//!
//! 这是一个最简单的 MCP 服务器实现，用于演示如何编写 claw 插件。
//! 功能：接收名字参数，返回个性化问候语。
//!
//! MCP 协议：JSON-RPC 2.0 over stdio
//!   - 从 stdin 逐行读取 JSON-RPC 请求
//!   - 向 stdout 写入 JSON-RPC 响应（每行一个）
//!   - 调试日志写入 stderr
//!
//! 构建： cargo build -p hello-mcp
//! 运行： cargo run -p hello-mcp  （然后通过 echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | cargo run -p hello-mcp 测试）

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

// ── JSON-RPC 类型定义 ──────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    id: Option<serde_json::Value>,
    method: Option<String>,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

// ── MCP 工具定义 ────────────────────────────────────────────────

/// 工具定义（对应 tools/list 响应中的 "tools" 数组）
#[derive(Debug, Serialize)]
struct ToolDefinition {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

/// 返回此服务器支持的工具有哪些
fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![ToolDefinition {
        name: "hello".to_string(),
        description: "向某人打招呼，返回个性化的问候语".to_string(),
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "要问候的人名"
                },
                "language": {
                    "type": "string",
                    "description": "语言 (zh/en/fr/ja)，默认 zh",
                    "enum": ["zh", "en", "fr", "ja"]
                }
            },
            "required": ["name"]
        }),
    }]
}

// ── 工具处理 ────────────────────────────────────────────────────

/// 执行工具调用，返回 JSON 结果
fn handle_tool_call(name: &str, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    match name {
        "hello" => {
            let name_str = args["name"]
                .as_str()
                .ok_or("缺少必填参数 'name'")?;
            let lang = args["language"].as_str().unwrap_or("zh");

            let greeting = match lang {
                "en" => format!("Hello, {}! Nice to meet you!", name_str),
                "fr" => format!("Bonjour, {}! Enchanté!", name_str),
                "ja" => format!("こんにちは、{}さん！はじめまして！", name_str),
                _ => format!("你好，{}！很高兴认识你！", name_str),
            };

            Ok(serde_json::json!({
                "greeting": greeting,
                "name": name_str,
                "language": lang
            }))
        }
        _ => Err(format!("未知工具: {}", name)),
    }
}

// ── 主循环 ──────────────────────────────────────────────────────

fn main() {
    // 注册工具列表
    let tools = get_tool_definitions();

    // 逐行读取 stdin 中的 JSON-RPC 请求
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[hello-mcp] 读取 stdin 错误: {}", e);
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        // 解析请求
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[hello-mcp] JSON 解析错误: {}", e);
                continue;
            }
        };

        let id = req.id.clone();

        // 处理通知（无 id → 不需要响应）
        if id.is_none() {
            continue;
        }

        let method = match req.method.as_deref() {
            Some(m) => m,
            None => {
                send_response(&JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32600,
                        message: "缺少 method".into(),
                    }),
                });
                continue;
            }
        };

        let response = match method {
            "initialize" => Ok(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "hello-mcp",
                    "version": "0.1.0"
                }
            })),
            "tools/list" => Ok(serde_json::json!({ "tools": tools })),
            "tools/call" => {
                let args = req.params.as_ref().and_then(|p| p.get("arguments")).cloned().unwrap_or(serde_json::Value::Null);
                let tool_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let tool_args = args.get("arguments").cloned().unwrap_or(serde_json::Value::Null);
                handle_tool_call(tool_name, &tool_args)
            }
            _ => Err(format!("不支持的方法: {}", method)),
        };

        match response {
            Ok(result) => {
                send_response(&JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id,
                    result: Some(result),
                    error: None,
                });
            }
            Err(msg) => {
                send_response(&JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: msg,
                    }),
                });
            }
        }
    }
}

/// 向 stdout 写入 JSON-RPC 响应
fn send_response(resp: &JsonRpcResponse) {
    if let Ok(json) = serde_json::to_string(resp) {
        let mut out = io::stdout().lock();
        let _ = writeln!(out, "{}", json);
        let _ = out.flush();
    }
}
