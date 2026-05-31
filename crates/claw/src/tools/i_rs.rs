use serde_json::Value;

use crate::error::ClawError;
use crate::tools::ToolContext;

/// Built-in tool that executes `i-rs-<tool> <command>` CLI commands directly.
pub struct IrsTool;

#[async_trait::async_trait]
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
                    "description": "i-rs 工具名称（如 weight/run/sleep/mood/todo 等）。不熟悉的工具先调用 command=skill args=[\"teach\"] 学习一次，学完即可使用"
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

    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        let tool = args.get("tool").and_then(|t| t.as_str()).unwrap_or("");
        // Validate tool against the whitelist of enabled i-rs CLI tools
        if !tool.is_empty() && !ctx.config.i_rs_tools.iter().any(|t| t == tool) {
            return Err(ClawError::Validation(format!("未知的 i-rs 工具: '{}'，可用工具: {}", tool, ctx.config.i_rs_tools.join(", "))));
        }
        let tool = tool.to_string();
        let cmd = args.get("command").and_then(|c| c.as_str()).unwrap_or("").to_string();
        let cmd_args: Vec<String> = args
            .get("args")
            .and_then(|a| a.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let timeout = ctx.config.cli_timeout_secs;

        tokio::task::spawn_blocking(move || {
            execute_cli(&tool, &cmd, &cmd_args, timeout)
        })
        .await
        .unwrap_or_else(|e| Err(ClawError::Execution(format!("CLI 执行任务失败: {}", e))))
    }
}

/// Execute `i-rs-<tool> <command> [args...]` and return the output.
/// Delegates to the shared `run_cli_command` for subprocess execution.
fn execute_cli(tool: &str, cmd: &str, args: &[String], cli_timeout_secs: u64) -> Result<String, ClawError> {
    let binary = format!("i-rs-{}", tool);
    let cmd_args: Vec<&str> = std::iter::once(cmd).chain(args.iter().map(|s| s.as_str())).collect();
    crate::utils::run_cli_command(&binary, &cmd_args, cli_timeout_secs)
        .map_err(ClawError::Execution)
}


