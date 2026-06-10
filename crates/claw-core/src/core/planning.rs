use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed { reason: String },
    Skipped { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredPlanStep {
    pub id: String,
    pub index: usize,
    pub description: String,
    pub tool: Option<String>,
    pub expected_output: Option<String>,
    pub status: PlanStepStatus,
    pub depends_on: Vec<String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub actual_result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuredPlan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<StructuredPlanStep>,
    pub status: PlanStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PlanStatus {
    Draft,
    Approved,
    Executing,
    Completed,
    PartiallyCompleted,
    Failed,
    Cancelled,
}

#[allow(dead_code)]
impl StructuredPlan {
    pub fn new(goal: &str) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
            goal: goal.to_string(),
            steps: Vec::new(),
            status: PlanStatus::Draft,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_step(
        &mut self,
        description: &str,
        tool: Option<&str>,
        expected_output: Option<&str>,
        depends_on: Vec<String>,
    ) -> &mut StructuredPlanStep {
        let index = self.steps.len();
        let id = format!("step_{}", index);
        self.steps.push(StructuredPlanStep {
            id: id.clone(),
            index,
            description: description.to_string(),
            tool: tool.map(|s| s.to_string()),
            expected_output: expected_output.map(|s| s.to_string()),
            status: PlanStepStatus::Pending,
            depends_on,
            retry_count: 0,
            max_retries: 2,
            actual_result: None,
        });
        self.updated_at = chrono::Utc::now().timestamp();
        self.steps.last_mut().expect("step just pushed in add_step")
    }

    pub fn start(&mut self) {
        self.status = PlanStatus::Executing;
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn next_step(&mut self) -> Option<&StructuredPlanStep> {
        self.steps
            .iter()
            .find(|s| matches!(s.status, PlanStepStatus::Pending))
    }

    pub fn mark_in_progress(&mut self, step_id: &str) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.id == step_id) {
            step.status = PlanStepStatus::InProgress;
            self.updated_at = chrono::Utc::now().timestamp();
        }
    }

    pub fn mark_completed(&mut self, step_id: &str, result: &str) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.id == step_id) {
            step.status = PlanStepStatus::Completed;
            step.actual_result = Some(result.to_string());
            self.updated_at = chrono::Utc::now().timestamp();
        }
        self.update_plan_status();
    }

    pub fn mark_failed(&mut self, step_id: &str, reason: &str) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.id == step_id) {
            step.retry_count += 1;
            if step.retry_count >= step.max_retries {
                step.status = PlanStepStatus::Failed {
                    reason: reason.to_string(),
                };
            } else {
                step.status = PlanStepStatus::Pending;
            }
            step.actual_result = Some(reason.to_string());
            self.updated_at = chrono::Utc::now().timestamp();
        }
        self.update_plan_status();
    }

    pub fn skip_step(&mut self, step_id: &str, reason: &str) {
        if let Some(step) = self.steps.iter_mut().find(|s| s.id == step_id) {
            step.status = PlanStepStatus::Skipped {
                reason: reason.to_string(),
            };
            self.updated_at = chrono::Utc::now().timestamp();
        }
        self.update_plan_status();
    }

    fn update_plan_status(&mut self) {
        let all_completed = self
            .steps
            .iter()
            .all(|s| matches!(s.status, PlanStepStatus::Completed));
        let any_failed = self
            .steps
            .iter()
            .any(|s| matches!(s.status, PlanStepStatus::Failed { .. }));
        let all_done = self.steps.iter().all(|s| {
            matches!(
                s.status,
                PlanStepStatus::Completed
                    | PlanStepStatus::Failed { .. }
                    | PlanStepStatus::Skipped { .. }
            )
        });
        if all_completed {
            self.status = PlanStatus::Completed;
        } else if all_done && any_failed {
            self.status = PlanStatus::PartiallyCompleted;
        } else if all_done {
            self.status = PlanStatus::Completed;
        }
    }

    pub fn progress(&self) -> (usize, usize) {
        let completed = self
            .steps
            .iter()
            .filter(|s| matches!(s.status, PlanStepStatus::Completed))
            .count();
        (completed, self.steps.len())
    }

    pub fn format_progress(&self) -> String {
        let (completed, total) = self.progress();
        let mut result = format!("📋 执行计划: {} ({:?})\n", self.goal, self.status);
        for step in &self.steps {
            let icon = match &step.status {
                PlanStepStatus::Pending => "⬜",
                PlanStepStatus::InProgress => "🔄",
                PlanStepStatus::Completed => "✅",
                PlanStepStatus::Failed { .. } => "❌",
                PlanStepStatus::Skipped { .. } => "⏭️",
            };
            let tool_info = step
                .tool
                .as_ref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();
            result.push_str(&format!(
                "{} {}{}. {}\n",
                icon,
                step.index + 1,
                tool_info,
                step.description
            ));
            if let PlanStepStatus::Failed { reason } = &step.status {
                result.push_str(&format!("   └ 失败原因: {}\n", reason));
            }
        }
        result.push_str(&format!("\n进度: {}/{} 步完成", completed, total));
        result
    }

    pub fn to_json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "steps": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "description": { "type": "string" },
                            "tool": { "type": "string" },
                            "expected_output": { "type": "string" },
                            "depends_on": {
                                "type": "array",
                                "items": { "type": "string" }
                            }
                        },
                        "required": ["description"]
                    }
                }
            },
            "required": ["steps"]
        })
    }

    pub fn parse_from_llm_output(text: &str) -> Option<Self> {
        let mut plan = None;
        let mut in_plan = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.contains("执行计划") || trimmed.contains("📋") {
                in_plan = true;
                let goal = trimmed
                    .replace("📋", "")
                    .replace("执行计划", "")
                    .replace(':', "")
                    .trim()
                    .to_string();
                plan = Some(StructuredPlan::new(&goal));
                continue;
            }
            if in_plan && let Some(ref mut p) = plan {
                let desc = trimmed
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start_matches('.')
                    .trim_start_matches("- ")
                    .trim_start_matches("✅ ")
                    .trim_start_matches("❌ ")
                    .trim_start_matches("🔄 ")
                    .trim();
                if desc.is_empty() || desc.len() < 3 {
                    continue;
                }
                let tool = if desc.contains("→") {
                    desc.split("→").nth(1).map(|s| s.trim().to_string())
                } else {
                    None
                };
                let description = if let Some(_t) = &tool {
                    desc.split("→").next().unwrap_or(desc).trim().to_string()
                } else {
                    desc.to_string()
                };
                if !description.is_empty() && description.len() > 2 {
                    p.add_step(&description, tool.as_deref(), None, vec![]);
                }
            }
        }
        plan
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_plan() {
        let mut plan = StructuredPlan::new("分析健康数据");
        plan.add_step("查询体重", Some("i_rs"), Some("体重列表"), vec![]);
        plan.add_step("查询心情", Some("i_rs"), Some("心情列表"), vec![]);
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.status, PlanStatus::Draft);
    }

    #[test]
    fn test_execute_plan() {
        let mut plan = StructuredPlan::new("测试");
        plan.add_step("步骤1", None, None, vec![]);
        plan.add_step("步骤2", None, None, vec![]);
        plan.start();
        let step = plan.next_step().unwrap();
        let id = step.id.clone();
        plan.mark_in_progress(&id);
        plan.mark_completed(&id, "ok");
        assert_eq!(plan.progress(), (1, 2));
        assert_eq!(plan.status, PlanStatus::Executing);
    }

    #[test]
    fn test_fail_step() {
        let mut plan = StructuredPlan::new("测试");
        plan.add_step("步骤1", None, None, vec![]);
        plan.start();
        let step = plan.next_step().unwrap();
        let id = step.id.clone();
        plan.mark_in_progress(&id);
        plan.mark_failed(&id, "error");
        plan.mark_failed(&id, "error");
        plan.mark_failed(&id, "error");
        assert!(matches!(
            plan.steps[0].status,
            PlanStepStatus::Failed { .. }
        ));
    }

    #[test]
    fn test_skip_step() {
        let mut plan = StructuredPlan::new("测试");
        plan.add_step("步骤1", None, None, vec![]);
        plan.skip_step("step_0", "不需要");
        assert!(matches!(
            plan.steps[0].status,
            PlanStepStatus::Skipped { .. }
        ));
    }

    #[test]
    fn test_format_progress() {
        let mut plan = StructuredPlan::new("健康分析");
        plan.add_step("查询体重", Some("i_rs"), None, vec![]);
        plan.add_step("查询心情", Some("i_rs"), None, vec![]);
        plan.mark_completed("step_0", "70kg");
        let formatted = plan.format_progress();
        assert!(formatted.contains("✅"));
        assert!(formatted.contains("⬜"));
    }

    #[test]
    fn test_parse_from_llm_output() {
        let text = "📋 执行计划：健康分析\n\
                    1. 查询体重数据 → i_rs\n\
                    2. 查询心情数据 → i_rs\n\
                    3. 生成综合报告";
        let plan = StructuredPlan::parse_from_llm_output(text);
        assert!(plan.is_some());
        let plan = plan.unwrap();
        assert_eq!(plan.steps.len(), 3);
    }

    #[test]
    fn test_to_json_schema() {
        let schema = StructuredPlan::to_json_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["steps"].is_object());
    }

    #[test]
    fn test_plan_completion() {
        let mut plan = StructuredPlan::new("测试");
        plan.add_step("步骤1", None, None, vec![]);
        plan.add_step("步骤2", None, None, vec![]);
        plan.start();
        plan.mark_completed("step_0", "ok");
        plan.mark_completed("step_1", "ok");
        assert_eq!(plan.status, PlanStatus::Completed);
    }
}
