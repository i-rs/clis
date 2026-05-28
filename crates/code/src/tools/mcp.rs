use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::sync::{Arc, LazyLock, Mutex};

use crate::tools::{Tool, ToolResult};

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

        crate::runtime::mcp_manager().connect(server_name, command, &cmd_args).await?;
        let tools = crate::runtime::mcp_manager().discover_tools(server_name).await?;

        if tools.is_empty() {
            return Ok(format!("Connected to '{}' but no tools discovered", server_name));
        }

        let mut registry = MCP_TOOL_REGISTRY.lock().unwrap();
        for tool_def in &tools {
            let wrapper = Arc::new(McpToolWrapper {
                server_name: server_name.to_string(),
                tool_name: tool_def.name.clone(),
                description: tool_def.description.clone(),
            });
            let key = format!("mcp:{}", tool_def.name);
            registry.insert(key, wrapper);
        }

        let tool_names: Vec<String> = tools.iter().map(|t| format!("  - {}: {}", t.name, t.description)).collect();
        Ok(format!("Connected to '{}' with {} tools (registered as mcp:*)\n{}", server_name, tools.len(), tool_names.join("\n")))
    }
}

static MCP_TOOL_REGISTRY: LazyLock<Mutex<std::collections::HashMap<String, Arc<dyn Tool>>>> =
    LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

pub fn get_mcp_tool(name: &str) -> Option<Arc<dyn Tool>> {
    MCP_TOOL_REGISTRY.lock().ok()?.get(name).cloned()
}

pub fn all_mcp_tools() -> Vec<Arc<dyn Tool>> {
    MCP_TOOL_REGISTRY.lock().map(|r| r.values().cloned().collect()).unwrap_or_default()
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
                "parameters": {
                    "type": "object",
                    "properties": {
                        "input": {
                            "type": "object",
                            "description": "Input parameters for this MCP tool"
                        }
                    },
                    "required": ["input"]
                }
            }
        })
    }
    async fn call(&self, _args: &Map<String, Value>) -> ToolResult {
        let result = crate::runtime::mcp_manager().call_tool(&self.server_name, &self.tool_name, serde_json::Value::Object(_args.clone())).await?;
        Ok(result.to_string())
    }
}
