use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};

pub struct GitTool;

#[async_trait]
impl Tool for GitTool {
    fn name(&self) -> &str { "git" }
    fn description(&self) -> &str { "Execute git operations (status, diff, commit, log)" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "git",
                "description": "Run git commands",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string", "description": "Git subcommand (status, diff, add, commit, log, etc.)"}
                    },
                    "required": ["command"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let cmd = args.get("command").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("command required"))?;
        let output = tokio::process::Command::new("git")
            .args(cmd.split_whitespace())
            .output()
            .await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut result = format!("$ git {}\n", cmd);
        if !stdout.is_empty() { result.push_str(&stdout); }
        if !stderr.is_empty() { result.push_str(&stderr); }
        Ok(result)
    }
}
