use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

pub struct CallCodeAgentTool;

const CODE_TIMEOUT_SECS: u64 = 300;

#[async_trait::async_trait]
impl ClawTool for CallCodeAgentTool {
    fn name(&self) -> &str {
        "call_code_agent"
    }

    fn description(&self) -> &str {
        "调用代码智能体（i-rs-code）执行编程任务，包括代码编写、重构、调试、审查等。\
         适用于复杂编程任务，返回代码智能体的完整输出。\
         后端可选 i-rs-code 或 opencode。"
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "编程任务描述，应清晰说明要做什么、在什么目录、使用什么语言/框架"
                },
                "backend": {
                    "type": "string",
                    "enum": ["auto", "i-rs-code", "opencode"],
                    "description": "代码智能体后端：auto 自动检测，i-rs-code（默认），opencode"
                },
                "workspace": {
                    "type": "string",
                    "description": "工作目录路径，默认为当前目录"
                }
            },
            "required": ["task"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        let task = args
            .get("task")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClawError::Validation("缺少必要参数: task".to_string()))?;

        let backend = args
            .get("backend")
            .and_then(|v| v.as_str())
            .unwrap_or("auto");

        let workspace = args
            .get("workspace")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty());

        let binary = resolve_binary(backend)?;
        let mut cmd = Command::new(&binary);
        cmd.arg("chat");
        cmd.arg(task);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if let Some(dir) = workspace {
            cmd.current_dir(dir);
        }

        let timeout_secs = ctx.config.cli_timeout_secs.max(CODE_TIMEOUT_SECS);

        tracing::info!(
            binary = %binary,
            task_len = task.len(),
            timeout = timeout_secs,
            "调用代码智能体"
        );

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            cmd.output(),
        )
        .await
        .map_err(|_| {
            ClawError::Timeout(format!("代码智能体执行超时 ({}s)", timeout_secs))
        })?
        .map_err(|e| ClawError::Execution(format!("启动代码智能体失败: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            let err_msg = if stderr.is_empty() { stdout.clone() } else { stderr };
            return Err(ClawError::Execution(format!(
                "代码智能体执行失败 (exit: {}): {}",
                output.status.code().unwrap_or(-1),
                err_msg.chars().take(1000).collect::<String>()
            )));
        }

        let result = if stdout.is_empty() { stderr } else { stdout };
        if result.is_empty() {
            return Err(ClawError::Execution("代码智能体未返回任何输出".to_string()));
        }

        // Strip ANSI escape sequences in case the code agent outputs them
        let clean = strip_ansi_escapes(&result);
        Ok(clean)
    }
}

fn resolve_binary(backend: &str) -> Result<&str, ClawError> {
    match backend {
        "i-rs-code" => Ok("i-rs-code"),
        "opencode" => Ok("opencode"),
        "auto" => {
            // Check if each binary exists by looking up PATH
            if binary_in_path("i-rs-code") {
                Ok("i-rs-code")
            } else if binary_in_path("opencode") {
                Ok("opencode")
            } else {
                Err(ClawError::NotFound(
                    "未找到代码智能体。请安装 i-rs-code (cargo install --path crates/code) \
                     或 opencode (https://github.com/anomalyco/opencode)"
                        .to_string(),
                ))
            }
        }
        _ => Err(ClawError::Validation(format!("不支持的后端: {}", backend))),
    }
}

fn binary_in_path(name: &str) -> bool {
    let path = match std::env::var_os("PATH") {
        Some(p) => p,
        None => return false,
    };
    let name_str = if cfg!(windows) {
        format!("{}.exe", name)
    } else {
        name.to_string()
    };
    std::env::split_paths(&path).any(|dir| dir.join(&name_str).exists())
}

fn strip_ansi_escapes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip CSI sequences: ESC[ <params> <letter>
            if chars.next() == Some('[') {
                while let Some(cc) = chars.next() {
                    if cc.is_ascii_alphabetic() || cc == '~' {
                        break;
                    }
                }
            }
            // Skip other ESC sequences
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_binary_i_rs_code() {
        assert_eq!(resolve_binary("i-rs-code").unwrap(), "i-rs-code");
    }

    #[test]
    fn test_resolve_binary_opencode() {
        assert_eq!(resolve_binary("opencode").unwrap(), "opencode");
    }

    #[test]
    fn test_resolve_binary_unknown() {
        let result = resolve_binary("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_strip_ansi_escapes_none() {
        assert_eq!(strip_ansi_escapes("hello world"), "hello world");
    }

    #[test]
    fn test_strip_ansi_escapes_sgr() {
        let input = "\x1b[31mred\x1b[0m normal";
        assert_eq!(strip_ansi_escapes(input), "red normal");
    }

    #[test]
    fn test_strip_ansi_escapes_csi_sequence() {
        let input = "\x1b[2Kclear line\x1b[Aup";
        assert_eq!(strip_ansi_escapes(input), "clear lineup");
    }

    #[test]
    fn test_strip_ansi_escapes_empty() {
        assert_eq!(strip_ansi_escapes(""), "");
    }
}
