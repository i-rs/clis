use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};

pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str { "glob" }
    fn description(&self) -> &str { "Find files matching a glob pattern" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "glob",
                "description": "Find files matching a glob pattern",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string", "description": "Glob pattern e.g. **/*.rs"},
                        "path": {"type": "string", "description": "Root directory (default: .)"}
                    },
                    "required": ["pattern"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let pattern = args.get("pattern").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("pattern required"))?.to_string();
        let root = args.get("path").and_then(|v| v.as_str()).unwrap_or(".").to_string();
        tokio::task::spawn_blocking(move || {
            let glob = globset::Glob::new(&pattern)?;
            let matcher = glob.compile_matcher();
            let mut results = Vec::new();
            for entry in ignore::WalkBuilder::new(&root).max_depth(Some(10)).build().flatten() {
                if entry.file_type().map(|t| t.is_file()).unwrap_or(false) && matcher.is_match(entry.path()) {
                    results.push(entry.path().to_string_lossy().to_string());
                }
            }
            results.sort();
            if results.is_empty() {
                Ok("No files found".into())
            } else {
                Ok(format!("Found {} files:\n{}", results.len(), results.join("\n")))
            }
        }).await?
    }
}
