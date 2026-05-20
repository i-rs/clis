use crate::error::ClawError;
use crate::semantic::SemanticSearch;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

/// Built-in tool for semantic search across conversation history.
///
/// Uses TF-IDF relevance scoring to find the most relevant messages
/// from past conversations, ranked by semantic relevance to the query.
/// Provides better results than simple keyword search by understanding
/// word importance and context.
pub struct SemanticSearchTool;

impl ClawTool for SemanticSearchTool {
    fn name(&self) -> &str {
        "semantic_search"
    }

    fn description(&self) -> &str {
        "Search past conversations using semantic relevance ranking. \
         Better than simple keyword search — understands which words matter \
         most and returns results ranked by relevance. Use this when the user \
         asks nuanced questions about past conversations or needs to find \
         information that might be phrased differently."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query describing what to find in past conversations"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default: 5, max: 20)",
                    "default": 5,
                    "minimum": 1,
                    "maximum": 20
                }
            },
            "required": ["query"],
            "additionalProperties": false
        })
    }

    fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        let query = args
            .get("query")
            .and_then(|q| q.as_str())
            .unwrap_or("")
            .trim();
        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .unwrap_or(5)
            .clamp(1, 20) as usize;

        if query.is_empty() {
            return Err(ClawError::Validation("Please provide a search query".to_string()));
        }

        let claw_dir = dirs::home_dir()
            .map(|h| h.join(".i-rs-claw").join("claw"))
            .ok_or_else(|| ClawError::NotFound("Cannot determine home directory".to_string()))?;

        let searcher = SemanticSearch::new(claw_dir);
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
                "{}. [会话: {}] (相关度: {:.0}%)\n",
                i + 1,
                sr.result.session_title,
                score_pct
            ));
            output.push_str(&format!("   类型: {}\n", sr.result.message_type));

            // Show excerpt
            let excerpt = if sr.result.excerpt.len() > 300 {
                format!("{}...", &sr.result.excerpt[..297])
            } else {
                sr.result.excerpt.clone()
            };
            output.push_str(&format!("   内容: {}\n", excerpt));

            // Context
            if !sr.result.context_before.is_empty() {
                let before = sr.result.context_before.join(" → ");
                output.push_str(&format!("   前文: {}\n", before));
            }
            if !sr.result.context_after.is_empty() {
                let after = sr.result.context_after.join(" → ");
                output.push_str(&format!("   后文: {}\n", after));
            }
            output.push('\n');
        }

        Ok(output.trim().to_string())
    }
}
