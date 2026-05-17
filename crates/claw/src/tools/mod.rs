pub mod chart_tool;
pub mod chat_search;
pub mod file_ops;
pub mod i_rs;
pub mod index;
pub mod mcp_tools;
pub mod search_tools;
pub mod semantic_search;
pub mod user_memory;
pub mod vision_tool;
pub mod web_search;

use serde_json::Value;
use std::collections::HashSet;

// ── Backward-compatible re-exports ──
pub use index::{format_index, TOOL_INDEX};

// ── Built-in tool trait ──

/// A built-in tool that the LLM can call.
pub trait ClawTool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    /// Generate the JSON schema for this tool's parameters.
    /// `enabled_cli_tools` is the list of i-rs CLI tools that are enabled
    /// (needed by IrsTool to generate the dynamic `tool.enum`).
    fn parameter_schema(&self, enabled_cli_tools: &[&str]) -> Value;
    /// Execute this tool with the given arguments.
    fn execute(&self, args: &Value) -> Result<String, String>;
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
                Box::new(vision_tool::VisionTool),
                Box::new(web_search::WebSearchTool),
            ],
        }
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
    pub fn execute(&self, name: &str, args: &Value) -> Result<String, String> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.execute(args))
            .unwrap_or_else(|| Err(format!("未知工具: {}", name)))
    }

    /// Check if a built-in tool exists.
    pub fn tool_exists(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name() == name)
    }
}
