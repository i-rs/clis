use crate::core::tool_chain::{ChainCondition, ChainStep, ConditionOp, ToolChain};
use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

pub struct ChainTool;

impl ChainTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ClawTool for ChainTool {
    fn name(&self) -> &str {
        "chain"
    }

    fn description(&self) -> &str {
        "工具链执行器。将多个工具调用按预定义的顺序编排执行，支持条件分支和上下文传递。\n\
         支持操作：list (列出预设链), build (构建自定义链)。"
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["list", "build"],
                    "description": "操作类型"
                },
                "steps": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "tool": { "type": "string" },
                            "args_template": { "type": "object" }
                        },
                        "required": ["tool"]
                    },
                    "description": "链步骤列表 (build 时使用)"
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
            "list" => {
                let chains = crate::core::tool_chain::builtin_chains();
                let list: Vec<Value> = chains
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "name": c.name,
                            "description": c.description,
                            "steps": c.steps.len(),
                        })
                    })
                    .collect();
                Ok(serde_json::to_string_pretty(&list).unwrap_or_else(|_| "[]".to_string()))
            }
            "build" => {
                let steps_val = args
                    .get("steps")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| ClawError::Validation("build 需要 steps 参数".to_string()))?;
                let mut chain = ToolChain::new("custom", "用户自定义工具链");
                for step_val in steps_val {
                    let tool_name = step_val
                        .get("tool")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let args_template = step_val
                        .get("args_template")
                        .cloned()
                        .unwrap_or(serde_json::json!({}));
                    let condition = step_val.get("condition").and_then(|c| {
                        let field = c.get("field")?.as_str()?;
                        let operator_str = c.get("operator")?.as_str()?;
                        let value = c.get("value")?;
                        let op = match operator_str {
                            "not_equals" => ConditionOp::NotEquals,
                            "contains" => ConditionOp::Contains,
                            "greater_than" => ConditionOp::GreaterThan,
                            "less_than" => ConditionOp::LessThan,
                            "is_empty" => ConditionOp::IsEmpty,
                            "is_not_empty" => ConditionOp::IsNotEmpty,
                            _ => ConditionOp::Equals,
                        };
                        Some(ChainCondition {
                            field: field.to_string(),
                            operator: op,
                            value: value.clone(),
                        })
                    });
                    chain.steps.push(ChainStep {
                        tool_name: tool_name.to_string(),
                        args_template,
                        output_key: format!("step_{}", chain.steps.len()),
                        condition,
                    });
                }
                Ok(format!(
                    "已构建工具链 '{}'，包含 {} 个步骤。使用 orchestration 工具执行。",
                    chain.name,
                    chain.steps.len()
                ))
            }
            _ => Err(ClawError::Validation(format!(
                "未知操作: {}。支持: list, build",
                action
            ))),
        }
    }
}
