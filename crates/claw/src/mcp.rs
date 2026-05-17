use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

// ── Configuration ──

/// Configuration for a single MCP server connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Display name for this MCP server.
    pub name: String,
    /// Transport type: "stdio" (local process) or "sse" (remote HTTP).
    #[serde(default = "default_transport")]
    pub transport_type: String,
    /// Command to execute (for stdio transport).
    #[serde(default)]
    pub command: Option<String>,
    /// Command arguments (for stdio transport).
    #[serde(default)]
    pub args: Option<Vec<String>>,
    /// URL endpoint (for sse transport).
    #[serde(default)]
    pub url: Option<String>,
    /// Environment variables in KEY=VAL format (for stdio transport).
    #[serde(default)]
    pub env: Option<Vec<String>>,
}

fn default_transport() -> String {
    "stdio".to_string()
}

// ── JSON-RPC 2.0 types ──

fn make_request(id: u64, method: &str, params: Option<Value>) -> Value {
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

fn is_matching_response(line: &str, id: u64) -> Option<Value> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(resp) = serde_json::from_str::<Value>(trimmed) {
        if resp.get("id").and_then(|v| v.as_u64()) == Some(id) {
            return Some(resp);
        }
    }
    None
}

// ── MCP Tool Definition ──

/// A tool discovered from an MCP server.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct McpToolDefinition {
    /// Name of the MCP server that provides this tool.
    #[allow(dead_code)]
    pub server_name: String,
    /// Tool name as defined by the MCP server.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON Schema for tool parameters.
    pub input_schema: Value,
}

// ── MCP Client (single server) ──

/// Internal connection for stdio-based MCP transport.
struct StdioInner {
    stdin: ChildStdinWrapper,
    stdout: BufReader<ChildStdoutWrapper>,
}

/// Internal connection for SSE-based MCP transport.
struct SseInner {
    client: reqwest::blocking::Client,
    url: String,
}

/// Internal transport enum for McpClient.
enum McpClientInner {
    Stdio(StdioInner),
    Sse(SseInner),
}

// Wrappers to handle the fact that ChildStdin/stdout are owned types
// that need to live as long as the McpClientInner.
struct ChildStdinWrapper(std::process::ChildStdin);
struct ChildStdoutWrapper(std::process::ChildStdout);

impl std::io::Read for ChildStdoutWrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

/// Client for a single MCP server connection.
///
/// Supports both stdio (subprocess stdin/stdout) and SSE (HTTP POST)
/// transports. Thread-safe via internal Mutex.
#[derive(Clone)]
pub struct McpClient {
    pub name: String,
    inner: Arc<Mutex<McpClientInner>>,
    next_id: Arc<AtomicU64>,
    /// Whether the client is in a healthy state.
    #[allow(dead_code)]
    pub healthy: bool,
}

