use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};
use serde_json::Value;

pub struct OrchestrationTool;

impl Default for OrchestrationTool {
    fn default() -> Self {
        Self::new()
    }
}

impl OrchestrationTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl ClawTool for OrchestrationTool {
    fn name(&self) -> &str {
        "orchestrate"
    }

    fn description(&self) -> &str {
        "多步骤编排执行器。支持顺序 (sequential)、并行 (parallel)、扇出收集 (fan_out_gather) 三种模式。\n\
         操作：plan (创建编排计划), status (查看计划状态)。"
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["plan", "status"],
                    "description": "操作类型"
                },
                "mode": {
                    "type": "string",
                    "enum": ["sequential", "parallel", "fan_out_gather"],
                    "description": "编排模式 (plan 时使用)"
                },
                "plan_name": {
                    "type": "string",
                    "description": "计划名称 (plan 时使用)"
                },
                "steps": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "tool": { "type": "string" },
                            "args": { "type": "object" },
                            "depends_on": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        },
                        "required": ["name", "tool"]
                    },
                    "description": "编排步骤 (plan 时使用)"
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
            "plan" => {
                let mode_str = args
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("sequential");
                let plan_name = args
                    .get("plan_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unnamed");
                let steps_val = args
                    .get("steps")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();

                let mode = match mode_str {
                    "parallel" => crate::core::orchestration::OrchestrationMode::Parallel,
                    "fan_out_gather" => crate::core::orchestration::OrchestrationMode::FanOutGather,
                    _ => crate::core::orchestration::OrchestrationMode::Sequential,
                };

                let orchestration_steps: Vec<crate::core::orchestration::OrchestrationStep> =
                    steps_val
                        .iter()
                        .map(|step_val| {
                            let name = step_val
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unnamed_step");
                            let tool = step_val
                                .get("tool")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let depends_on: Vec<String> = step_val
                                .get("depends_on")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect()
                                })
                                .unwrap_or_default();
                            crate::core::orchestration::OrchestrationStep {
                                agent_id: format!("{}:{}", name, tool),
                                task: tool.to_string(),
                                depends_on,
                                status: crate::core::orchestration::StepStatus::Pending,
                            }
                        })
                        .collect();

                let plan = crate::core::orchestration::OrchestrationPlan {
                    mode,
                    steps: orchestration_steps,
                    context: plan_name.to_string(),
                };

                let ready = plan.ready_steps();
                let total = plan.steps.len();
                let ready_names: Vec<String> = ready
                    .iter()
                    .map(|&i| {
                        plan.steps
                            .get(i)
                            .map(|s| s.agent_id.clone())
                            .unwrap_or_default()
                    })
                    .collect();
                Ok(format!(
                    "已创建编排计划 '{}' (模式: {}, {} 步骤)\n就绪步骤: {}\n\n请使用对应工具逐步执行就绪的步骤。",
                    plan_name,
                    mode_str,
                    total,
                    ready_names.join(", ")
                ))
            }
            "status" => {
                Ok("编排计划状态查询。当前无活跃计划。使用 plan 操作创建新计划。".to_string())
            }
            _ => Err(ClawError::Validation(format!(
                "未知操作: {}。支持: plan, status",
                action
            ))),
        }
    }
}
