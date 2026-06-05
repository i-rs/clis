//! MCP (Model Context Protocol) client using the official rmcp SDK.
//!
//! Wraps `rmcp` (Rust MCP SDK) behind the existing public API for backward compatibility.
//! Provides McpServerConfig, McpClient, McpToolDefinition, and McpRegistry.
//!
//! ## Feature gate
//! When `mcp` feature is disabled, McpClient and McpRegistry become no-op stubs
//! (all operations return empty/error). The data types McpServerConfig and
//! McpToolDefinition are always compiled so config parsing and schema conversion
//! work without the feature.

use crate::error::ClawError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

// ── Feature-gated runtime impl ──

#[cfg(feature = "mcp")]
mod mcp_gated {
    use super::*;
    use rmcp::{
        ServiceExt,
        model::{CallToolRequestParams, CallToolResult, RawContent, ResourceContents},
        service::{RoleClient, RunningService, ServiceError},
        transport::{StreamableHttpClientTransport, TokioChildProcess},
    };
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::process::Command;

    // ── McpClient (single server connection) ──

    /// Client for a single MCP server connection backed by rmcp.
    ///
    /// Thread-safe via internal Arc. Automatically handles the full MCP lifecycle:
    /// initialize handshake, initialized notification, and graceful shutdown.
    ///
    /// Uses a shared tokio runtime passed from McpRegistry to avoid per-connection
    /// runtime creation (see review C-4).
    #[derive(Debug, Clone)]
    pub struct McpClient {
        pub name: String,
        rt: Arc<tokio::runtime::Runtime>,
        service: Arc<RunningService<RoleClient, ()>>,
        config: McpServerConfig,
    }

    impl Drop for McpClient {
        fn drop(&mut self) {
            // Graceful shutdown is handled by rmcp when RunningService is dropped.
            // The shared runtime remains alive (owned by McpRegistry).
        }
    }

    #[allow(dead_code)]
    impl McpClient {
        /// Connect to an MCP server via stdio subprocess, using a shared tokio runtime.
        pub fn connect(
            config: &McpServerConfig,
            rt: &Arc<tokio::runtime::Runtime>,
        ) -> Result<Self, String> {
            let command = config
                .command
                .as_deref()
                .ok_or_else(|| "MCP 服务器缺少 command 配置".to_string())?;
            let args = config.args.as_deref().unwrap_or(&[]);
            let env_vars = config.env.as_deref().unwrap_or(&[]);

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
                ().serve(transport)
                    .await
                    .map_err(|e| format!("MCP 连接 '{}' 失败: {}", config.name, e))
            })?;

