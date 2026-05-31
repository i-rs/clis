use super::check_path_async;
use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{Map, Value, json};

pub struct EditTool;

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str {
        "edit"
    }
    fn description(&self) -> &str {
        "Edit a file by replacing text (use replace_all for batch renames)"
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "edit",
                "description": "Edit a file by replacing text",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"},
                        "old_string": {"type": "string", "description": "Text to replace (must match exactly once, unless replace_all)"},
                        "new_string": {"type": "string"},
                        "replace_all": {"type": "boolean", "description": "Replace all occurrences (default: false)"}
                    },
                    "required": ["file_path", "old_string", "new_string"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("file_path required"))?;
        let old = args
            .get("old_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("old_string required"))?;
        let new = args
            .get("new_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("new_string required"))?;
        let replace_all = args
            .get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        check_path_async(path).await?;
        let content = tokio::fs::read_to_string(path).await?;
        let count = content.matches(old).count();
        if count == 0 {
            anyhow::bail!("old_string not found in {}", path);
        }
        if count > 1 && !replace_all {
            anyhow::bail!(
                "Found {} matches for old_string. Use replace_all=true or provide more context.",
                count
            );
        }

        let new_content = if replace_all {
            content.replace(old, new)
        } else {
            content.replacen(old, new, 1)
        };

        let diff = crate::diff::diff_text(&content, &new_content);
        let tmp_path = format!("{}.tmp", path);
        tokio::fs::write(&tmp_path, &new_content).await?;
        tokio::fs::rename(&tmp_path, path).await?;

        Ok(format!(
            "Edited {} (+{} -{}{})\n```diff\n{}\n```",
            path,
            diff.lines_added,
            diff.lines_removed,
            if replace_all {
                format!(" ({} replacements)", count)
            } else {
                String::new()
            },
            diff.patch
        ))
    }
}
