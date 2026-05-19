use serde_json::Value;
use std::process::Command;
use std::time::Duration;

use crate::tools::index;
use crate::tools::ToolContext;

/// CLI execution timeout.
const CLI_TIMEOUT: Duration = Duration::from_secs(30);

/// Built-in tool that executes `i-rs <tool> <command>` CLI commands.
pub struct IrsTool;

impl super::ClawTool for IrsTool {
    fn name(&self) -> &str {
        "i_rs"
    }

    fn description(&self) -> &str {
        "执行 i-rs CLI 命令来管理用户的个人数据"
    }

    fn parameter_schema(&self, enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "tool": {
                    "type": "string",
                    "enum": enabled_cli_tools,
                    "description": "i-rs 工具名称（i-rs 后的第一个参数，如 weight/run/sleep/mood/todo 等）。不熟悉的工具先调用 command=skill args=[\"teach\"] 学习一次，学完即可使用"
                },
                "command": {
                    "type": "string",
                    "description": "子命令。常用：add/list/get/delete/update/stats。不熟悉的工具先用 skill teach 学习一次，学完即可使用。skill 子命令常见参数：[\"teach\"]"
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "参数数组，按工具需要的顺序传入。每个参数独立元素，不要合并值"
                },
                "explanation": {
                    "type": "string",
                    "description": "用中文解释当前操作"
                }
            },
            "required": ["tool", "command"]
        })
    }

    fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, String> {
        let tool = args.get("tool").and_then(|t| t.as_str()).unwrap_or("");
        let cmd = args.get("command").and_then(|c| c.as_str()).unwrap_or("");
        let cmd_args: Vec<String> = args
            .get("args")
            .and_then(|a| a.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        execute_cli(tool, cmd, &cmd_args)
    }
}

/// Execute `i-rs <tool> <command> [args...]` and return the output.
fn execute_cli(tool: &str, cmd: &str, args: &[String]) -> Result<String, String> {
    let mut all_args = Vec::with_capacity(args.len() + 1);
    all_args.push(cmd.to_string());
    all_args.extend_from_slice(args);

    let mut child = Command::new("i-rs")
        .arg(tool)
        .args(&all_args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("执行 i-rs {} {} 失败: {}", tool, cmd, e))?;

    let start = std::time::Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output()
                    .map_err(|e| format!("读取命令输出失败: {}", e))?;

                if status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let trimmed = stdout.trim();
                    if trimmed.is_empty() {
                        return Ok(r#"{"success":true}"#.to_string());
                    } else {
                        return Ok(trimmed.to_string());
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let combined = if stderr.trim().is_empty() {
                        stdout.trim().to_string()
                    } else {
                        stderr.trim().to_string()
                    };
                    return Err(combined);
                }
            }
            Ok(None) => {
                if start.elapsed() > CLI_TIMEOUT {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("命令执行超时 (30s): i-rs {} {}", tool, cmd));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(format!("等待命令完成失败: {}", e)),
        }
    }
}

/// List enabled CLI tool names from TOOL_INDEX filtered by `enabled` set.
pub fn enabled_cli_tool_names(enabled: Option<&std::collections::HashSet<String>>) -> Vec<&'static str> {
    index::TOOL_INDEX
        .iter()
        .map(|(name, _, _)| *name)
        .filter(|t| index::is_tool_enabled(t, enabled))
        .collect()
}
