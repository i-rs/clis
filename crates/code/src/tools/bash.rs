use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};

pub struct BashTool;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str { "bash" }
    fn description(&self) -> &str { "Execute a shell command" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "bash",
                "description": "Execute a shell command",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string", "description": "Shell command to execute"},
                        "description": {"type": "string", "description": "Brief description of what this command does"}
                    },
                    "required": ["command", "description"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let cmd = args.get("command").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("command required"))?;
        let desc = args.get("description").and_then(|v| v.as_str()).unwrap_or("");

        let blocked_patterns = [
            "rm -rf /",
            "mkfs",
            "dd if=",
            ":(){ :|:& };:",
            "> /dev/sd",
            "chmod -R 777 /",
        ];
        for b in &blocked_patterns {
            if cmd.contains(b) {
                anyhow::bail!("Command contains dangerous pattern: {}", b);
            }
        }

        let output = tokio::process::Command::new("sh")
            .args(["-c", cmd])
            .output()
            .await?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut result = format!("$ {}\n{}\n", desc, cmd);
        if !stdout.is_empty() {
            result.push_str(&format!("stdout:\n{}", stdout));
        }
        if !stderr.is_empty() {
            result.push_str(&format!("stderr:\n{}", stderr));
        }
        if !output.status.success() {
            result.push_str(&format!("exit code: {}", output.status.code().unwrap_or(-1)));
        }
        Ok(result)
    }
}
