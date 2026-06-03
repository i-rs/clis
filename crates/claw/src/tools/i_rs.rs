use serde_json::Value;

use crate::error::ClawError;
use crate::require_str;
use crate::tools::ToolContext;

/// Built-in tool that executes `i-rs-<tool> <command>` CLI commands directly.
pub struct IrsTool;

#[async_trait::async_trait]
impl super::ClawTool for IrsTool {
    fn name(&self) -> &str {
        "i_rs"
    }

    fn description(&self) -> &str {
        "执行 i-rs CLI 命令来管理用户个人数据。使用标准流程：先用 command=skill args=[\"teach\"] 学习工具的命令参数格式，再按文档调用。日期用 YYYY-MM-DD 格式，需要解析统计数字时 args 末尾加 \"--json\"。"
    }

    fn parameter_schema(&self, enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "tool": {
                    "type": "string",
                    "enum": enabled_cli_tools,
                    "description": "i-rs 工具名称。首次使用不熟悉的工具时，先调用 command=skill args=[\"teach\"] 获取完整命令文档再操作。"
                },
                "command": {
                    "type": "string",
                    "description": "子命令。必须先用 skill teach 确认工具支持哪些命令（add/list/get/delete/update/stats 等），不要猜测。skill 子命令用法：command=\"skill\", args=[\"teach\"]"
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "参数数组。严格按 skill teach 返回的文档中的参数顺序传入，每个参数独立为数组的一个元素，不要合并值。日期统一用 YYYY-MM-DD 格式。需要解析数值结果时末尾追加 \"--json\"。"
                },
                "explanation": {
                    "type": "string",
                    "description": "用中文简要解释当前操作"
                }
            },
            "required": ["tool", "command"]
        })
    }

    async fn execute(&self, args: &Value, ctx: &ToolContext) -> Result<String, ClawError> {
        let tool = require_str!(args, "tool");
        if !ctx.config.i_rs_tools.iter().any(|t| t == tool) {
            return Err(ClawError::Validation(format!(
                "未知的 i-rs 工具: '{}'，可用工具: {}",
                tool,
                ctx.config.i_rs_tools.join(", ")
            )));
        }
        let tool = tool.to_string();
        let cmd = args
            .get("command")
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string();
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

        tokio::task::spawn_blocking(move || execute_cli(&tool, &cmd, &cmd_args, timeout))
            .await
            .unwrap_or_else(|e| Err(ClawError::Execution(format!("CLI 执行任务失败: {}", e))))
    }
}

/// Execute `i-rs-<tool> <command> [args...]` and return the output.
/// Delegates to the shared `run_cli_command` for subprocess execution.
fn execute_cli(
    tool: &str,
    cmd: &str,
    args: &[String],
    cli_timeout_secs: u64,
) -> Result<String, ClawError> {
    let binary = format!("i-rs-{}", tool);
    let cmd_args: Vec<&str> = std::iter::once(cmd)
        .chain(args.iter().map(|s| s.as_str()))
        .collect();
    crate::utils::run_cli_command(&binary, &cmd_args, cli_timeout_secs)
        .map_err(ClawError::Execution)
}
