use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OrchestrationMode {
    Sequential,
    Parallel,
    FanOutGather,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStep {
    pub agent_id: String,
    pub task: String,
    pub depends_on: Vec<String>,
    pub status: StepStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationPlan {
    pub mode: OrchestrationMode,
    pub steps: Vec<OrchestrationStep>,
    pub context: String,
}

#[allow(dead_code)]
impl OrchestrationPlan {
    pub fn sequential(steps: Vec<(String, String)>) -> Self {
        let orchestration_steps: Vec<OrchestrationStep> = steps
            .iter()
            .enumerate()
            .map(|(i, (agent_id, task))| {
                let depends_on = if i > 0 {
                    vec![format!("step_{}", i - 1)]
                } else {
                    vec![]
                };
                OrchestrationStep {
                    agent_id: agent_id.clone(),
                    task: task.clone(),
                    depends_on,
                    status: StepStatus::Pending,
                }
            })
            .collect();
        Self {
            mode: OrchestrationMode::Sequential,
            steps: orchestration_steps,
            context: String::new(),
        }
    }

    pub fn parallel(steps: Vec<(String, String)>) -> Self {
        let orchestration_steps: Vec<OrchestrationStep> = steps
            .into_iter()
            .map(|(agent_id, task)| OrchestrationStep {
                agent_id,
                task,
                depends_on: vec![],
                status: StepStatus::Pending,
            })
            .collect();
        Self {
            mode: OrchestrationMode::Parallel,
            steps: orchestration_steps,
            context: String::new(),
        }
    }

    pub fn with_context(mut self, context: &str) -> Self {
        self.context = context.to_string();
        self
    }

    pub fn ready_steps(&self) -> Vec<usize> {
        self.steps
            .iter()
            .enumerate()
            .filter(|(_, step)| {
                step.status == StepStatus::Pending
                    && step.depends_on.is_empty()
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn mark_running(&mut self, idx: usize) {
        if let Some(step) = self.steps.get_mut(idx) {
            step.status = StepStatus::Running;
        }
    }

    pub fn mark_completed(&mut self, idx: usize) {
        if let Some(step) = self.steps.get_mut(idx) {
            step.status = StepStatus::Completed;
        }
        self.update_dependents();
    }

    pub fn mark_failed(&mut self, idx: usize) {
        if let Some(step) = self.steps.get_mut(idx) {
            step.status = StepStatus::Failed;
        }
    }

    fn update_dependents(&mut self) {
        let completed_ids: Vec<String> = self
            .steps
            .iter()
            .enumerate()
            .filter(|(_, s)| s.status == StepStatus::Completed)
            .map(|(i, _)| format!("step_{}", i))
            .collect();
        for step in &mut self.steps {
            if step.status == StepStatus::Pending && !step.depends_on.is_empty() {
                let all_deps_met = step
                    .depends_on
                    .iter()
                    .all(|dep| completed_ids.contains(dep));
                if all_deps_met {
                    step.depends_on.clear();
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn is_complete(&self) -> bool {
        self.steps.iter().all(|s| {
            s.status == StepStatus::Completed
                || s.status == StepStatus::Failed
                || s.status == StepStatus::Skipped
        })
    }

    pub fn progress_summary(&self) -> String {
        let total = self.steps.len();
        let completed = self.steps.iter().filter(|s| s.status == StepStatus::Completed).count();
        let failed = self.steps.iter().filter(|s| s.status == StepStatus::Failed).count();
        format!(
            "编排进度: {}/{} 完成, {} 失败 (模式: {:?})",
            completed, total, failed, self.mode
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationResult {
    pub plan: OrchestrationPlan,
    pub step_results: HashMap<usize, Result<String, String>>,
    pub total_elapsed_ms: u64,
}

impl OrchestrationResult {
    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        self.step_results.values().all(|r| r.is_ok())
    }

    #[allow(dead_code)]
    pub fn combined_output(&self) -> String {
        let mut parts = Vec::new();
        for (idx, result) in &self.step_results {
            match result {
                Ok(output) => parts.push(format!("[步骤 {}] {}", idx, output)),
                Err(e) => parts.push(format!("[步骤 {}] 失败: {}", idx, e)),
            }
        }
        parts.join("\n\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_plan() {
        let plan = OrchestrationPlan::sequential(vec![
            ("analyst".to_string(), "分析数据".to_string()),
            ("coder".to_string(), "生成报告".to_string()),
        ]);
        assert_eq!(plan.mode, OrchestrationMode::Sequential);
        assert_eq!(plan.steps.len(), 2);
        assert!(plan.steps[0].depends_on.is_empty());
        assert!(!plan.steps[1].depends_on.is_empty());
    }

    #[test]
    fn test_parallel_plan() {
        let plan = OrchestrationPlan::parallel(vec![
            ("agent_a".to_string(), "任务A".to_string()),
            ("agent_b".to_string(), "任务B".to_string()),
        ]);
        assert_eq!(plan.mode, OrchestrationMode::Parallel);
        assert!(plan.steps[0].depends_on.is_empty());
        assert!(plan.steps[1].depends_on.is_empty());
        let ready = plan.ready_steps();
        assert_eq!(ready.len(), 2);
    }

    #[test]
    fn test_sequential_execution_flow() {
        let mut plan = OrchestrationPlan::sequential(vec![
            ("a".to_string(), "task_a".to_string()),
            ("b".to_string(), "task_b".to_string()),
        ]);
        let ready = plan.ready_steps();
        assert_eq!(ready, vec![0]);
        plan.mark_running(0);
        plan.mark_completed(0);
        let ready = plan.ready_steps();
        assert_eq!(ready, vec![1]);
        plan.mark_completed(1);
        assert!(plan.is_complete());
    }

    #[test]
    fn test_progress_summary() {
        let plan = OrchestrationPlan::parallel(vec![
            ("a".to_string(), "task_a".to_string()),
            ("b".to_string(), "task_b".to_string()),
        ]);
        let summary = plan.progress_summary();
        assert!(summary.contains("0/2"));
    }

    #[test]
    fn test_orchestration_result() {
        let plan = OrchestrationPlan::sequential(vec![
            ("a".to_string(), "task".to_string()),
        ]);
        let mut step_results = HashMap::new();
        step_results.insert(0, Ok("done".to_string()));
        let result = OrchestrationResult {
            plan,
            step_results,
            total_elapsed_ms: 1000,
        };
        assert!(result.is_success());
        assert!(result.combined_output().contains("done"));
    }

    #[test]
    fn test_failed_step() {
        let mut plan = OrchestrationPlan::sequential(vec![
            ("a".to_string(), "task".to_string()),
            ("b".to_string(), "task2".to_string()),
        ]);
        plan.mark_failed(0);
        assert!(!plan.is_complete());
        let ready = plan.ready_steps();
        assert!(ready.is_empty());
    }
}