            Ok(Self {
                name: config.name.clone(),
                rt: rt.clone(),
                service: Arc::new(service),
                config: config.clone(),
            })
        }

        /// Connect to an MCP server via Streamable HTTP (SSE) transport, using a shared tokio runtime.
        pub fn connect_sse(
            config: &McpServerConfig,
            rt: &Arc<tokio::runtime::Runtime>,
        ) -> Result<Self, String> {
            let url = config
                .url
                .as_deref()
                .ok_or_else(|| "SSE MCP 服务器缺少 url 配置".to_string())?;

            let service = rt.block_on(async {
                let transport = StreamableHttpClientTransport::from_uri(url.to_string());
                ().serve(transport)
                    .await
                    .map_err(|e| format!("MCP SSE 连接 '{}' 失败: {}", config.name, e))
            })?;

            Ok(Self {
                name: config.name.clone(),
                rt: rt.clone(),
                service: Arc::new(service),
                config: config.clone(),
            })
        }

        /// Send the initialize handshake (handled automatically by rmcp).
        /// Kept for API compatibility — this is a no-op.
        pub fn initialize(&self) -> Result<(), String> {
            Ok(())
        }

        /// Health check: try to list tools.
        /// Returns `true` if the MCP server is responsive, `false` otherwise.
        pub fn health_check(&self) -> bool {
            self.rt.block_on(self.service.list_all_tools()).is_ok()
        }

        /// Attempt to reconnect this MCP client using the stored config.
        /// Returns `Ok(new_client)` on success, `Err(e)` if reconnection fails.
        pub fn reconnect(&self) -> Result<McpClient, String> {
            match self.config.transport_type.as_str() {
                "stdio" => McpClient::connect(&self.config, &self.rt),
                "sse" => McpClient::connect_sse(&self.config, &self.rt),
                other => Err(format!("不支持的传输方式 '{}'", other)),
            }
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
        /// Async wrapper for MCP tool calls.
        pub async fn call_tool_async(
            &self,
            tool_name: &str,
            args: &Value,
        ) -> Result<String, ClawError> {
            let json_map = args.as_object().ok_or_else(|| {
                ClawError::Validation("MCP 工具参数必须是 JSON 对象".to_string())
            })?;

            let params =
                CallToolRequestParams::new(tool_name.to_string()).with_arguments(json_map.clone());

            let service = self.service.clone();
            let result: CallToolResult = service
                .call_tool(params)
                .await
                .map_err(|e| ClawError::Mcp(format!("MCP 错误: {}", mcp_service_err(e))))?;

            extract_text_from_call_result(result)
        }
    }

    fn extract_text_from_call_result(result: CallToolResult) -> Result<String, ClawError> {
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
            Ok(serde_json::to_string(&result).unwrap_or_default())
        } else {
            Ok(text_parts.join("\n"))
        }
    }

    /// Format an rmcp ServiceError into a user-friendly message.
    fn mcp_service_err(e: ServiceError) -> String {
        match e {
            ServiceError::UnexpectedResponse => "意外的服务器响应格式".to_string(),
            ServiceError::TransportClosed => "MCP 连接已关闭".to_string(),
            other => format!("{}", other),
        }
    }

    // ── McpRegistry ──

    /// Registry managing all MCP server connections.
    ///
    /// Creates a single shared tokio runtime for all connected MCP clients,
    /// eliminating the per-connection runtime anti-pattern (see review C-4).
    #[derive(Debug, Clone)]
    pub struct McpRegistry {
        /// All connected MCP clients.
        clients: Vec<McpClient>,
        /// All discovered tools (flattened across all servers).
        tools: Vec<(usize, McpToolDefinition)>, // (client_index, tool_def)
        /// O(1) tool name lookup → (client_index, tool_def).
        /// Only the first occurrence of each tool name is kept (first-server wins).
        tool_map: HashMap<String, (usize, McpToolDefinition)>,
        /// Shared tokio runtime for all MCP connections.
        #[allow(dead_code)]
        rt: Arc<tokio::runtime::Runtime>,
        /// Original server configs for reconnection.
        #[allow(dead_code)]
        pub server_configs: Vec<McpServerConfig>,
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
        /// All MCP clients share a single tokio runtime.
        pub fn new(servers: &[McpServerConfig]) -> Self {
            let rt = Arc::new(
                tokio::runtime::Runtime::new().expect("创建 MCP 共享运行时失败"),
            );
            let mut clients = Vec::new();
            let mut tools = Vec::new();
            let mut tool_map = HashMap::new();

            for server in servers.iter() {
                // Skip disabled servers
                if !server.enabled {
                    continue;
                }

                // Dispatch based on transport type
                let client = match server.transport_type.as_str() {
                    "stdio" => match McpClient::connect(server, &rt) {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::warn!("MCP 连接失败 '{}': {}", server.name, e);
                            continue;
                        }
                    },
                    "sse" => match McpClient::connect_sse(server, &rt) {
                        Ok(c) => c,
                        Err(e) => {
                            tracing::warn!("MCP SSE 连接失败 '{}': {}", server.name, e);
                            continue;
                        }
                    },
                    other => {
                        tracing::warn!(
                            "MCP 警告: '{}' 使用了不支持的传输方式 '{}'，已跳过",
                            server.name,
                            other
                        );
                        continue;
                    }
                };

                match client.list_tools() {
                    Ok(tool_defs) => {
                        let client_index = clients.len();
                        for td in tool_defs {
                            // Only the first occurrence of each name is kept
                            tool_map
                                .entry(td.name.clone())
                                .or_insert_with(|| (client_index, td.clone()));
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

            Self {
                clients,
                tools,
                tool_map,
                rt,
                server_configs: servers.to_vec(),
            }
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

        /// Check health of all MCP clients and attempt to reconnect failed ones.
        /// Returns the number of successfully reconnected clients.
        pub fn health_check_and_reconnect(&mut self) -> usize {
            let mut reconnected = 0;

            let failed_indices: Vec<usize> = self
                .clients
                .iter()
                .enumerate()
                .filter(|(_, client)| !client.health_check())
                .map(|(i, _)| i)
                .collect();

            for idx in failed_indices {
                let old_client = &self.clients[idx];
                tracing::warn!(
                    "MCP 客户端 '{}' 连接断开, 尝试重连...",
                    old_client.name
                );
                match old_client.reconnect() {
                    Ok(new_client) => {
                        tracing::info!("MCP 客户端 '{}' 重连成功", new_client.name);
                        match new_client.list_tools() {
                            Ok(new_tools) => {
                                self.tools.retain(|(ci, _)| *ci != idx);
                                // Remove old tool_map entries for this client
                                self.tool_map.retain(|_, (ci, _)| *ci != idx);
                                for td in new_tools {
                                    self.tool_map
                                        .entry(td.name.clone())
                                        .or_insert_with(|| (idx, td.clone()));
                                    self.tools.push((idx, td));
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "MCP 重连后工具发现失败 '{}': {}",
                                    new_client.name,
                                    e
                                );
                            }
                        }
                        self.clients[idx] = new_client;
                        reconnected += 1;
                    }
                    Err(e) => {
                        tracing::warn!("MCP 客户端 '{}' 重连失败: {}", old_client.name, e);
                    }
                }
            }

            reconnected
        }

        /// Get all connected MCP clients.
        pub fn clients(&self) -> &[McpClient] {
            &self.clients
        }

        /// Get all discovered MCP tools (flattened across all servers).
        pub fn tools(&self) -> &[(usize, McpToolDefinition)] {
            &self.tools
        }

        /// Look up a tool by name, returning its (client_index, tool_def).
        #[allow(dead_code)]
        pub fn find_tool(&self, name: &str) -> Option<&(usize, McpToolDefinition)> {
            self.tool_map.get(name)
        }

        /// Get the number of connected MCP clients.
        #[allow(dead_code)]
        pub fn client_count(&self) -> usize {
            self.clients.len()
        }
    }

    #[cfg(test)]
    impl McpRegistry {
        /// Create an empty McpRegistry for testing without creating a tokio runtime on the
        /// current thread. The tokio runtime is created on a separate OS thread to avoid
        /// nested runtime panics.
        pub fn empty_for_test() -> Self {
            static RT: std::sync::OnceLock<Arc<tokio::runtime::Runtime>> =
                std::sync::OnceLock::new();
            let rt = RT
                .get_or_init(|| {
                    Arc::new(
                        std::thread::spawn(|| {
                            tokio::runtime::Runtime::new().expect("创建 MCP 测试运行时失败")
                        })
                        .join()
                        .expect("MCP 测试运行时线程崩溃"),
                    )
                })
                .clone();
            Self {
                clients: Vec::new(),
                tools: Vec::new(),
                tool_map: HashMap::new(),
                rt,
                server_configs: Vec::new(),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::tools::mcp_tools::mcp_schema_to_openai;
        use serde_json::json;

        #[test]
        fn test_mcp_registry_new_empty() {
            let registry = McpRegistry::new(&[]);
            assert!(registry.clients().is_empty(), "空服务器列表不应创建客户端");
            assert!(registry.tools().is_empty(), "空服务器列表不应发现工具");
            assert_eq!(registry.tool_count(), 0);
            assert!(!registry.has_tools());
            assert_eq!(registry.client_count(), 0);
        }

        #[test]
        fn test_mcp_registry_empty_for_test() {
            let registry = McpRegistry::empty_for_test();
            assert!(registry.clients().is_empty());
            assert!(registry.tools().is_empty());
            assert_eq!(registry.tool_count(), 0);
            assert!(!registry.has_tools());
            assert_eq!(registry.client_count(), 0);
        }

        #[test]
        fn test_mcp_schema_conversion() {
            let tool_def = McpToolDefinition {
                server_name: "test-server".to_string(),
                name: "get_weather".to_string(),
                description: "获取天气信息".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "city": {"type": "string", "description": "城市名"}
                    },
                    "required": ["city"]
                }),
            };

            let schema = mcp_schema_to_openai(&tool_def);
            assert_eq!(schema["type"], "function");
            assert_eq!(schema["function"]["name"], "get_weather");
            assert_eq!(schema["function"]["description"], "获取天气信息");
            assert_eq!(schema["function"]["parameters"]["type"], "object");
            assert_eq!(
                schema["function"]["parameters"]["additionalProperties"],
                json!(false)
            );
        }

        #[test]
        fn test_mcp_for_agent_empty() {
            use std::collections::HashSet;
            let agent_config = crate::config::ResolvedAgentConfig {
                agent_id: "test".to_string(),
                provider: crate::providers::ProviderKind::OpenAI,
                api_key: "test-key".to_string(),
                base_url: "http://localhost:9999/v1".to_string(),
                model: "test-model".to_string(),
                enabled_tools: HashSet::new(),
                system_prompt: None,
                mcp_servers: Vec::new(),
                allowed_dirs: Vec::new(),
                capabilities: Vec::new(),
                execution_mode: crate::config::ExecutionMode::React,
            };
            let global_servers: Vec<McpServerConfig> = Vec::new();
            let registry = McpRegistry::for_agent(&agent_config, &global_servers);
            assert!(registry.clients().is_empty(), "无 MCP 服务器时不应创建客户端");
            assert!(registry.tools().is_empty());
        }

        #[test]
        fn test_mcp_for_agent_with_global_servers() {
            // 当 agent 自身没有 mcp_servers 时，应使用 global_servers（空列表）
            use std::collections::HashSet;
            let agent_config = crate::config::ResolvedAgentConfig {
                agent_id: "test".to_string(),
                provider: crate::providers::ProviderKind::OpenAI,
                api_key: "test-key".to_string(),
                base_url: "http://localhost:9999/v1".to_string(),
                model: "test-model".to_string(),
                enabled_tools: HashSet::new(),
                system_prompt: None,
                mcp_servers: Vec::new(),
                allowed_dirs: Vec::new(),
                capabilities: Vec::new(),
                execution_mode: crate::config::ExecutionMode::React,
            };
            // 构造一个带有 enabled=false 的服务器（跳过实际连接）
            let disabled_server = McpServerConfig {
                name: "skip-me".to_string(),
                transport_type: "stdio".to_string(),
                command: Some("nonexistent".to_string()),
                args: None,
                url: None,
                env: None,
                enabled: false,
            };
            let global_servers = vec![disabled_server];
            let registry = McpRegistry::for_agent(&agent_config, &global_servers);
            assert!(registry.clients().is_empty(), "disabled 服务器不应连接");
            assert_eq!(registry.server_configs.len(), 1);
        }

        #[test]
        fn test_mcp_tool_count_and_has_tools() {
            let registry = McpRegistry::empty_for_test();
            assert_eq!(registry.tool_count(), 0);
            assert!(!registry.has_tools());
        }
    }
}

#[cfg(not(feature = "mcp"))]
mod mcp_gated {
    use super::*;
    use std::collections::HashMap;

    // ── McpClient (stub) ──

    /// Stub McpClient used when the `mcp` feature is disabled.
    /// All operations return errors or no-op values.
    #[derive(Debug, Clone)]
    pub struct McpClient {
        #[allow(dead_code)]
        pub name: String,
    }

    #[allow(dead_code)]
    impl McpClient {
        pub fn connect(
            _config: &McpServerConfig,
            _rt: &std::sync::Arc<tokio::runtime::Runtime>,
        ) -> Result<Self, String> {
            Err("MCP 功能未启用 (编译时未开启 mcp feature)".to_string())
        }

        pub fn connect_sse(
            _config: &McpServerConfig,
            _rt: &std::sync::Arc<tokio::runtime::Runtime>,
        ) -> Result<Self, String> {
            Err("MCP 功能未启用 (编译时未开启 mcp feature)".to_string())
        }

        pub fn initialize(&self) -> Result<(), String> {
            Ok(())
        }

        pub fn health_check(&self) -> bool {
            false
        }

        pub fn reconnect(&self) -> Result<McpClient, String> {
            Err("MCP 功能未启用 (编译时未开启 mcp feature)".to_string())
        }

        pub fn list_tools(&self) -> Result<Vec<McpToolDefinition>, String> {
            Ok(Vec::new())
        }

        pub async fn call_tool_async(
            &self,
            _tool_name: &str,
            _args: &Value,
        ) -> Result<String, ClawError> {
            Err(ClawError::Mcp(
                "MCP 功能未启用 (编译时未开启 mcp feature)".to_string(),
            ))
        }
    }

    // ── McpRegistry (stub) ──

    /// Stub McpRegistry used when the `mcp` feature is disabled.
    /// All methods return empty collections.
    #[derive(Debug, Clone)]
    pub struct McpRegistry {
        clients: Vec<McpClient>,
        tools: Vec<(usize, McpToolDefinition)>,
        tool_map: HashMap<String, (usize, McpToolDefinition)>,
        #[allow(dead_code)]
        pub server_configs: Vec<McpServerConfig>,
    }

    impl McpRegistry {
        pub fn for_agent(
            _agent_config: &crate::config::ResolvedAgentConfig,
            _global_servers: &[McpServerConfig],
        ) -> Self {
            Self { clients: Vec::new(), tools: Vec::new(), tool_map: HashMap::new(), server_configs: Vec::new() }
        }

        pub fn new(_servers: &[McpServerConfig]) -> Self {
            Self { clients: Vec::new(), tools: Vec::new(), tool_map: HashMap::new(), server_configs: Vec::new() }
        }

        #[allow(dead_code)]
        pub fn tool_count(&self) -> usize {
            self.tools.len()
        }

        #[allow(dead_code)]
        pub fn has_tools(&self) -> bool {
            !self.tools.is_empty()
        }

        pub fn health_check_and_reconnect(&mut self) -> usize {
            0
        }

        pub fn clients(&self) -> &[McpClient] {
            &self.clients
        }

        pub fn tools(&self) -> &[(usize, McpToolDefinition)] {
            &self.tools
        }

        #[allow(dead_code)]
        pub fn find_tool(&self, name: &str) -> Option<&(usize, McpToolDefinition)> {
            self.tool_map.get(name)
        }

        #[allow(dead_code)]
        pub fn client_count(&self) -> usize {
            self.clients.len()
        }

        #[cfg(test)]
        pub fn empty_for_test() -> Self {
            Self { clients: Vec::new(), tools: Vec::new(), tool_map: HashMap::new(), server_configs: Vec::new() }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::tools::mcp_tools::mcp_schema_to_openai;
        use serde_json::json;

        #[test]
        fn test_mcp_registry_new_empty() {
            let registry = McpRegistry::new(&[]);
            assert!(registry.clients().is_empty());
            assert!(registry.tools().is_empty());
            assert_eq!(registry.tool_count(), 0);
            assert!(!registry.has_tools());
            assert_eq!(registry.client_count(), 0);
        }

        #[test]
        fn test_mcp_registry_empty_for_test() {
            let registry = McpRegistry::empty_for_test();
            assert!(registry.clients().is_empty());
            assert!(registry.tools().is_empty());
            assert_eq!(registry.tool_count(), 0);
            assert!(!registry.has_tools());
            assert_eq!(registry.client_count(), 0);
        }

        #[test]
        fn test_mcp_schema_conversion() {
            let tool_def = McpToolDefinition {
                server_name: "test-server".to_string(),
                name: "get_weather".to_string(),
                description: "获取天气信息".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "city": {"type": "string", "description": "城市名"}
                    },
                    "required": ["city"]
                }),
            };

            let schema = mcp_schema_to_openai(&tool_def);
            assert_eq!(schema["type"], "function");
            assert_eq!(schema["function"]["name"], "get_weather");
            assert_eq!(schema["function"]["description"], "获取天气信息");
            assert_eq!(schema["function"]["parameters"]["type"], "object");
            assert_eq!(
                schema["function"]["parameters"]["additionalProperties"],
                json!(false)
            );
        }

        #[test]
        fn test_mcp_for_agent_empty() {
            use std::collections::HashSet;
            let agent_config = crate::config::ResolvedAgentConfig {
                agent_id: "test".to_string(),
                provider: crate::providers::ProviderKind::OpenAI,
                api_key: "test-key".to_string(),
                base_url: "http://localhost:9999/v1".to_string(),
                model: "test-model".to_string(),
                enabled_tools: HashSet::new(),
                system_prompt: None,
                mcp_servers: Vec::new(),
                allowed_dirs: Vec::new(),
                capabilities: Vec::new(),
                execution_mode: crate::config::ExecutionMode::React,
            };
            let global_servers: Vec<McpServerConfig> = Vec::new();
            let registry = McpRegistry::for_agent(&agent_config, &global_servers);
            assert!(registry.clients().is_empty());
            assert!(registry.tools().is_empty());
        }

        #[test]
        fn test_mcp_for_agent_with_global_servers() {
            use std::collections::HashSet;
            let agent_config = crate::config::ResolvedAgentConfig {
                agent_id: "test".to_string(),
                provider: crate::providers::ProviderKind::OpenAI,
                api_key: "test-key".to_string(),
                base_url: "http://localhost:9999/v1".to_string(),
                model: "test-model".to_string(),
                enabled_tools: HashSet::new(),
                system_prompt: None,
                mcp_servers: Vec::new(),
                allowed_dirs: Vec::new(),
                capabilities: Vec::new(),
                execution_mode: crate::config::ExecutionMode::React,
            };
            let disabled_server = McpServerConfig {
                name: "skip-me".to_string(),
                transport_type: "stdio".to_string(),
                command: Some("nonexistent".to_string()),
                args: None,
                url: None,
                env: None,
                enabled: false,
            };
            let global_servers = vec![disabled_server];
            let registry = McpRegistry::for_agent(&agent_config, &global_servers);
            assert!(registry.clients().is_empty());
            assert_eq!(registry.server_configs.len(), 1);
        }

        #[test]
        fn test_mcp_tool_count_and_has_tools() {
            let registry = McpRegistry::empty_for_test();
            assert_eq!(registry.tool_count(), 0);
            assert!(!registry.has_tools());
        }

        #[test]
        fn test_stub_connect_returns_error() {
            let rt = std::sync::Arc::new(
                tokio::runtime::Runtime::new().unwrap(),
            );
            let config = McpServerConfig {
                name: "test".to_string(),
                transport_type: "stdio".to_string(),
                command: Some("echo".to_string()),
                args: None,
                url: None,
                env: None,
                enabled: true,
            };
            let result = McpClient::connect(&config, &rt);
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.contains("MCP 功能未启用"));
        }

        #[test]
        fn test_stub_health_check() {
            let client = McpClient {
                name: "stub".to_string(),
            };
            assert!(!client.health_check());
        }

        #[test]
        fn test_stub_call_tool_async() {
            let client = McpClient {
                name: "stub".to_string(),
            };
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(client.call_tool_async("test", &json!({})));
            assert!(result.is_err());
        }
    }
}

// ── Re-export ──

pub use mcp_gated::*;