impl McpClient {
    /// Connect to an MCP server via stdio subprocess.
    pub fn connect(config: &McpServerConfig) -> Result<Self, String> {
        let command = config
            .command
            .as_deref()
            .ok_or_else(|| "MCP 服务器缺少 command 配置".to_string())?;
        let args = config.args.as_deref().unwrap_or(&[]);
        let env_vars = config.env.as_deref().unwrap_or(&[]);

        let mut cmd = Command::new(command);
        cmd.args(args);
        for var in env_vars {
            if let Some((k, v)) = var.split_once('=') {
                cmd.env(k, v);
            }
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("无法启动 MCP 服务器 '{}': {}", config.name, e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "无法获取 MCP 服务器 stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "无法获取 MCP 服务器 stdout".to_string())?;

        let inner = McpClientInner::Stdio(StdioInner {
            stdin: ChildStdinWrapper(stdin),
            stdout: BufReader::new(ChildStdoutWrapper(stdout)),
        });

        Ok(Self {
            name: config.name.clone(),
            inner: Arc::new(Mutex::new(inner)),
            next_id: Arc::new(AtomicU64::new(1)),
            healthy: true,
        })
    }

    /// Connect to an MCP server via SSE (HTTP POST) transport.
    ///
    /// Uses `reqwest::blocking::Client` for synchronous HTTP requests.
    /// The server URL is taken from the config's `url` field.
    pub fn connect_sse(config: &McpServerConfig) -> Result<Self, String> {
        let url = config
            .url
            .as_deref()
            .ok_or_else(|| "SSE MCP 服务器缺少 url 配置".to_string())?;

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

        let inner = McpClientInner::Sse(SseInner {
            client,
            url: url.to_string(),
        });

        Ok(Self {
            name: config.name.clone(),
            inner: Arc::new(Mutex::new(inner)),
            next_id: Arc::new(AtomicU64::new(1)),
            healthy: true,
        })
    }

    /// Send the initialize handshake to the MCP server.
    /// Must be called before any other method.
    pub fn initialize(&self) -> Result<(), String> {
        let params = serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "i-rs-claw",
                "version": "0.1"
            }
        });
        let result = self.send_request("initialize", Some(params))?;
        // Expect protocolVersion in response
        if result.get("protocolVersion").is_none() {
            return Err(format!("MCP 服务器 '{}' 返回了无效的 initialize 响应", self.name));
        }
        Ok(())
    }

    /// Discover tools from this MCP server.
    pub fn list_tools(&self) -> Result<Vec<McpToolDefinition>, String> {
        let result = self.send_request("tools/list", None)?;
        let tools_array = result
            .get("tools")
            .and_then(|v| v.as_array())
            .ok_or_else(|| format!("MCP 服务器 '{}' 返回了无效的 tools 列表", self.name))?;

        let tools = tools_array
            .iter()
            .map(|t| McpToolDefinition {
                server_name: self.name.clone(),
                name: t
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                description: t
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                input_schema: t
                    .get("inputSchema")
                    .cloned()
                    .or_else(|| t.get("input_schema").cloned())
                    .unwrap_or(serde_json::json!({})),
            })
            .collect();

        Ok(tools)
    }

    /// Call a tool on this MCP server.
    pub fn call_tool(&self, tool_name: &str, args: &Value) -> Result<String, String> {
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": args,
        });
        let result = self.send_request("tools/call", Some(params))?;

        // MCP tool result can contain multiple content items (text, image, etc.)
        // We extract text content items.
        let content = result.get("content").and_then(|v| v.as_array());
        if let Some(items) = content {
            let text_parts: Vec<String> = items
                .iter()
                .filter_map(|item| {
                    if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                        item.get("text").and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else if item.get("type").and_then(|v| v.as_str()) == Some("resource") {
                        // Resource content: extract text from resource blob
                        item.get("resource")
                            .and_then(|r| r.get("text"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            if text_parts.is_empty() {
                return Ok(serde_json::to_string(&result).unwrap_or_default());
            }
            return Ok(text_parts.join("\n"));
        }

        // Fallback: serialize entire result
        Ok(serde_json::to_string(&result).unwrap_or_default())
    }

    /// Send a JSON-RPC request and wait for the matching response.
    /// Dispatches to the appropriate transport (stdio or sse).
    fn send_request(&self, method: &str, params: Option<Value>) -> Result<Value, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = make_request(id, method, params);

        let mut inner = self.inner.lock().map_err(|e| format!("MCP 锁错误: {}", e))?;

        match &mut *inner {
            McpClientInner::Stdio(stdio) => {
                // Write request to stdin
                let request_str =
                    serde_json::to_string(&request).map_err(|e| format!("JSON 序列化失败: {}", e))?;
                writeln!(stdio.stdin.0, "{}", request_str)
                    .map_err(|e| format!("写入 MCP stdin 失败: {}", e))?;
                stdio.stdin.0.flush().map_err(|e| format!("刷新 MCP stdin 失败: {}", e))?;

                // Read responses until we find the matching ID
                let mut line = String::new();
                loop {
                    line.clear();
                    let bytes_read = stdio
                        .stdout
                        .read_line(&mut line)
                        .map_err(|e| format!("读取 MCP stdout 失败: {}", e))?;
                    if bytes_read == 0 {
                        return Err("MCP 服务器连接已关闭".to_string());
                    }

                    if let Some(resp) = is_matching_response(&line, id) {
                        if let Some(error) = resp.get("error") {
                            let msg = error
                                .get("message")
                                .and_then(|v| v.as_str())
                                .unwrap_or("未知错误");
                            return Err(format!("MCP 错误 ({}): {}", method, msg));
                        }
                        return Ok(resp.get("result").cloned().unwrap_or(Value::Null));
                    }
                }
            }
            McpClientInner::Sse(sse) => {
                // Send JSON-RPC via HTTP POST
                let response = sse
                    .client
                    .post(&sse.url)
                    .json(&request)
                    .send()
                    .map_err(|e| format!("SSE POST 失败: {}", e))?;

                if !response.status().is_success() {
                    let status = response.status();
                    let text = response.text().unwrap_or_default();
                    return Err(format!("SSE HTTP 错误 {}: {}", status, text));
                }

                let resp: Value = response
                    .json()
                    .map_err(|e| format!("SSE 响应解析失败: {}", e))?;

                // SSE transport returns response directly (not line-buffered),
                // but JSON-RPC structure is the same.
                if let Some(error) = resp.get("error") {
                    let msg = error
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知错误");
                    return Err(format!("MCP 错误 ({}): {}", method, msg));
                }
                Ok(resp.get("result").cloned().unwrap_or(Value::Null))
            }
        }
    }
}

// ── MCP Registry ──

/// Registry managing all MCP server connections.
#[derive(Clone)]
pub struct McpRegistry {
    /// All connected MCP clients.
    pub clients: Vec<McpClient>,
    /// All discovered tools (flattened across all servers).
    pub tools: Vec<(usize, McpToolDefinition)>, // (client_index, tool_def)
}

impl McpRegistry {
    /// Initialize MCP connections for a specific agent.
    /// Uses agent-specific servers if configured, otherwise falls back to global ones.
    pub fn for_agent(
        agent_config: &crate::config::ResolvedAgentConfig,
        global_servers: &[McpServerConfig],
    ) -> Self {
        let servers: &[McpServerConfig] = if agent_config.mcp_servers.is_empty() {
            global_servers
        } else {
            &agent_config.mcp_servers
        };
        Self::new(servers)
    }

    /// Initialize MCP connections from config.
    /// Failed connections are logged but don't block startup.
    #[allow(dead_code)]
    pub fn new(servers: &[McpServerConfig]) -> Self {
        let mut clients = Vec::new();
        let mut tools = Vec::new();

        for (_idx, server) in servers.iter().enumerate() {
            // Dispatch based on transport type
            let client = match server.transport_type.as_str() {
                "stdio" => match McpClient::connect(server) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("⚠ MCP 连接失败 '{}': {}", server.name, e);
                        continue;
                    }
                },
                "sse" => match McpClient::connect_sse(server) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("⚠ MCP SSE 连接失败 '{}': {}", server.name, e);
                        continue;
                    }
                },
                other => {
                    eprintln!(
                        "⚠ MCP 警告: '{}' 使用了不支持的传输方式 '{}'，已跳过",
                        server.name, other
                    );
                    continue;
                }
            };

            if let Err(e) = client.initialize() {
                eprintln!("⚠ MCP 初始化失败 '{}': {}", server.name, e);
                continue;
            }

            match client.list_tools() {
                Ok(tool_defs) => {
                    let client_index = clients.len();
                    for td in tool_defs {
                        tools.push((client_index, td));
                    }
                    clients.push(client);
                }
                Err(e) => {
                    eprintln!("⚠ MCP 工具发现失败 '{}': {}", server.name, e);
                }
            }
        }

        if !clients.is_empty() {
            eprintln!(
                "✓ MCP: {} 个服务器已连接, {} 个工具已发现",
                clients.len(),
                tools.len()
            );
        }

        Self { clients, tools }
    }

    /// Get the number of discovered MCP tools.
    #[allow(dead_code)]
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// Check if any MCP tools are available.
    #[allow(dead_code)]
    pub fn has_tools(&self) -> bool {
        !self.tools.is_empty()
    }
}
