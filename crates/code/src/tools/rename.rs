use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};
use super::filesystem::resolve_safe_path;

pub struct RenameTool;

#[async_trait]
impl Tool for RenameTool {
    fn name(&self) -> &str { "rename" }
    fn description(&self) -> &str { "Rename or move a file/directory within the workspace." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "rename",
                "description": "Rename or move a file/directory within the workspace.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "from": {"type": "string", "description": "Current path"},
                        "to": {"type": "string", "description": "New path"}
                    },
                    "required": ["from", "to"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let from = args.get("from").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("from required"))?;
        let to = args.get("to").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("to required"))?;
        let safe_from = resolve_safe_path(from)?;
        let safe_to = resolve_safe_path(to)?;

        if !safe_from.exists() {
            anyhow::bail!("Source does not exist: {}", from);
        }
        if safe_to.exists() {
            anyhow::bail!("Destination already exists: {}", to);
        }

        if let Some(parent) = safe_to.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::rename(&safe_from, &safe_to)?;
        Ok(format!("Renamed {} → {}", from, to))
    }
}
