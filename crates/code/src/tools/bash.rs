use async_trait::async_trait;
use serde_json::{json, Value, Map};
use crate::tools::{Tool, ToolResult};

/// Maximum output size per stream (64KB)
const MAX_OUTPUT_BYTES: usize = 64 * 1024;
/// Default command timeout in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 120;

/// Truncate a string if it exceeds MAX_OUTPUT_BYTES, preserving UTF-8 boundaries.
fn truncate_output(s: &str) -> String {
    if s.len() <= MAX_OUTPUT_BYTES {
        return s.to_string();
    }
    let mut end = MAX_OUTPUT_BYTES;
    // Find last newline before the limit to avoid breaking mid-line
    if let Some(pos) = s[..end].rfind('\n') {
        end = pos + 1;
    }
    let truncated = &s[..end];
    format!(
        "{}\n\n[output truncated: {} bytes omitted, total {} bytes]",
        truncated,
        s.len() - truncated.len(),
        s.len()
    )
}

/// Whitelisted command prefixes (safe to execute)
const ALLOWED_PREFIXES: &[&str] = &[
    // Build tools
    "cargo", "rustc", "rustup",
    // Version control
    "git",
    // File operations
    "ls", "cat", "head", "tail", "wc", "find", "mkdir", "cp", "mv",
    "rm", "chmod", "chown",
    // Text processing
    "grep", "sed", "awk", "sort", "uniq", "diff", "file",
    // Shell builtins
    "echo", "printf", "pwd", "which", "test", "true", "false",
    "cd", "export", "unset",
    // Date/time
    "date", "cal",
    // Languages
    "python", "python3", "pip", "pip3", "node", "npm", "npx", "bun",
    "deno", "go", "javac", "java", "mvn", "gradle", "make", "cmake",
    "scala", "scalac", "kotlinc", "kotlin", "swift", "swiftc",
    "ruby", "gem", "bundle", "rake", "rails",
    "lua", "luac", "zig", "zig build",
    "gcc", "g++", "cc", "c++", "clang", "clang++",
    // Container/infra
    "docker", "docker-compose", "kubectl", "terraform", "ansible",
    // Network
    "curl", "wget", "ssh", "scp", "rsync",
    // Archives
    "tar", "gzip", "gunzip", "zip", "unzip", "xz", "bzip2",
    // Data formats
    "jq", "yq",
    // System info
    "env", "uname", "whoami", "hostname", "df", "du", "free",
    "top", "ps", "lsof", "strace", "ldd",
    // Misc
    "base64", "md5sum", "sha256sum", "sha512sum",
    "sh", "bash", "zsh",
    "sleep", "wait", "kill",
];

/// Commands that require a TTY (interactive) — always blocked
const BLOCKED_INTERACTIVE: &[&str] = &[
    "vim", "vi", "nano", "emacs", "ed",
    "less", "more", "bat",
    "ssh", "telnet", "nc", "ncat", "netcat",
    "screen", "tmux",
    "top", "htop",
];

/// Dangerous patterns that are always blocked regardless of whitelist
const BLOCKED_PATTERNS: &[&str] = &[
    "rm -rf /", "rm -rf /*", "rm -rf ~",
    "mkfs", "dd of=/dev/", "dd if=/dev/zero",
    "> /dev/sd", "> /dev/nvme", "> /dev/disk",
    "chmod -R 777 /", "chmod 777 /",
    "poweroff", "shutdown", "reboot", "halt",
    "init 0", "init 6",
    "systemctl poweroff", "systemctl reboot", "systemctl halt",
    ":(){",  // fork bomb
];

/// Extract the first command name from a shell command string.
fn extract_command(cmd: &str) -> &str {
    let trimmed = cmd.trim_start();
    // Handle simple commands like "cargo check"
    if let Some(space) = trimmed.find(char::is_whitespace) {
        &trimmed[..space]
    } else {
        trimmed
    }
}

