use async_trait::async_trait;
use once_cell::sync::Lazy;
use serde_json::{json, Map, Value};

use crate::mcp::McpManager;
use crate::tools::{Tool, ToolResult};

static MCP_MANAGER: Lazy<McpManager> = Lazy::new(McpManager::new);

pub struct McpConnectTool;

#[async_trait]
impl Tool for McpConnectTool {
    fn name(&self) -> &str { "mcp_connect" }
    fn description(&self) -> &str {
        "Connect to an MCP server and register its tools. E.g. 'npx @anthropic-ai/claude-code-mcp'"
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "mcp_connect",
                "description": "Connect to an MCP server and make its tools available.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "server_name": {
                            "type": "string",
                            "description": "A unique name for this MCP server"
                        },
                        "command": {
                            "type": "string",
                            "description": "The command to start the MCP server"
                        },
                        "args": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Command arguments (optional)"
                        }
                    },
                    "required": ["server_name", "command"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let server_name = args.get("server_name").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("server_name required"))?;
        let command = args.get("command").and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("command required"))?;
        let cmd_args: Vec<String> = args.get("args")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        MCP_MANAGER.connect(server_name, command, &cmd_args).await?;
        let tools = MCP_MANAGER.discover_tools(server_name).await?;

        if tools.is_empty() {
            return Ok(format!("Connected to '{}' but no tools discovered", server_name));
        }

        let tool_names: Vec<String> = tools.iter().map(|t| format!("  - {}: {}", t.name, t.description)).collect();
        Ok(format!("Connected to '{}' with {} tools:\n{}", server_name, tools.len(), tool_names.join("\n")))
    }
}

struct McpToolWrapper {
    server_name: String,
    tool_name: String,
    description: String,
}

#[async_trait]
impl Tool for McpToolWrapper {
    fn name(&self) -> &str { &self.tool_name }
    fn description(&self) -> &str { &self.description }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": self.tool_name,
                "description": self.description,
                "parameters": {"type": "object", "properties": {}}
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let result = MCP_MANAGER.call_tool(&self.server_name, &self.tool_name, json!(args)).await?;
        Ok(serde_json::to_string_pretty(&result)?)
    }
}
