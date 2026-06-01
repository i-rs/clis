use crate::config::ResolvedAgentConfig;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskComplexity {
    Simple,
    Complex,
    Heavy,
}

/// Classify task complexity based on user input.
#[allow(dead_code)]
pub fn classify_complexity(task: &str) -> TaskComplexity {
    let task_lower = task.to_lowercase();

    let heavy_keywords = [
        "统计", "分析", "图表", "对比", "比较", "汇总", "报告", "stats", "chart", "analyze",
        "compare", "report", "summary", "趋势", "规划", "计划",
    ];
    for kw in &heavy_keywords {
        if task_lower.contains(kw) {
            return TaskComplexity::Heavy;
        }
    }

    let complex_keywords = [
        "同时", "并且", "然后", "分别", "and", "then", "also", "plus", "both",
    ];
    for kw in &complex_keywords {
        if task_lower.contains(kw) {
            return TaskComplexity::Complex;
        }
    }

    let tool_count = task_lower.matches("查看").count()
        + task_lower.matches("记录").count()
        + task_lower.matches("查询").count()
        + task_lower.matches("列出").count();
    if tool_count >= 2 {
        return TaskComplexity::Complex;
    }

    TaskComplexity::Simple
}

pub struct TaskRouter {
    agents: Vec<ResolvedAgentConfig>,
    sub_agents: Vec<ResolvedAgentConfig>,
}

impl TaskRouter {
    pub fn new(agents: Vec<ResolvedAgentConfig>, sub_agents: Vec<ResolvedAgentConfig>) -> Self {
        Self { agents, sub_agents }
    }

    #[allow(dead_code)]
    pub fn select_agent<'a>(
        &'a self,
        task: &str,
        _current_agent_id: &str,
        default_agent_id: &str,
    ) -> Option<(&'a str, bool)> {
        let complexity = classify_complexity(task);

        for agent in &self.sub_agents {
            if !agent.capabilities.is_empty() {
                for cap in &agent.capabilities {
                    if task.contains(cap.as_str()) {
                        return Some((&agent.agent_id, true));
                    }
                }
            }
        }

        if complexity != TaskComplexity::Simple {
            for agent in &self.agents {
                if agent.agent_id == default_agent_id {
                    continue;
                }
                if !agent.capabilities.is_empty() {
                    for cap in &agent.capabilities {
                        if task.contains(cap.as_str()) {
                            return Some((&agent.agent_id, false));
                        }
                    }
                }
            }
        }

        if complexity == TaskComplexity::Heavy {
            for agent in &self.agents {
                if agent.agent_id == default_agent_id {
                    continue;
                }
                let provider = agent.provider.to_lowercase();
                if provider == "anthropic" {
                    return Some((&agent.agent_id, false));
                }
            }
            if let Some(agent) = self.agents.iter().find(|a| a.agent_id != default_agent_id) {
                return Some((&agent.agent_id, false));
            }
        }

        None
    }

    pub fn routing_hint(&self) -> String {
        let mut hint = String::new();

        if !self.sub_agents.is_empty() {
            hint.push_str("## 可用子智能体\n\n");
            hint.push_str("以下子智能体可通过 delegate_task 工具委托任务：\n\n");
            for agent in &self.sub_agents {
                let caps = if agent.capabilities.is_empty() {
                    "通用".to_string()
                } else {
                    agent.capabilities.join(", ")
                };
                hint.push_str(&format!(
                    "- **{}**: {} ({}) — 擅长: {}\n",
                    agent.agent_id, agent.model, agent.provider, caps
                ));
            }
            hint.push('\n');
        }

        if self.agents.len() > 1 {
            hint.push_str("## 可用主智能体\n\n");
            for agent in &self.agents {
                let caps = if agent.capabilities.is_empty() {
                    String::new()
                } else {
                    format!(" (擅长: {})", agent.capabilities.join(", "))
                };
                hint.push_str(&format!(
                    "- {}: {} ({}){}\n",
                    agent.agent_id, agent.model, agent.provider, caps
                ));
            }
            hint.push('\n');
        }

        hint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_simple() {
        assert_eq!(classify_complexity("记录体重 70kg"), TaskComplexity::Simple);
    }

    #[test]
    fn test_classify_heavy_stats() {
        assert_eq!(classify_complexity("统计本周体重"), TaskComplexity::Heavy);
    }

    #[test]
    fn test_classify_heavy_analyze() {
        assert_eq!(
            classify_complexity("analyze my weight trend"),
            TaskComplexity::Heavy
        );
    }

    #[test]
    fn test_classify_complex_multi() {
        assert_eq!(
            classify_complexity("查看体重然后记录心情"),
            TaskComplexity::Complex
        );
    }

    #[test]
    fn test_routing_hint_empty() {
        let router = TaskRouter::new(vec![], vec![]);
        assert!(router.routing_hint().is_empty());
    }

    #[test]
    fn test_routing_hint_with_sub_agents() {
        let router = TaskRouter::new(
            vec![],
            vec![ResolvedAgentConfig {
                agent_id: "analyst".to_string(),
                provider: "openai".to_string(),
                api_key: String::new(),
                base_url: String::new(),
                model: "gpt-4o".to_string(),
                enabled_tools: Default::default(),
                system_prompt: None,
                mcp_servers: vec![],
                allowed_dirs: vec![],
                capabilities: vec!["数据分析".to_string()],
                execution_mode: crate::config::ExecutionMode::React,
            }],
        );
        let hint = router.routing_hint();
        assert!(hint.contains("analyst"));
        assert!(hint.contains("数据分析"));
    }

    #[test]
    fn test_select_agent_by_capability() {
        let router = TaskRouter::new(
            vec![],
            vec![ResolvedAgentConfig {
                agent_id: "analyst".to_string(),
                provider: "openai".to_string(),
                api_key: String::new(),
                base_url: String::new(),
                model: "gpt-4o".to_string(),
                enabled_tools: Default::default(),
                system_prompt: None,
                mcp_servers: vec![],
                allowed_dirs: vec![],
                capabilities: vec!["数据分析".to_string()],
                execution_mode: crate::config::ExecutionMode::React,
            }],
        );
        let result = router.select_agent("帮我做数据分析", "default", "default");
        assert_eq!(result, Some(("analyst", true)));
    }

    #[test]
    fn test_select_agent_no_match() {
        let router = TaskRouter::new(vec![], vec![]);
        let result = router.select_agent("记录体重", "default", "default");
        assert!(result.is_none());
    }
}