/// Check if a command is allowed by the whitelist.
fn is_command_allowed(cmd: &str) -> Result<(), String> {
    let command = extract_command(cmd);

    // Always block dangerous patterns
    for pattern in BLOCKED_PATTERNS {
        if cmd.contains(pattern) {
            return Err(format!("Dangerous pattern blocked: {}", pattern));
        }
    }

    // Block interactive commands
    for blocked in BLOCKED_INTERACTIVE {
        if command == *blocked {
            return Err(format!("Interactive command not allowed: {}", blocked));
        }
    }

    // Block sudo and any absolute path to a system binary not in whitelist
    if command.starts_with("sudo") || command.starts_with("doas") {
        return Err(format!("Privilege escalation blocked: {}", command));
    }

    // Check whitelist — strip ./ prefix for matching
    let mut command_stripped = command.strip_prefix("./").unwrap_or(command);
    // Also strip leading ./  for compound commands
    while command_stripped.starts_with("./") {
        command_stripped = &command_stripped[2..];
    }
    // If the command has no recognizable prefix, allow it if it's a local script
    let is_local_script = command.starts_with("./");
    for allowed in ALLOWED_PREFIXES {
        if command_stripped == *allowed || command_stripped.starts_with(&format!("{} ", allowed)) {
            return Ok(());
        }
    }
    // Allow local scripts (./script.sh or relative paths)
    if is_local_script {
        return Ok(());
    }

    // Allow absolute paths to whitelisted binaries (e.g. /usr/bin/python3)
    if command.starts_with('/') {
        let basename = command.rsplit('/').next().unwrap_or(command);
        for allowed in ALLOWED_PREFIXES {
            if basename == *allowed || basename.starts_with(&format!("{} ", allowed)) {
                return Ok(());
            }
        }
        return Err(format!("Command not in whitelist: {}", command));
    }

    Err(format!("Command not in whitelist: {}", command))
}

