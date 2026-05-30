use crate::mcp::McpRegistry;
use serde_json::Value;

/// Execute a parsed tool call and return the result.
/// Tries built-in tools first, then falls back to MCP-discovered tools.
/// Uses shared `ToolRegistry` to avoid double construction and
/// `tool_map` HashMap for O(1) MCP lookup.
pub(crate) async fn execute_tool_call(
    name: &str,
    args: &Value,
    registry: &crate::tools::ToolRegistry,
    mcp: Option<&McpRegistry>,
    ctx: &crate::tools::ToolContext,
) -> String {
    // Single pass: try built-in tools, no separate exists check
    if let Some(tool) = registry.tools.iter().find(|t| t.name() == name) {
        return tool.execute(args, ctx).await.unwrap_or_else(|e| e.to_string());
    }

    // O(1) MCP lookup via HashMap
    if let Some(mcp) = mcp {
        if let Some((client_idx, _)) = mcp.tool_map.get(name) {
            if let Some(client) = mcp.clients.get(*client_idx) {
                return client.call_tool_async(name, args).await.unwrap_or_else(|e| format!("MCP 错误: {}", e));
            }
        }
    }

    format!("错误: 未知工具 {}", name)
}
