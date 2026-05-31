use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{Map, Value, json};

pub struct GitTool;

#[async_trait]
impl Tool for GitTool {
    fn name(&self) -> &str {
        "git"
    }
    fn description(&self) -> &str {
        "Execute git operations (status, diff, add, commit, log, stash, checkout, stash_pop, etc.)"
    }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "git",
                "description": "Run git commands in the current working directory.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string", "description": "Git subcommand, e.g. 'status --short', 'add -A', 'commit -m \"msg\"', 'log --oneline -10', 'stash push -m wip', 'stash pop', 'checkout -- <file>', 'diff --cached', 'branch -a', 'show HEAD:path/to/file', 'checkout -b <branch>', 'merge --no-ff'"}
                    },
                    "required": ["command"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let cmd = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("command required"))?;
        let blocked = [
            "push --force",
            "push -f",
            "reset --hard",
            "clean -fd",
            "filter-branch",
        ];
        for pattern in &blocked {
            if cmd.contains(pattern) {
                return Err(anyhow::anyhow!(
                    "git command blocked: '{}' contains '{}'. This is dangerous and should be done manually.",
                    cmd,
                    pattern
                ));
            }
        }
        let cwd = std::env::current_dir()?;
        let output = tokio::process::Command::new("sh")
            .args([
                "-c",
                &format!("cd {} && git {}", cwd.to_string_lossy(), cmd),
            ])
            .output()
            .await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut result = format!("$ git {}\n", cmd);
        if !stdout.is_empty() {
            result.push_str(&stdout);
        }
        if !stderr.is_empty() && !output.status.success() {
            result.push_str(&stderr);
        }
        if !output.status.success() {
            result.push_str(&format!(
                "exit code: {}",
                output.status.code().unwrap_or(-1)
            ));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_non_empty() {
        let tool = GitTool;
        assert!(!tool.name().is_empty());
    }

    #[test]
    fn test_description_non_empty() {
        let tool = GitTool;
        assert!(!tool.description().is_empty());
    }
}
