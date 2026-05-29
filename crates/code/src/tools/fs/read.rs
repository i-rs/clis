use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};
use super::check_path_async;

pub struct ReadTool;

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str { "read" }
    fn description(&self) -> &str { "Read a file with line numbers. Use offset and limit for large files." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "read",
                "description": "Read a file with line numbers. Use offset and limit for large files.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string", "description": "Path to the file"},
                        "offset": {"type": "integer", "description": "Line number to start from (1-indexed, default: 1)"},
                        "limit": {"type": "integer", "description": "Max lines to read (default: 200)"}
                    },
                    "required": ["file_path"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args.get("file_path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        check_path_async(path).await?;
        let content = tokio::fs::read_to_string(path).await?;

        let total_lines = content.lines().count();
        let offset = args.get("offset").and_then(|v| v.as_u64()).unwrap_or(1).max(1) as usize;
        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(200) as usize;

        let lines: Vec<&str> = content.lines().collect();
        let end = (offset + limit - 1).min(lines.len());
        let selected: Vec<&str> = lines[(offset - 1)..end].to_vec();

        let max_digits = end.to_string().len();
        let numbered: Vec<String> = selected.iter().enumerate()
            .map(|(i, l)| format!("{:>width$}: {}", offset + i, l, width = max_digits))
            .collect();

        let header = if offset > 1 || end < total_lines {
            format!("{} (lines {}-{} of {})\n```\n{}\n```", path, offset, end, total_lines, numbered.join("\n"))
        } else {
            format!("{}\n```\n{}\n```", path, numbered.join("\n"))
        };
        Ok(header)
    }
}
