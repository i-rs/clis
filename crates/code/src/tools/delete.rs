use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};
use super::filesystem::resolve_safe_path;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_name_and_description() {
        let tool = DeleteTool;
        assert_eq!(tool.name(), "delete");
        assert!(tool.description().contains("directory"));
    }

    #[test]
    fn test_delete_schema_has_path_and_recursive() {
        let tool = DeleteTool;
        let schema = tool.schema();
        let params = &schema["function"]["parameters"];
        assert!(params["properties"]["path"].is_object());
        assert!(params["properties"]["recursive"].is_object());
        assert!(params["required"].as_array().unwrap().contains(&json!("path")));
    }

    #[tokio::test]
    async fn test_delete_missing_path_rejected() {
        let tool = DeleteTool;
        let args = Map::new();
        let result = tool.call(&args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("path required"));
    }
}

pub struct DeleteTool;

#[async_trait]
impl Tool for DeleteTool {
    fn name(&self) -> &str { "delete" }
    fn description(&self) -> &str { "Delete a file or directory. Use recursive=true for non-empty directories." }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "delete",
                "description": "Delete a file or directory",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Path to the file or directory to delete"},
                        "recursive": {"type": "boolean", "description": "Recursively delete directory contents (default: false)"}
                    },
                    "required": ["path"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("path required"))?;
        let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);
        let safe_path = resolve_safe_path(path)?;

        if !safe_path.exists() {
            anyhow::bail!("Path does not exist: {}", path);
        }

        if safe_path.is_dir() {
            if recursive {
                std::fs::remove_dir_all(&safe_path)?;
                Ok(format!("Deleted directory {} (recursive)", path))
            } else {
                if safe_path.read_dir()?.next().is_some() {
                    anyhow::bail!("Directory not empty: {}. Use recursive=true to delete.", path);
                }
                std::fs::remove_dir(&safe_path)?;
                Ok(format!("Deleted empty directory {}", path))
            }
        } else {
            std::fs::remove_file(&safe_path)?;
            Ok(format!("Deleted file {}", path))
        }
    }
}
