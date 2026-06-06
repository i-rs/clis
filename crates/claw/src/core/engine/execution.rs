use serde_json::Value;

/// Execute a parsed tool call and return the result.
/// All tools (built-in, skills, MCP) are registered in the shared `ToolRegistry`.
pub(crate) async fn execute_tool_call(
    name: &str,
    args: &Value,
    registry: &i_rs_claw_core::tools::ToolRegistry,
    ctx: &i_rs_claw_core::tools::ToolContext,
) -> String {
    if let Some(tool) = registry.tools.iter().find(|t| t.name() == name) {
        return tool
            .execute(args, ctx)
            .await
            .unwrap_or_else(|e| e.to_string());
    }

    format!("错误: 未知工具 {}", name)
}
