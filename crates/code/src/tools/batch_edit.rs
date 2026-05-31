use crate::tools::{Tool, ToolError, ToolResult};
use async_trait::async_trait;
use serde_json::{Map, Value, json};

pub struct BatchEditTool;

#[async_trait]
impl Tool for BatchEditTool {
    fn name(&self) -> &str {
        "batch_edit"
    }
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
        let edits_val = args
            .get("edits")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::invalid_args("edits array required"))?;

        if edits_val.is_empty() {
            return Ok("No edits provided".into());
        }

        struct Backup {
            path: String,
            original: String,
        }

        let mut edits: Vec<(String, String, String)> = Vec::new();
        for (i, edit_val) in edits_val.iter().enumerate() {
            let file = edit_val
                .get("file_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::invalid_args(format!("edits[{}].file_path required", i))
                })?;
            let old = edit_val
                .get("old_string")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::invalid_args(format!("edits[{}].old_string required", i))
                })?;
            let new = edit_val
                .get("new_string")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolError::invalid_args(format!("edits[{}].new_string required", i))
                })?;
            let path = std::path::Path::new(file);
            if !path.exists() {
                return Err(ToolError::not_found_path(file).into());
            }
            edits.push((file.to_string(), old.to_string(), new.to_string()));
        }

        let mut backups: Vec<Backup> = Vec::new();
        for (file, _, _) in &edits {
            match tokio::fs::read_to_string(file).await {
                Ok(content) => backups.push(Backup {
                    path: file.clone(),
                    original: content,
                }),
                Err(e) => {
                    return Err(ToolError::not_found_path(format!(
                        "backup read failed for {}: {}",
                        file, e
                    ))
                    .into());
                }
            }
        }

        let mut applied_count = 0usize;
        let result = apply_edits(&edits, &mut applied_count).await;

        if result.is_err() {
            for backup in &backups {
                let _ = tokio::fs::write(&backup.path, &backup.original).await;
            }
            let tmp_files: Vec<String> = edits
                .iter()
                .map(|(f, _, _)| format!("{}.batchtmp", f))
                .collect();
            for tf in &tmp_files {
                let _ = tokio::fs::remove_file(tf).await;
            }
        }

        result
    }
}

async fn apply_edits(edits: &[(String, String, String)], applied_count: &mut usize) -> ToolResult {
    let mut results = Vec::new();
    for (file, old, new) in edits {
        let content = tokio::fs::read_to_string(file)
            .await
            .map_err(|e| anyhow::anyhow!("read failed for {}: {}", file, e))?;
        if !content.contains(old.as_str()) {
            results.push(format!("{}: old_string not found (skipped)", file));
            continue;
        }
        let count = content.matches(old.as_str()).count();
        if count > 1 {
            return Err(ToolError::invalid_args(format!(
                "{} has {} matches. Use non-ambiguous old_string.",
                file, count
            ))
            .into());
        }
        let new_content = content.replacen(old, new, 1);
        let tmp_path = format!("{}.batchtmp", file);
        tokio::fs::write(&tmp_path, &new_content).await?;
        tokio::fs::rename(&tmp_path, file).await?;
        let _ = tokio::fs::remove_file(&tmp_path).await;
        *applied_count += 1;
        results.push(format!("{}: ok", file));
    }

    Ok(format!(
        "Batch edit ({} operations):\n{}",
        edits.len(),
        results.join("\n")
    ))
}
