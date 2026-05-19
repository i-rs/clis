use serde_json::Value;

use crate::tools::index;
use crate::tools::ToolContext;

/// Built-in tool that searches the i-rs CLI tool index by keyword.
pub struct SearchToolsTool;

impl super::ClawTool for SearchToolsTool {
    fn name(&self) -> &str {
        "search_tools"
    }

    fn description(&self) -> &str {
        "搜索哪些 i-rs 工具可以处理特定的需求"
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "搜索关键词，描述用户想要完成的任务"
                }
            },
            "required": ["query"]
        })
    }

    fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
        Ok(index::search(query))
    }
}
