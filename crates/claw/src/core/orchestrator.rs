#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ── Data Types ──

/// A single step in an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub id: String,
    pub description: String,
    pub status: StepStatus,
    /// Which agent should execute this step (None = default agent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Step {
    pub fn new(description: impl Into<String>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            description: description.into(),
            status: StepStatus::Pending,
            agent_id: None,
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn mark_done(&mut self, result: String) {
        self.status = StepStatus::Completed;
        self.result = Some(result);
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn mark_failed(&mut self, error: String) {
        self.status = StepStatus::Failed;
        self.error = Some(error);
        self.updated_at = chrono::Utc::now().timestamp();
    }

    pub fn mark_in_progress(&mut self) {
        self.status = StepStatus::InProgress;
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

/// Lifecycle status of a single step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// Overall status of a plan.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlanStatus {
    Active,
    Completed,
    Failed,
}

/// A complete execution plan with multiple steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<Step>,
    pub status: PlanStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Plan {
    pub fn new(goal: impl Into<String>, steps: Vec<Step>) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            goal: goal.into(),
            steps,
            status: PlanStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update plan status based on step statuses.
    pub fn refresh_status(&mut self) {
        self.updated_at = chrono::Utc::now().timestamp();
        let all_done = self.steps.iter().all(|s| {
            s.status == StepStatus::Completed || s.status == StepStatus::Skipped
        });
        let any_failed = self.steps.iter().any(|s| s.status == StepStatus::Failed);
        if all_done {
            self.status = PlanStatus::Completed;
        } else if any_failed {
            self.status = PlanStatus::Failed;
        }
    }

    /// Number of completed steps.
    pub fn completed_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .count()
    }

    /// Number of pending (not yet started) steps.
    pub fn pending_count(&self) -> usize {
        self.steps
            .iter()
            .filter(|s| s.status == StepStatus::Pending)
            .count()
    }
}

// ── Orchestrator Trait ──

/// Context needed for orchestrator operations.
#[allow(dead_code)]
pub struct PlanContext {
    /// Conversation config with tool overrides.
    pub config: crate::config::Config,
    /// Tool schemas for the LLM.
    pub tool_schemas: Vec<serde_json::Value>,
    /// MCP registry for tool execution.
    pub mcp: crate::mcp::McpRegistry,
}

/// Orchestrator trait — different strategies for plan creation and execution.
///
/// # Strategies
///
/// ## Sequential (default)
/// Each step is executed one at a time. The LLM focuses on one step at a time,
/// producing focused output without overwhelming context.
///
/// ## Parallel (future)
/// Independent steps could be routed to different sub-agents concurrently.
///
/// ## Hierarchical (future)
/// A supervising agent decomposes high-level goals into sub-plans managed by
/// sub-agents, with the supervisor coordinating outcomes.
#[allow(dead_code)]
pub trait Orchestrator: Send + Sync {
    fn name(&self) -> &str;

    /// Create a structured execution plan from a user goal.
    /// Uses the LLM to decompose the goal into discrete steps.
    fn create_plan(
        &self,
        goal: &str,
        context: &PlanContext,
    ) -> Result<Plan, String>;

    /// Get a human-readable status summary of the plan.
    fn status_summary(&self, plan: &Plan) -> String;

    /// Check whether the plan needs user confirmation before proceeding.
    fn needs_approval(&self, plan: &Plan) -> bool;
}

// ── Sequential Orchestrator ──

/// Default sequential orchestrator.
///
/// Creates plans by prompting the LLM to decompose goals into steps,
/// then executes them one at a time through the normal chat loop.
pub struct SequentialOrchestrator;

impl SequentialOrchestrator {
    /// Build a system prompt fragment for plan-then-execute mode.
    pub fn plan_prompt(&self, current_step: Option<&Step>) -> String {
        match current_step {
            Some(step) => format!(
                "## 执行计划（当前步骤）\n\n📋 当前步骤：{}\n\n专注于完成当前步骤。完成后，我会报告进度。",
                step.description
            ),
            None => "\
## 执行计划（多步骤操作）

如果用户请求涉及 **2 个或以上不同工具调用**，请先输出结构化执行计划。

格式要求（纯文本，不要用 JSON 代码块）：
📋 执行计划：
1. 步骤描述一
2. 步骤描述二
3. 步骤描述三

然后按顺序执行每一步。每完成一步，在回复中报告进度。"
                .to_string(),
        }
    }

    /// Format a plan as a brief status string for the system prompt.
    pub fn format_plan_status(plan: &Plan) -> String {
        let completed = plan.completed_count();
        let total = plan.steps.len();
        let mut s = format!("当前执行计划: {} (进度: {}/{})\n", plan.goal, completed, total);
        for (i, step) in plan.steps.iter().enumerate() {
            let marker = match step.status {
                StepStatus::Completed => "✅",
                StepStatus::InProgress => "🔄",
                StepStatus::Failed => "❌",
                StepStatus::Skipped => "⏭️",
                StepStatus::Pending => "  ",
            };
            s.push_str(&format!("{}. {} [{}]\n", i + 1, step.description, marker));
        }
        s
    }
}

impl Orchestrator for SequentialOrchestrator {
    fn name(&self) -> &str {
        "sequential"
    }

    fn create_plan(
        &self,
        goal: &str,
        _context: &PlanContext,
    ) -> Result<Plan, String> {
        // For the current implementation, the plan is created by the LLM
        // in the normal chat flow using system prompt instructions.
        // The create_plan method here provides the data structure foundation;
        // actual plan extraction from LLM output happens via pattern matching.
        //
        // When the LLM outputs a "📋 执行计划:" section in its first response,
        // the caller can parse it into Plan/Step structs.
        //
        // This method creates a minimal plan placeholder that gets populated
        // as the LLM works through the steps.
        let now = chrono::Utc::now().timestamp();
        Ok(Plan {
            id: uuid::Uuid::new_v4().to_string(),
            goal: goal.to_string(),
            steps: Vec::new(),
            status: PlanStatus::Active,
            created_at: now,
            updated_at: now,
        })
    }

    fn status_summary(&self, plan: &Plan) -> String {
        Self::format_plan_status(plan)
    }

    fn needs_approval(&self, _plan: &Plan) -> bool {
        // Sequential orchestrator always proceeds without user confirmation
        // (steps are visible as they execute). Future strategies might defer.
        false
    }
}
