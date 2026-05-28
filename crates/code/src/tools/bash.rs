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
            "rm -rf /", "rm -rf --no-preserve-root", "rm -rf /*",
            "rm -rf $HOME", "rm -rf ~",
            "mkfs", "dd if=", "dd of=",
            ":(){", "> /dev/sd", "> /dev/nvme", "> /dev/disk",
            "chmod -R 777 /", "chmod 777 /", "chmod 000 ",
            "chown -R", "sudo",
            "wget -O /", "curl -o /", "wget -O /tmp/",
            "mv / ", "cp / ",
            "poweroff", "shutdown", "reboot", "halt",
            "init 0", "init 6",
            "systemctl poweroff", "systemctl reboot", "systemctl halt",
        ];
        for b in &blocked_patterns {
            if cmd.contains(b) {
                anyhow::bail!("Command contains dangerous pattern: {}", b);
            }
        }

        // 强制在 workspace 内执行
        let cwd = std::env::current_dir()?;
        let cmd = format!("cd {} && {}", cwd.to_string_lossy(), cmd);

        let output = tokio::process::Command::new("sh")
            .args(["-c", &cmd])
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

#[cfg(test)]
mod tests {
    use super::*;

    fn blocked_patterns() -> Vec<&'static str> {
        vec![
            "rm -rf /", "rm -rf --no-preserve-root", "rm -rf /*",
            "mkfs", "dd if=", ":(){ :|:& };:", "> /dev/sd",
            "chmod -R 777 /", "chmod 777 /", "sudo ",
            "wget -O /", "curl -o /",
        ]
    }

    #[test]
    fn test_safe_commands_not_blocked() {
        let safe = vec!["cargo check", "python main.py", "git status", "ls -la", "npm test", "echo hello"];
        let patterns = blocked_patterns();
        for cmd in &safe {
            assert!(!patterns.iter().any(|p| cmd.contains(p)), "cmd '{}' should not be blocked", cmd);
        }
    }

    #[test]
    fn test_dangerous_commands_blocked() {
        let patterns = blocked_patterns();
        let dangerous = vec![
            "rm -rf /", "rm -rf --no-preserve-root /", "rm -rf /*",
            "mkfs.ext4 /dev/sda1", "dd if=/dev/zero",
            ":(){ :|:& };:", "echo test > /dev/sda",
            "sudo apt install", "wget -O /tmp/test http://x.com", "curl -o /tmp/test http://x.com",
            "chmod -R 777 /", "chmod 777 /etc",
        ];
        for cmd in &dangerous {
            assert!(patterns.iter().any(|p| cmd.contains(p)), "cmd '{}' should be blocked", cmd);
        }
    }

    #[tokio::test]
    async fn test_bash_echo() {
        let args: serde_json::Map<String, serde_json::Value> = [
            ("command".into(), serde_json::json!("echo hello")),
            ("description".into(), serde_json::json!("test echo")),
        ].into_iter().collect();
        let result = BashTool.call(&args).await.expect("echo should work");
        assert!(result.contains("hello"));
    }

    #[tokio::test]
    async fn test_bash_missing_command() {
        let args: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        let result = BashTool.call(&args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bash_dangerous_blocked() {
        let args: serde_json::Map<String, serde_json::Value> = [
            ("command".into(), serde_json::json!("rm -rf /")),
            ("description".into(), serde_json::json!("test dangerous")),
        ].into_iter().collect();
        let result = BashTool.call(&args).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dangerous"));
    }
}
