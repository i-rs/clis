use crate::error::ClawError;
use serde_json::Value;

use crate::tools::ToolContext;

/// Built-in tool that searches the i-rs CLI tool index by keyword.
pub struct SearchToolsTool;

#[async_trait::async_trait]
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

    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
        if query.is_empty() {
            return Ok("请输入搜索关键词".to_string());
        }

        let lower = query.to_lowercase();
        let mut results: Vec<String> = ctx
            .config
            .i_rs_tool_index
            .iter()
            .filter(|(name, desc)| name.contains(&lower) || desc.to_lowercase().contains(&lower))
            .map(|(name, desc)| {
                if desc.is_empty() {
                    format!("- {}", name)
                } else {
                    format!("- {}: {}", name, desc)
                }
            })
            .collect();

        results.sort();
        if results.is_empty() {
            Ok(format!("未找到与 '{}' 相关的工具", query))
        } else {
            let mut out = format!("找到 {} 个相关工具:\n\n", results.len());
            for r in &results {
                out.push_str(r);
                out.push('\n');
            }
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{ClawTool, ToolContext};
    use serde_json::json;
    use std::collections::HashMap;

    fn test_context() -> ToolContext {
        let mut config = crate::test_helpers::test_config();
        config.i_rs_tool_index = HashMap::from([
            ("i-rs-kv".into(), "Key-value storage".into()),
            ("i-rs-todo".into(), "Todo list management".into()),
            ("i-rs-weight".into(), "Weight tracking".into()),
            ("i-rs-water".into(), "Water intake tracker".into()),
        ]);
        ToolContext {
            config,
            http_client: reqwest::Client::new(),
            delegate_runtime: None,
            user_id: "test".to_string(),
        }
    }

    #[tokio::test]
    async fn test_search_exact_match() {
        let tool = SearchToolsTool;
        let result = tool
            .execute(&json!({"query": "kv"}), &test_context())
            .await
            .unwrap();
        assert!(result.contains("i-rs-kv"), "should find kv tool");
        assert!(result.contains("Key-value"), "should show description");
    }

    #[tokio::test]
    async fn test_search_partial_match() {
        let tool = SearchToolsTool;
        let result = tool
            .execute(&json!({"query": "track"}), &test_context())
            .await
            .unwrap();
        assert!(result.contains("i-rs-weight"), "should match weight");
        assert!(result.contains("i-rs-water"), "should match water");
    }

    #[tokio::test]
    async fn test_search_no_match() {
        let tool = SearchToolsTool;
        let result = tool
            .execute(&json!({"query": "nonexistent"}), &test_context())
            .await
            .unwrap();
        assert!(result.contains("未找到"), "should say not found");
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let tool = SearchToolsTool;
        let result = tool
            .execute(&json!({"query": ""}), &test_context())
            .await
            .unwrap();
        assert!(result.contains("请输入"));
    }
}
