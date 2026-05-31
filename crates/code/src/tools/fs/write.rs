use super::check_path_async;
use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{Map, Value, json};
use std::path::Path;

pub struct WriteTool;

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str {
        "write"
    }
    fn description(&self) -> &str {
        "Create or overwrite a file"
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "write",
                "description": "Create or overwrite a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "content": {"type": "string"}
                    },
                    "required": ["file_path", "content"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        let content = args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("content required"))?;
        check_path_async(path).await?;

        let old_content = if tokio::fs::metadata(path).await.is_ok() {
            tokio::fs::read_to_string(path).await.unwrap_or_default()
        } else {
            String::new()
        };

        if let Some(parent) = Path::new(path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let tmp_path = format!("{}.tmp", path);
        tokio::fs::write(&tmp_path, content).await?;
        tokio::fs::rename(&tmp_path, path).await?;

        if old_content.is_empty() {
            Ok(format!(
                "Created {} ({} bytes)\n```{}\n```",
                path,
                content.len(),
                content
            ))
        } else {
            let diff = crate::diff::diff_text(&old_content, content);
            Ok(format!(
                "Modified {} (+{} -{})\n```diff\n{}\n```",
                path, diff.lines_added, diff.lines_removed, diff.patch
            ))
        }
    }
}
