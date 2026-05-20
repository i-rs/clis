//! MCP (Model Context Protocol) client using the official rmcp SDK.
//!
//! Wraps `rmcp` (Rust MCP SDK) behind the existing public API for backward compatibility.
//! Provides McpServerConfig, McpClient, McpToolDefinition, and McpRegistry.
use crate::error::ClawError;

use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, CallToolResult, RawContent, ResourceContents},
    service::{RoleClient, RunningService, ServiceError},
    transport::{TokioChildProcess, StreamableHttpClientTransport},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::process::Command;

// ── Helper ──

fn default_enabled() -> bool {
    true
}

fn default_transport() -> String {
    "stdio".to_string()
}

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
    /// Whether this MCP server is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

// ── Tool Definition ──

/// A tool discovered from an MCP server.
#[derive(Debug, Clone)]
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

// ── McpClient (single server connection) ──

/// Client for a single MCP server connection backed by rmcp.
///
/// Thread-safe via internal Arc. Automatically handles the full MCP lifecycle:
/// initialize handshake, initialized notification, and graceful shutdown.
#[derive(Debug, Clone)]
pub struct McpClient {
    pub name: String,
    rt: Arc<tokio::runtime::Runtime>,
    service: Arc<RunningService<RoleClient, ()>>,
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

        // Create a tokio runtime for this connection
        let rt =
            tokio::runtime::Runtime::new().map_err(|e| format!("创建 MCP 运行时失败: {}", e))?;

        let service = rt.block_on(async {
            let mut cmd = Command::new(command);
            cmd.args(args);
            for var in env_vars {
                if let Some((k, v)) = var.split_once('=') {
                    cmd.env(k, v);
                }
            }
            let transport = TokioChildProcess::new(cmd)
                .map_err(|e| format!("创建 MCP 子进程失败: {}", e))?;
            ()
                .serve(transport)
                .await
                .map_err(|e| format!("MCP 连接 '{}' 失败: {}", config.name, e))
        })?;

        Ok(Self {
            name: config.name.clone(),
            rt: Arc::new(rt),
            service: Arc::new(service),
        })
    }

    /// Connect to an MCP server via Streamable HTTP (SSE) transport.
    pub fn connect_sse(config: &McpServerConfig) -> Result<Self, String> {
        let url = config
            .url
            .as_deref()
            .ok_or_else(|| "SSE MCP 服务器缺少 url 配置".to_string())?;

        let rt =
            tokio::runtime::Runtime::new().map_err(|e| format!("创建 MCP 运行时失败: {}", e))?;

        let service = rt.block_on(async {
            let transport = StreamableHttpClientTransport::from_uri(url.to_string());
            ()
                .serve(transport)
                .await
                .map_err(|e| format!("MCP SSE 连接 '{}' 失败: {}", config.name, e))
        })?;

        Ok(Self {
            name: config.name.clone(),
            rt: Arc::new(rt),
            service: Arc::new(service),
        })
    }

    /// Send the initialize handshake (handled automatically by rmcp).
    /// Kept for API compatibility — this is a no-op.
    pub fn initialize(&self) -> Result<(), String> {
        Ok(())
    }

    /// Discover tools from this MCP server.
    pub fn list_tools(&self) -> Result<Vec<McpToolDefinition>, String> {
        let tools = self
            .rt
            .block_on(self.service.list_all_tools())
            .map_err(|e| format!("MCP 工具发现失败: {}", e))?;

        Ok(tools
            .into_iter()
            .map(|t| {
                let input_schema = t.input_schema;
                McpToolDefinition {
                    server_name: self.name.clone(),
                    name: t.name.to_string(),
                    description: t.description.unwrap_or_default().to_string(),
                    input_schema: Value::Object((*input_schema).clone()),
                }
            })
            .collect())
    }

    /// Call a tool on this MCP server.
    pub fn call_tool(&self, tool_name: &str, args: &Value) -> Result<String, ClawError> {
        let json_map = args
            .as_object()
            .ok_or_else(|| ClawError::Validation("MCP 工具参数必须是 JSON 对象".to_string()))?;

        let params = CallToolRequestParams::new(tool_name.to_string())
            .with_arguments(json_map.clone());

        let result: CallToolResult = self
            .rt
            .block_on(self.service.call_tool(params))
            .map_err(|e| ClawError::Mcp(format!("MCP 错误: {}", mcp_service_err(e))))?;

        // Extract text from content items
        let text_parts: Vec<String> = result
            .content
            .iter()
            .filter_map(|c| match &c.raw {
                RawContent::Text(t) => Some(t.text.clone()),
                RawContent::Resource(r) => match &r.resource {
                    ResourceContents::TextResourceContents { text, .. } => Some(text.clone()),
                    _ => None,
                },
                _ => None,
            })
            .collect();

        if text_parts.is_empty() {
            // Fallback: serialize entire result
            Ok(serde_json::to_string(&result).unwrap_or_default())
        } else {
            Ok(text_parts.join("\n"))
        }
    }
}

/// Format an rmcp ServiceError into a user-friendly message.
fn mcp_service_err(e: ServiceError) -> String {
    match e {
        ServiceError::UnexpectedResponse => "意外的服务器响应格式".to_string(),
        ServiceError::TransportClosed => {
            "MCP 连接已关闭".to_string()
        }
        other => format!("{}", other),
    }
}

// ── McpRegistry ──

/// Registry managing all MCP server connections.
#[derive(Debug, Clone)]
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
    pub fn new(servers: &[McpServerConfig]) -> Self {
        let mut clients = Vec::new();
        let mut tools = Vec::new();

        for server in servers.iter() {
            // Skip disabled servers
            if !server.enabled {
                continue;
            }

            // Dispatch based on transport type
            let client = match server.transport_type.as_str() {
                "stdio" => match McpClient::connect(server) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("MCP 连接失败 '{}': {}", server.name, e);
                        continue;
                    }
                },
                "sse" => match McpClient::connect_sse(server) {
                    Ok(c) => c,
                    Err(e) => {
                        tracing::warn!("MCP SSE 连接失败 '{}': {}", server.name, e);
                        continue;
                    }
                },
                other => {
                    tracing::warn!(
                        "MCP 警告: '{}' 使用了不支持的传输方式 '{}'，已跳过",
                        server.name, other
                    );
                    continue;
                }
            };

            // Note: rmcp's serve() already handles initialize + initialized handshake.
            // The initialize() call on the client is a no-op for API compatibility.

            match client.list_tools() {
                Ok(tool_defs) => {
                    let client_index = clients.len();
                    for td in tool_defs {
                        tools.push((client_index, td));
                    }
                    clients.push(client);
                }
                Err(e) => {
                    tracing::warn!("MCP 工具发现失败 '{}': {}", server.name, e);
                }
            }
        }

        if !clients.is_empty() {
            tracing::info!(
                "MCP: {} 个服务器已连接, {} 个工具已发现",
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
