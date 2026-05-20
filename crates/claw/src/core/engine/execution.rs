use crate::mcp::McpRegistry;
use crate::skill_store::SkillDefinition;
use serde_json::Value;

/// Execute a parsed tool call and return the result.
/// Tries built-in tools first, then falls back to MCP-discovered tools.
#[tracing::instrument(skip(args, skills, ctx))]
pub(crate) fn execute_tool_call(
    name: &str,
    args: &Value,
    skills: &[SkillDefinition],
    mcp: Option<&McpRegistry>,
    ctx: &crate::tools::ToolContext,
) -> String {
    // Try built-in tools first (including skill tools)
    let registry = crate::tools::ToolRegistry::with_skills(skills);
    if registry.tool_exists(name) {
        return registry.execute(name, args, ctx).unwrap_or_else(|e| e.to_string());
    }

    // Try MCP-discovered tools (from the registry parameter)
    if let Some(mcp) = mcp {
        for (client_idx, tool_def) in &mcp.tools {
            if tool_def.name == name
                && let Some(client) = mcp.clients.get(*client_idx) {
                    return client.call_tool(name, args).unwrap_or_else(|e| format!("MCP 错误: {}", e));
                }
        }
    }

    format!("错误: 未知工具 {}", name)
}
