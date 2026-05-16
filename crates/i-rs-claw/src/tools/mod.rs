pub mod i_rs;
pub mod index;
pub mod search_tools;
pub mod user_memory;

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

/// Registry of all built-in tools.
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
                Box::new(i_rs::IrsTool),
                Box::new(search_tools::SearchToolsTool),
                Box::new(user_memory::UserMemoryTool),
            ],
        }
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

    /// Execute a built-in tool by name.
    pub fn execute(&self, name: &str, args: &Value) -> Result<String, String> {
        self.tools
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.execute(args))
            .unwrap_or_else(|| Err(format!("未知工具: {}", name)))
    }
}
