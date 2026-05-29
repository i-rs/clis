//! MCP (Model Context Protocol) client using the official rmcp SDK.
//!
//! Replaces the previous hand-rolled JSON-RPC implementation with rmcp v1.7.0,
//! providing standard protocol handling, proper error recovery, and automatic
//! lifecycle management (initialize handshake, initialized notification, shutdown).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    service::{RoleClient, RunningService},
    transport::TokioChildProcess,
};
use serde_json::Value;
use tokio::process::Command;

/// A tool definition discovered from an MCP server.
#[derive(Debug, Clone)]
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

type McpService = Arc<RunningService<RoleClient, ()>>;

/// Manages MCP server connections using the rmcp SDK.
///
/// Thread-safe: connections are stored behind Arc, and the connection map
/// is guarded by a std Mutex (fast path — only the HashMap lookup is locked,
/// not the service calls themselves).
pub struct McpManager {
    connections: Mutex<HashMap<String, McpService>>,
}

impl McpManager {
    pub fn new() -> Self {
        Self { connections: Mutex::new(HashMap::new()) }
    }

    /// Connect to an MCP server via stdio subprocess.
    /// The rmcp SDK handles the initialize handshake automatically.
    pub async fn connect(&self, name: &str, command: &str, args: &[String]) -> anyhow::Result<()> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        let transport = TokioChildProcess::new(cmd)
            .map_err(|e| anyhow::anyhow!("Failed to spawn MCP process '{}': {}", name, e))?;

        let service: RunningService<RoleClient, ()> = ()
            .serve(transport)
            .await
            .map_err(|e| anyhow::anyhow!("MCP connection '{}' failed: {}", name, e))?;

        let mut map = self.connections.lock().unwrap();
        map.insert(name.to_string(), Arc::new(service));
        Ok(())
    }

    /// Discover tools from an MCP server by querying tools/list.
    pub async fn discover_tools(&self, name: &str) -> anyhow::Result<Vec<McpToolDef>> {
        let service = {
            let map = self.connections.lock().unwrap();
            map.get(name).cloned()
                .ok_or_else(|| anyhow::anyhow!("no MCP connection: {}", name))?
        };

        let tools = service
            .list_all_tools()
            .await
            .map_err(|e| anyhow::anyhow!("MCP tool discovery failed: {}", e))?;

        Ok(tools
            .into_iter()
            .map(|t| McpToolDef {
                name: t.name.to_string(),
                description: t.description.unwrap_or_default().to_string(),
                input_schema: Value::Object((*t.input_schema).clone()),
            })
            .collect())
    }

    /// Call a tool on an MCP server.
    pub async fn call_tool(&self, server_name: &str, tool_name: &str, args: Value) -> anyhow::Result<Value> {
        let service = {
            let map = self.connections.lock().unwrap();
            map.get(server_name).cloned()
                .ok_or_else(|| anyhow::anyhow!("no MCP connection: {}", server_name))?
        };

        let json_map = args
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("MCP tool arguments must be a JSON object"))?
            .clone();

        let params = CallToolRequestParams::new(tool_name.to_string())
            .with_arguments(json_map);

        let result = service
            .call_tool(params)
            .await
            .map_err(|e| anyhow::anyhow!("MCP tool call failed: {}", e))?;

        // Serialize the CallToolResult to a JSON Value for backward compatibility
        Ok(serde_json::to_value(&result)
            .unwrap_or_else(|_| Value::String(serde_json::to_string(&result).unwrap_or_default())))
    }
}