pub struct BashTool;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str { "bash" }
    fn description(&self) -> &str { "Execute a shell command (whitelist-enforced, with timeout)" }
    fn schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "bash",
                "description": "Execute a shell command. Only whitelisted commands are allowed.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string", "description": "Shell command to execute"},
                        "description": {"type": "string", "description": "Brief description of what this command does"},
                        "timeout_secs": {"type": "integer", "description": "Timeout in seconds (default: 120, max: 600)"}
                    },
                    "required": ["command", "description"]
                }
            }
        })
    }
    async fn call(&self, args: &Map<String, Value>) -> ToolResult {
        let cmd = args.get("command").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("command required"))?;
        let desc = args.get("description").and_then(|v| v.as_str()).ok_or_else(|| anyhow::anyhow!("description required"))?;
        let timeout_secs = args.get("timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
            .clamp(1, 600);

        // Whitelist check
        if let Err(reason) = is_command_allowed(cmd) {
            anyhow::bail!("{}", reason);
        }

        // Force execution within workspace
        let cwd = std::env::current_dir()?;
        let full_cmd = format!("cd {} && {}", cwd.to_string_lossy(), cmd);

        let output = match tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            tokio::process::Command::new("sh")
                .args(["-c", &full_cmd])
                .output()
        ).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                return Ok(format!("$ {}\nCommand failed: {}\n", desc, e));
            }
            Err(_) => {
                return Ok(format!(
                    "$ {}\nCommand timed out after {}s\n[blocked: command killed]",
                    desc, timeout_secs
                ));
            }
        };

        let stdout = truncate_output(&String::from_utf8_lossy(&output.stdout));
        let stderr = truncate_output(&String::from_utf8_lossy(&output.stderr));

        let mut result = format!("$ {}\n{}\n", desc, full_cmd);
        if !output.stdout.is_empty() {
            result.push_str(&format!("stdout:\n{}", stdout));
        }
        if !output.stderr.is_empty() {
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

    #[test]
    fn test_allowed_commands_pass() {
        let safe = vec![
            "cargo check", "cargo test -- --test-threads=1", "cargo clippy -- -D warnings",
            "git status", "git diff", "git log --oneline -5",
            "ls -la", "cat README.md", "grep pattern src/",
            "python main.py", "node index.js", "npm test", "go build ./...",
            "echo hello", "pwd", "which cargo",
            "docker ps", "kubectl get pods", "curl -sL https://example.com",
            "tar xzf file.tar.gz", "jq '.name' data.json",
            "make build", "cmake ..",
            "sh script.sh", "bash -c 'echo ok'",
            "./build.sh",
        ];
        for cmd in &safe {
            assert!(is_command_allowed(cmd).is_ok(), "cmd '{}' should be allowed", cmd);
        }
    }

    #[test]
    fn test_blocked_dangerous_patterns() {
        let dangerous = vec![
            "rm -rf /", "rm -rf /*", "rm -rf ~",
            "mkfs.ext4 /dev/sda1",
            "dd of=/dev/sda",
            "echo x > /dev/sda",
            "chmod -R 777 /",
            "poweroff", "shutdown -h now",
            ":(){ :|:& };:",
        ];
        for cmd in &dangerous {
            assert!(is_command_allowed(cmd).is_err(), "cmd '{}' should be blocked", cmd);
        }
    }

    #[test]
    fn test_blocked_interactive() {
        let interactive = vec!["vim file.txt", "vi", "nano file", "less README", "ssh user@host"];
        for cmd in &interactive {
            assert!(is_command_allowed(cmd).is_err(), "cmd '{}' should be blocked (interactive)", cmd);
        }
    }

    #[test]
    fn test_blocked_privilege_escalation() {
        let cmds = vec!["sudo rm -rf /", "sudo apt install vim", "doas something"];
        for cmd in &cmds {
            assert!(is_command_allowed(cmd).is_err(), "cmd '{}' should be blocked (sudo)", cmd);
        }
    }

    #[test]
    fn test_blocked_not_in_whitelist() {
        let cmds = vec!["nmap localhost", "nc -l 8080", " exploit "];
        for cmd in &cmds {
            assert!(is_command_allowed(cmd).is_err(), "cmd '{}' should be blocked (not in whitelist)", cmd);
        }
    }

    #[test]
    fn test_allowed_absolute_path() {
        assert!(is_command_allowed("/usr/bin/python3 script.py").is_ok());
        assert!(is_command_allowed("/usr/local/bin/cargo check").is_ok());
    }

    #[test]
    fn test_blocked_absolute_path_unknown() {
        assert!(is_command_allowed("/usr/bin/nmap localhost").is_err());
    }

    #[test]
    fn test_extract_command() {
        assert_eq!(extract_command("cargo check"), "cargo");
        assert_eq!(extract_command("  git status"), "git");
        assert_eq!(extract_command("ls"), "ls");
        assert_eq!(extract_command("./script.sh"), "./script.sh");
    }

    #[test]
    fn test_truncate_output_short() {
        let short = "hello world";
        assert_eq!(truncate_output(short), short);
    }

    #[test]
    fn test_truncate_output_exact_limit() {
        let s = "a".repeat(MAX_OUTPUT_BYTES);
        assert_eq!(truncate_output(&s).len(), s.len());
    }

    #[test]
    fn test_truncate_output_over_limit() {
        let limit = super::MAX_OUTPUT_BYTES;
        let s = "a".repeat(limit + 1000);
        let result = super::truncate_output(&s);
        assert!(result.len() < s.len());
        let needle = "[output truncated]";
        assert_eq!(result.contains(needle), true, "missing needle in result");
    }

    #[test]
    fn test_truncate_output_utf8_safe() {
        // Create a string that would break at a non-UTF8 boundary if we just sliced
        let s = "hello".repeat(MAX_OUTPUT_BYTES / 5 + 100);
        let result = truncate_output(&s);
        assert!(result.contains("[output truncated]"));
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
    async fn test_bash_missing_command_arg() {
        let args: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        let result = BashTool.call(&args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bash_missing_description_arg() {
        let args: serde_json::Map<String, serde_json::Value> = [
            ("command".into(), serde_json::json!("echo hello")),
        ].into_iter().collect();
        let result = BashTool.call(&args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bash_whitelist_blocked() {
        let args: serde_json::Map<String, serde_json::Value> = [
            ("command".into(), serde_json::json!("nmap localhost")),
            ("description".into(), serde_json::json!("test blocked")),
        ].into_iter().collect();
        let result = BashTool.call(&args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bash_timeout() {
        let args: serde_json::Map<String, serde_json::Value> = [
            ("command".into(), serde_json::json!("sleep 30")),
            ("description".into(), serde_json::json!("test timeout")),
            ("timeout_secs".into(), serde_json::json!(1)),
        ].into_iter().collect();
        let result = BashTool.call(&args).await.expect("timeout should return Ok");
        assert!(result.contains("timed out"));
    }
}
