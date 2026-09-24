use serde_json::Value;

use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};

/// Unified tool that searches past conversations using either keyword
/// matching (fast, via ConvStore) or semantic ranking (slower, via TF-IDF).
///
/// Replaces the separate `search_conversations` (chat_search) and
/// `semantic_search` tools with a single entry point to reduce LLM confusion.
pub struct SearchConversationsTool;

#[async_trait::async_trait]
impl ClawTool for SearchConversationsTool {
    fn name(&self) -> &str {
        "search_conversations"
    }

    fn description(&self) -> &str {
        "Search past conversations to recall previous discussions. \
         Use 'keyword' method for exact phrase matching (fast), or \
         'semantic' method for meaning-based relevance ranking (slower but \
         finds related ideas even when wording differs)."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query — keywords or a natural-language description of what to find"
                },
                "method": {
                    "type": "string",
                    "enum": ["keyword", "semantic"],
                    "description": "Search method: 'keyword' for fast exact matching, 'semantic' for relevance-based ranking"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 5, max: 20)",
                    "default": 5,
                    "minimum": 1,
                    "maximum": 20
                }
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        let query = args
            .get("query")
            .and_then(|q| q.as_str())
            .unwrap_or("")
            .trim();
        let method = args
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("keyword");
        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(5)
            .clamp(1, 20) as usize;

        if query.is_empty() {
            return Err(ClawError::Validation(
                "Please provide a search query".to_string(),
            ));
        }

        match method {
            "semantic" => search_semantic(query, max_results),
            _ => search_keyword(query),
        }
    }
}

fn search_keyword(query: &str) -> Result<String, ClawError> {
    let claw_dir = crate::utils::claw_dir()
        .ok_or_else(|| ClawError::NotFound("Cannot determine home directory".to_string()))?;

    let store = crate::convstore::ConvStore::for_claw_dir(claw_dir);
    let results = store.search(query, 10);

    if results.is_empty() {
        return Ok("No matching conversations found.".to_string());
    }

    let mut output = format!("Found {} matching conversation(s):\n\n", results.len());
    for (i, r) in results.iter().enumerate() {
        output.push_str(&format!(
            "{}. {}\n",
            i + 1,
            crate::tools::format_search_result(
                &r.session_title,
                &r.message_type,
                &r.excerpt,
                &r.context_before,
                &r.context_after,
                None,
            ),
        ));
    }
    Ok(output.trim().to_string())
}

fn search_semantic(query: &str, max_results: usize) -> Result<String, ClawError> {
    let claw_dir = crate::utils::claw_dir()
        .ok_or_else(|| ClawError::NotFound("Cannot determine home directory".to_string()))?;

    let searcher = crate::semantic::SemanticSearch::new(claw_dir);
    let results = searcher.search(query, max_results);

    if results.is_empty() {
        return Ok("没有找到相关会话内容。".to_string());
    }

    let mut output = format!(
        "找到 {} 条相关结果（按语义相关性排序）：\n\n",
        results.len()
    );
    for (i, sr) in results.iter().enumerate() {
        let score_pct = (sr.score * 100.0).clamp(0.0, 99.0);
        output.push_str(&format!(
            "{}. {}\n",
            i + 1,
            crate::tools::format_search_result(
                &sr.result.session_title,
                &sr.result.message_type,
                &sr.result.excerpt,
                &sr.result.context_before,
                &sr.result.context_after,
                Some(score_pct),
            ),
        ));
    }
    Ok(output.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{ClawTool, ToolContext};
    use serde_json::json;

    #[test]
    fn test_parameter_schema() {
        let tool = SearchConversationsTool;
        let schema = tool.parameter_schema(&[]);
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["query"].is_object());
        assert!(schema["properties"]["method"].is_object());
        assert!(schema["properties"]["max_results"].is_object());
        assert!(
            schema["required"]
                .as_array()
                .unwrap()
                .contains(&json!("query"))
        );
    }

    #[tokio::test]
    async fn test_execute_empty_query() {
        let tool = SearchConversationsTool;
        let ctx = ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: reqwest::Client::new(),
            delegate_runtime: None,
            user_id: "test".to_string(),
        };
        let result = tool.execute(&json!({}), &ctx).await;
        assert!(result.is_err(), "empty query should fail");
        assert!(result.unwrap_err().to_string().contains("query"));
    }

    #[tokio::test]
    async fn test_execute_empty_query_string() {
        let tool = SearchConversationsTool;
        let ctx = ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: reqwest::Client::new(),
            delegate_runtime: None,
            user_id: "test".to_string(),
        };
        let result = tool.execute(&json!({"query": ""}), &ctx).await;
        assert!(result.is_err(), "blank query should fail");
    }
}
