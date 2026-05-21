pub mod chart_tool;
pub mod chat_search;
pub mod delegate;
pub mod file_ops;
pub mod i_rs;
pub mod index;
pub mod mcp_tools;
pub mod search_tools;
pub mod semantic_search;
pub mod skill_tool;
pub mod user_memory;
pub mod vision_tool;
pub mod web_search;

use crate::skill_store::SkillDefinition;
use serde_json::Value;
use std::collections::HashSet;

// ── Backward-compatible re-exports ──
pub use index::{format_index, TOOL_INDEX};

/// Execution context passed to all tools during execution.
/// Contains application state needed for advanced tool operations
/// such as task delegation to sub-agents.
#[derive(Clone)]
pub struct ToolContext {
    pub config: crate::config::Config,
    #[allow(dead_code)]
    pub mcp: crate::mcp::McpRegistry,
    pub http_client: reqwest::Client,
}

// ── Built-in tool trait ──

/// A built-in tool that the LLM can call.
pub trait ClawTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    /// Generate the JSON schema for this tool's parameters.
    /// `enabled_cli_tools` is the list of i-rs CLI tools that are enabled
    /// (needed by IrsTool to generate the dynamic `tool.enum`).
    fn parameter_schema(&self, enabled_cli_tools: &[&str]) -> Value;
    /// Execute this tool with the given arguments and execution context.
    fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, crate::error::ClawError>;
}

// ── Tool registry ──

/// Registry of all tools (built-in + MCP-discovered).
///
/// Adding a new built-in tool:
/// 1. Create `tools/my_tool.rs` with a struct implementing `ClawTool`
/// 2. Add `Box::new(my_tool::MyTool)` to `ToolRegistry::new()`
pub struct ToolRegistry {
    tools: Vec<Box<dyn ClawTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: vec![
                Box::new(chart_tool::ChartTool),
                Box::new(chat_search::ChatSearchTool),
                Box::new(file_ops::FileOpsTool),
                Box::new(i_rs::IrsTool),
                Box::new(search_tools::SearchToolsTool),
                Box::new(semantic_search::SemanticSearchTool),
                Box::new(user_memory::UserMemoryTool),
                Box::new(delegate::DelegateTool),
                Box::new(vision_tool::VisionTool),
                Box::new(web_search::WebSearchTool),
            ],
        }
    }

    /// Create registry with built-in tools + skill tools from SkillStore.
    pub fn with_skills(skills: &[SkillDefinition]) -> Self {
        let mut reg = Self::new();
        for skill in skills {
            reg.tools
                .push(Box::new(skill_tool::SkillTool::new(skill.clone())));
        }
        reg
    }

    /// Create registry with additional MCP-discovered tools.
    #[allow(dead_code)]
    pub fn with_mcp(
        mcp_registry: &crate::mcp::McpRegistry,
    ) -> Self {
        let mut reg = Self::new();
        for (client_idx, tool_def) in &mcp_registry.tools {
            if let Some(client) = mcp_registry.clients.get(*client_idx) {
                reg.tools.push(Box::new(mcp_tools::McpToolWrapper::new(
                    tool_def.clone(),
                    client.clone(),
                )));
            }
        }
        reg
    }

    /// Get tool schemas for OpenAI-compatible chat completion APIs.
    /// Filters CLI tools by `enabled` if provided (empty set = all).
    pub fn enabled_schemas(&self, enabled: Option<&HashSet<String>>) -> Vec<Value> {
        let enabled_cli = i_rs::enabled_cli_tool_names(enabled);
        self.tools
            .iter()
            .map(|tool| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameter_schema(&enabled_cli),
                    }
                })
            })
            .collect()
    }

   /// Execute a tool by name.
    pub fn execute(&self, name: &str, args: &Value, ctx: &ToolContext) -> Result<String, crate::error::ClawError> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.execute(args, ctx))
            .unwrap_or_else(|| Err(crate::error::ClawError::NotFound(format!("未知工具: {}", name))))
    }

    /// Check if a built-in tool exists.
    pub fn tool_exists(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_exists_known() {
        let reg = ToolRegistry::new();
        assert!(reg.tool_exists("i_rs"), "i_rs 应为已知工具");
        assert!(reg.tool_exists("web_search"), "web_search 应为已知工具");
        assert!(reg.tool_exists("chart"), "chart 应为已知工具");
    }

    #[test]
    fn test_tool_exists_unknown() {
        let reg = ToolRegistry::new();
        assert!(!reg.tool_exists("nonexistent"), "不存在的工具应返回 false");
        assert!(!reg.tool_exists(""), "空字符串应返回 false");
    }

    #[test]
    fn test_enabled_schemas_all() {
        let reg = ToolRegistry::new();
        let schemas = reg.enabled_schemas(None);
        assert!(schemas.len() >= 5, "至少应有 5 个内置工具 schema");
        for schema in &schemas {
            assert_eq!(
                schema["type"].as_str(),
                Some("function"),
                "每个 schema 应为 function 类型"
            );
            let func = &schema["function"];
            assert!(func["name"].as_str().is_some(), "每个工具应有 name");
            assert!(func["description"].as_str().is_some(), "每个工具应有 description");
            assert!(func["parameters"].is_object(), "每个工具应有 parameters");
        }
    }

    #[test]
    fn test_tool_registry_with_skills() {
        let skills = vec![crate::skill_store::SkillDefinition {
            name: "test_skill".to_string(),
            description: "A test skill".to_string(),
            parameters: None,
            content: "do something".to_string(),
        }];
        let reg = ToolRegistry::with_skills(&skills);
        assert!(reg.tool_exists("skill_test_skill"), "skill 工具应被注册");
        assert!(reg.tool_exists("i_rs"), "内置工具仍应存在");
    }

    #[test]
    fn test_tool_execute_unknown() {
        let reg = ToolRegistry::new();
        let ctx = ToolContext {
            config: crate::test_helpers::test_config(),
            mcp: crate::mcp::McpRegistry::empty_for_test(),
            http_client: crate::providers::shared_client(),
        };
        let result = reg.execute("不存在", &json!({}), &ctx);
        assert!(result.is_err(), "未知工具应返回错误");
    }
}
