//! calculator-mcp — MCP Server 示例：计算器插件
//!
//! 演示如何定义多个工具 (add/sub/mul/div)、参数校验、错误处理。
//! 构建后可在 claw 中通过插件系统自动加载。

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

// ── JSON-RPC 类型 ──────────────────────────────────────────────

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

// ── 工具定义 ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct ToolDefinition {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

fn get_tool_definitions() -> Vec<ToolDefinition> {
    // 通用的 "两个数字运算" 参数 schema
    let number_operands = serde_json::json!({
        "type": "object",
        "properties": {
            "a": { "type": "number", "description": "第一个数字" },
            "b": { "type": "number", "description": "第二个数字" }
        },
        "required": ["a", "b"]
    });

    vec![
        ToolDefinition {
            name: "add".to_string(),
            description: "加法: a + b".to_string(),
            input_schema: number_operands.clone(),
        },
        ToolDefinition {
            name: "subtract".to_string(),
            description: "减法: a - b".to_string(),
            input_schema: number_operands.clone(),
        },
        ToolDefinition {
            name: "multiply".to_string(),
            description: "乘法: a × b".to_string(),
            input_schema: number_operands.clone(),
        },
        ToolDefinition {
            name: "divide".to_string(),
            description: "除法: a ÷ b（分母不能为 0）".to_string(),
            input_schema: number_operands.clone(),
        },
        ToolDefinition {
            name: "power".to_string(),
            description: "幂运算: a 的 b 次方".to_string(),
            input_schema: number_operands,
        },
        ToolDefinition {
            name: "stats".to_string(),
            description: "计算一组数字的总和、平均值、最大值、最小值".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "numbers": {
                        "type": "array",
                        "items": { "type": "number" },
                        "description": "数字列表"
                    }
                },
                "required": ["numbers"]
            }),
        },
    ]
}

// ── 工具执行 ────────────────────────────────────────────────────

fn handle_tool_call(name: &str, args: &serde_json::Value) -> Result<serde_json::Value, String> {
    match name {
        "add" | "subtract" | "multiply" | "divide" | "power" => {
            let a = args["a"].as_f64().ok_or("参数 'a' 必须是数字")?;
            let b = args["b"].as_f64().ok_or("参数 'b' 必须是数字")?;

            let (result, expression) = match name {
                "add" => (a + b, format!("{} + {} = {}", a, b, a + b)),
                "subtract" => (a - b, format!("{} - {} = {}", a, b, a - b)),
                "multiply" => (a * b, format!("{} × {} = {}", a, b, a * b)),
                "divide" => {
                    if b == 0.0 {
                        return Err("除数不能为 0".into());
                    }
                    (a / b, format!("{} ÷ {} = {}", a, b, a / b))
                }
                "power" => (a.powf(b), format!("{} ^ {} = {}", a, b, a.powf(b))),
                _ => unreachable!(),
            };

            Ok(serde_json::json!({
                "result": result,
                "expression": expression,
                "operation": name
            }))
        }
        "stats" => {
            let nums: Vec<f64> = args["numbers"]
                .as_array()
                .ok_or("参数 'numbers' 必须是数组")?
                .iter()
                .map(|v| v.as_f64().ok_or_else(|| format!("元素 {} 不是数字", v)))
                .collect::<Result<Vec<_>, _>>()?;

            if nums.is_empty() {
                return Err("数字列表不能为空".into());
            }

            let sum: f64 = nums.iter().sum();
            let count = nums.len();
            let avg = sum / count as f64;
            let max = nums.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min = nums.iter().cloned().fold(f64::INFINITY, f64::min);

            Ok(serde_json::json!({
                "sum": sum,
                "average": avg,
                "max": max,
                "min": min,
                "count": count,
            }))
        }
        _ => Err(format!("未知工具: {}", name)),
    }
}

// ── MCP 主循环 ─────────────────────────────────────────────────

fn main() {
    let tools = get_tool_definitions();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[calculator-mcp] 读取错误: {}", e);
                break;
            }
        };

        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[calculator-mcp] JSON 解析错误: {}", e);
                continue;
            }
        };

        let id = req.id.clone();
        if id.is_none() {
            continue;
        }

        let method = match req.method.as_deref() {
            Some(m) => m,
            None => {
                send_err(&id, -32600, "缺少 method");
                continue;
            }
        };

        let response = match method {
            "initialize" => Ok(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "calculator-mcp", "version": "0.1.0" }
            })),
            "tools/list" => Ok(serde_json::json!({ "tools": tools })),
            "tools/call" => {
                let args = req.params.as_ref()
                    .and_then(|p| p.get("arguments"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let tool_name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let tool_args = args.get("arguments").cloned().unwrap_or(serde_json::Value::Null);
                handle_tool_call(tool_name, &tool_args)
            }
            _ => Err(format!("不支持的方法: {}", method)),
        };

        match response {
            Ok(result) => send_ok(&id, result),
            Err(msg) => send_err(&id, -32603, &msg),
        }
    }
}

fn send_ok(id: &Option<serde_json::Value>, result: serde_json::Value) {
    send(&JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id: id.clone(),
        result: Some(result),
        error: None,
    });
}

fn send_err(id: &Option<serde_json::Value>, code: i32, msg: &str) {
    send(&JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id: id.clone(),
        result: None,
        error: Some(JsonRpcError { code, message: msg.into() }),
    });
}

fn send(resp: &JsonRpcResponse) {
    if let Ok(json) = serde_json::to_string(resp) {
        let mut out = io::stdout().lock();
        let _ = writeln!(out, "{}", json);
        let _ = out.flush();
    }
}
