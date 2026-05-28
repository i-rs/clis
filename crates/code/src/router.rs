#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskComplexity {
    Simple,
    Complex,
    Heavy,
}

impl TaskComplexity {
    pub fn recommended_model(&self) -> Option<&'static str> {
        match self {
            Self::Heavy => Some("o3-mini"),
            Self::Complex => Some("gpt-4o"),
            Self::Simple => None,
        }
    }

    pub fn execution_mode(&self) -> ExecutionMode {
        match self {
            Self::Heavy | Self::Complex => ExecutionMode::PlanThenExecute,
            Self::Simple => ExecutionMode::ReAct,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionMode {
    ReAct,
    PlanThenExecute,
}

pub fn classify_complexity(task: &str) -> TaskComplexity {
    let task_lower = task.to_lowercase();

    let heavy_keywords = [
        "refactor", "重构", "rewrite", "重写",
        "migrate", "迁移", "implement", "实现",
        "all tests", "全量测试", "整个项目",
        "分析", "analyze", "review",
        "design", "设计", "架构",
    ];
    for kw in &heavy_keywords {
        if task_lower.contains(kw) {
            return TaskComplexity::Heavy;
        }
    }

    let complex_keywords = [
        "同时", "并且", "然后", "分别",
        "and then", "also", "both", "multi",
        "多个文件", "multiple files",
    ];
    for kw in &complex_keywords {
        if task_lower.contains(kw) {
            return TaskComplexity::Complex;
        }
    }

    TaskComplexity::Simple
}

pub fn build_plan_prompt(task: &str) -> String {
    format!(
        "You are a planning agent. Create a detailed step-by-step plan for the task below.\n\
         \n\
         Task: {}\n\
         \n\
         Rules:\n\
         - Use numbered steps (1., 2., 3., ...)\n\
         - Each step must be specific and actionable\n\
         - For each step, list the tools or commands needed\n\
         - Estimate files that need to be modified\n\
         - Keep under 15 steps\n\
         \n\
         Plan:",
        task
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_heavy() {
        assert_eq!(classify_complexity("重构这个模块"), TaskComplexity::Heavy);
        assert_eq!(classify_complexity("refactor this code"), TaskComplexity::Heavy);
        assert_eq!(classify_complexity("implement login feature"), TaskComplexity::Heavy);
    }

    #[test]
    fn test_classify_complex() {
        assert_eq!(classify_complexity("同时修改 A 和 B"), TaskComplexity::Complex);
        assert_eq!(classify_complexity("modify multiple files"), TaskComplexity::Complex);
    }

    #[test]
    fn test_classify_simple() {
        assert_eq!(classify_complexity("read main.rs"), TaskComplexity::Simple);
        assert_eq!(classify_complexity("what does this function do"), TaskComplexity::Simple);
    }

    #[test]
    fn test_execution_mode_mapping() {
        assert_eq!(classify_complexity("read file").execution_mode(), ExecutionMode::ReAct);
        assert_eq!(classify_complexity("refactor module").execution_mode(), ExecutionMode::PlanThenExecute);
        assert_eq!(classify_complexity("同时修改 A 和 B").execution_mode(), ExecutionMode::PlanThenExecute);
    }

    #[test]
    fn test_recommended_model() {
        assert!(classify_complexity("read file").recommended_model().is_none());
        assert!(classify_complexity("refactor module").recommended_model().is_some());
    }

    #[test]
    fn test_build_plan_prompt() {
        let prompt = build_plan_prompt("fix bug in login");
        assert!(prompt.contains("fix bug in login"));
        assert!(prompt.contains("Plan:"));
    }
}
