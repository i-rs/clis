use crate::tools::{Tool, ToolResult};
use async_trait::async_trait;
use serde_json::{Map, Value, json};

fn shell_split(input: &str) -> anyhow::Result<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
                chars.next();
            }
            '"' => {
                chars.next();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        break;
                    }
                    current.push(c);
                    chars.next();
                }
            }
            '\'' => {
                chars.next();
                while let Some(&c) = chars.peek() {
                    if c == '\'' {
                        chars.next();
                        break;
                    }
                    current.push(c);
                    chars.next();
                }
            }
            '\\' => {
                chars.next();
                if let Some(c) = chars.next() {
                    current.push(c);
                }
            }
            _ => {
                current.push(ch);
                chars.next();
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    Ok(args)
}

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

        let blocked_subcmds = [
            "push --force",
            "push -f",
            "reset --hard",
            "clean -fd",
            "filter-branch",
        ];
        for pattern in &blocked_subcmds {
            if cmd.contains(pattern) {
                return Err(anyhow::anyhow!(
                    "git command blocked: '{}' contains '{}'. This is dangerous and should be done manually.",
                    cmd,
                    pattern
                ));
            }
        }

        let git_args = shell_split(cmd)?;
        if git_args.is_empty() {
            anyhow::bail!("empty git command");
        }

        let cwd = std::env::current_dir()?;
        let output = tokio::process::Command::new("git")
            .args(&git_args)
            .current_dir(&cwd)
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
