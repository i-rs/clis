use serde_json::Value;
use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};

/// Built-in tool that searches past conversation sessions by keyword.
///
/// Uses ConvStore to scan all JSONL session files for matching messages,
/// returning excerpts with surrounding context. This enables the LLM to
/// recall information from previous conversations.
pub struct ChatSearchTool;

impl ClawTool for ChatSearchTool {
    fn name(&self) -> &str {
        "search_conversations"
    }

    fn description(&self) -> &str {
        "Search all past conversations for messages containing specific keywords. \
         Use this when the user asks about something you discussed before, or \
         when they want to find information from previous conversations."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Keyword or phrase to search for in past conversations"
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
        if query.is_empty() {
            return Err(ClawError::Validation("Please provide a search query".to_string()));
        }

        let claw_dir = dirs::home_dir()
            .map(|h| h.join(".i-rs").join("claw"))
            .ok_or_else(|| ClawError::NotFound("Cannot determine home directory".to_string()))?;

        let store = crate::convstore::ConvStore::new(claw_dir);
        let results = store.search(query, 10);

        if results.is_empty() {
            return Ok("No matching conversations found.".to_string());
        }

        let mut output =
            format!("Found {} matching conversation(s):\n\n", results.len());

        for (i, r) in results.iter().enumerate() {
            output.push_str(&format!(
                "{}. [会话: {}]\n",
                i + 1,
                r.session_title
            ));
            output.push_str(&format!("   类型: {}\n", r.message_type));
            output.push_str(&format!("   内容: {}\n", r.excerpt));

            if !r.context_before.is_empty() {
                let before = r.context_before.join(" → ");
                output.push_str(&format!("   前文: {}\n", before));
            }
            if !r.context_after.is_empty() {
                let after = r.context_after.join(" → ");
                output.push_str(&format!("   后文: {}\n", after));
            }
            output.push('\n');
        }

        Ok(output.trim().to_string())
    }
}
