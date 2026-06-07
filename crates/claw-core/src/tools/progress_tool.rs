use crate::core::streaming::{ProgressStage, ToolProgress};
use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

pub struct ProgressTool;

impl ProgressTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ClawTool for ProgressTool {
    fn name(&self) -> &str {
        "progress"
    }

    fn description(&self) -> &str {
        "工具执行进度报告器。用于长时间操作中报告中间状态。\n\
         支持操作：report (报告进度), format (格式化进度)。"
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["report", "format"],
                    "description": "操作类型"
                },
                "stage": {
                    "type": "string",
                    "enum": ["started", "running", "completed", "failed"],
                    "description": "进度阶段 (report 时使用)"
                },
                "message": {
                    "type": "string",
                    "description": "进度消息"
                },
                "percentage": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 100,
                    "description": "完成百分比 (report 时使用)"
                }
            },
            "required": ["action"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClawError::Validation("缺少 action 参数".to_string()))?;

        match action {
            "report" => {
                let stage_str = args
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .unwrap_or("running");
                let stage = match stage_str {
                    "started" => ProgressStage::Started,
                    "completed" => ProgressStage::Completed,
                    "failed" => ProgressStage::Failed,
                    _ => ProgressStage::Running,
                };
                let message = args.get("message").and_then(|v| v.as_str()).unwrap_or("");
                let percentage = args
                    .get("percentage")
                    .and_then(|v| v.as_u64())
                    .map(|p| p as u8);
                let progress = ToolProgress {
                    tool_name: "progress".to_string(),
                    stage,
                    message: message.to_string(),
                    percentage,
                };
                Ok(crate::core::streaming::format_progress(&progress))
            }
            "format" => Ok(
                "进度报告格式: ⏳ started / 🔄 running (N%) / ✅ completed / ❌ failed".to_string(),
            ),
            _ => Err(ClawError::Validation(format!(
                "未知操作: {}。支持: report, format",
                action
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{ClawTool, ToolContext};
    use serde_json::json;

    fn test_ctx() -> ToolContext {
        ToolContext {
            config: crate::test_helpers::test_config(),
            http_client: reqwest::Client::new(),
            delegate_runtime: None,
        }
    }

    #[tokio::test]
    async fn test_report_started() {
        let tool = ProgressTool::new();
        let result = tool.execute(&json!({
            "action": "report",
            "stage": "started",
            "message": "开始处理"
        }), &test_ctx()).await.unwrap();
        assert!(result.contains("⏳"), "started should show hourglass");
    }

    #[tokio::test]
    async fn test_report_running() {
        let tool = ProgressTool::new();
        let result = tool.execute(&json!({
            "action": "report",
            "stage": "running",
            "message": "处理中",
            "percentage": 45
        }), &test_ctx()).await.unwrap();
        assert!(result.contains("🔄"), "running should show spinner");
        assert!(result.contains("45%"), "should show percentage");
    }

    #[tokio::test]
    async fn test_report_completed() {
        let tool = ProgressTool::new();
        let result = tool.execute(&json!({
            "action": "report",
            "stage": "completed",
            "message": "完成！"
        }), &test_ctx()).await.unwrap();
        assert!(result.contains("✅"), "completed should show checkmark");
    }

    #[tokio::test]
    async fn test_report_failed() {
        let tool = ProgressTool::new();
        let result = tool.execute(&json!({
            "action": "report",
            "stage": "failed",
            "message": "出错"
        }), &test_ctx()).await.unwrap();
        assert!(result.contains("❌"), "failed should show cross");
    }

    #[tokio::test]
    async fn test_format_action() {
        let tool = ProgressTool::new();
        let result = tool.execute(&json!({"action": "format"}), &test_ctx()).await.unwrap();
        assert!(result.contains("⏳"), "format should describe emoji format");
    }

    #[tokio::test]
    async fn test_invalid_action() {
        let tool = ProgressTool::new();
        let result = tool.execute(&json!({"action": "invalid"}), &test_ctx()).await;
        assert!(result.is_err(), "invalid action should fail");
    }

    #[tokio::test]
    async fn test_missing_action() {
        let tool = ProgressTool::new();
        let result = tool.execute(&json!({}), &test_ctx()).await;
        assert!(result.is_err(), "missing action should fail");
    }
}
