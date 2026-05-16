use serde_json::Value;
use std::process::Command;

use crate::tools::index;

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
                    "description": "i-rs 工具名称（i-rs 后的第一个参数）"
                },
                "command": {
                    "type": "string",
                    "description": "子命令。常见的有 add/list/delete/stats。还有 skill（用来学习工具用法）"
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

    fn execute(&self, args: &Value) -> Result<String, String> {
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

/// Execute `i-rs <tool> <command> [args...] --json` and return the output.
fn execute_cli(tool: &str, cmd: &str, args: &[String]) -> Result<String, String> {
    // Append --json for structured output (all i-rs tools support this)
    let mut all_args = Vec::with_capacity(args.len() + 2);
    all_args.push(cmd.to_string());
    all_args.extend_from_slice(args);
    all_args.push("--json".to_string());

    let output = Command::new("i-rs")
        .arg(tool)
        .args(&all_args)
        .output()
        .map_err(|e| format!("执行 i-rs {} {} 失败: {}", tool, cmd, e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            Ok(r#"{"success":true}"#.to_string())
        } else {
            Ok(trimmed.to_string())
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        Err(combined)
    }
}

/// List enabled CLI tool names from TOOL_INDEX filtered by `enabled` set.
pub fn enabled_cli_tool_names(enabled: Option<&std::collections::HashSet<String>>) -> Vec<&'static str> {
    index::TOOL_INDEX
        .iter()
        .map(|(name, _)| *name)
        .filter(|t| index::is_tool_enabled(t, enabled))
        .collect()
}
