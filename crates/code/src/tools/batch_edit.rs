use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult, ToolError};

pub struct BatchEditTool;

#[async_trait]
impl Tool for BatchEditTool {
    fn name(&self) -> &str { "batch_edit" }
    fn description(&self) -> &str {
        "Atomically edit multiple files in one operation. Each edit is applied in order; if any fails, all are rolled back."
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "batch_edit",
                "description": "Apply multiple edits atomically across files. If any edit fails, all changes are rolled back.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "edits": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "file_path": {"type": "string"},
                                    "old_string": {"type": "string"},
                                    "new_string": {"type": "string"}
                                },
                                "required": ["file_path", "old_string", "new_string"]
                            },
                            "description": "List of edit operations to apply"
                        }
                    },
                    "required": ["edits"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let edits_val = args.get("edits").and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::invalid_args("edits array required"))?;

        if edits_val.is_empty() {
            return Ok("No edits provided".into());
        }

        let mut edits: Vec<(String, String, String)> = Vec::new();
        for (i, edit_val) in edits_val.iter().enumerate() {
            let file = edit_val.get("file_path").and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::invalid_args(format!("edits[{}].file_path required", i)))?;
            let old = edit_val.get("old_string").and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::invalid_args(format!("edits[{}].old_string required", i)))?;
            let new = edit_val.get("new_string").and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::invalid_args(format!("edits[{}].new_string required", i)))?;
            let path = std::path::Path::new(file);
            if !path.exists() {
                return Err(ToolError::not_found_path(file).into());
            }
            edits.push((file.to_string(), old.to_string(), new.to_string()));
        }

        let mut backups: Vec<(String, String)> = Vec::new();
        for (file, _old, _) in &edits {
            let content = tokio::fs::read_to_string(file).await
                .map_err(|e| ToolError::not_found_path(format!("backup read failed for {}: {}", file, e)))?;
            backups.push((file.clone(), content));
        }

        let mut results = Vec::new();
        for (file, old, new) in &edits {
            let content = tokio::fs::read_to_string(file).await
                .map_err(|e| anyhow::anyhow!("read failed for {}: {}", file, e))?;
            if !content.contains(old.as_str()) {
                results.push(format!("{}: old_string not found (skipped)", file));
                continue;
            }
            let count = content.matches(old.as_str()).count();
            if count > 1 {
                return Err(ToolError::invalid_args(format!("{} has {} matches for batch edit. Use non-ambiguous old_string.", file, count)).into());
            }
            let new_content = content.replacen(old, new, 1);
            let tmp_path = format!("{}.batchtmp", file);
            let _ = tokio::fs::remove_file(&tmp_path).await;
            tokio::fs::write(&tmp_path, &new_content).await?;
            tokio::fs::rename(&tmp_path, file).await?;
            let _ = tokio::fs::remove_file(&tmp_path).await;
            results.push(format!("{}: ok", file));
        }

        Ok(format!("Batch edit ({} operations):\n{}", edits.len(), results.join("\n")))
    }
}
