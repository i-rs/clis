use super::check_path_async;
use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{Map, Value, json};

pub struct LsTool;

#[async_trait]
impl Tool for LsTool {
    fn name(&self) -> &str {
        "ls"
    }
    fn description(&self) -> &str {
        "List directory contents"
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "ls",
                "description": "List directory contents",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Directory path (default: .)"}
                    },
                    "required": []
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        check_path_async(path).await?;
        let mut entries = tokio::fs::read_dir(path).await?;
        let mut items = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            let ftype = if entry.file_type().await?.is_dir() {
                "dir"
            } else {
                "file"
            };
            items.push(format!(
                "{}  {}",
                ftype,
                entry.file_name().to_string_lossy()
            ));
        }
        items.sort();
        Ok(format!("{}:\n{}", path, items.join("\n")))
    }
}
