#![allow(dead_code)]

use crate::config::ResolvedAgentConfig;

/// Task complexity classification for model routing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskComplexity {
    /// Simple lookup, single tool call, list/get operations.
    Simple,
    /// Requires multiple tool calls, data aggregation, or cross-tool reasoning.
    Complex,
    /// Heavy analysis, planning, chart generation, multi-step workflows.
    Heavy,
}

/// Classify task complexity based on user input.
pub fn classify_complexity(task: &str) -> TaskComplexity {
    let task_lower = task.to_lowercase();

    // Heavy indicators: stats, chart, plan, analyze, compare, report
    let heavy_keywords = [
        "统计", "分析", "图表", "对比", "比较", "汇总", "报告",
        "stats", "chart", "analyze", "compare", "report", "summary",
        "趋势", "规划", "计划",
    ];
    for kw in &heavy_keywords {
        if task_lower.contains(kw) {
            return TaskComplexity::Heavy;
        }
    }

    // Complex indicators: multi-step, cross-tool
    let complex_keywords = [
        "同时", "并且", "然后", "分别",
        "and", "then", "also", "plus", "both",
    ];
    for kw in &complex_keywords {
        if task_lower.contains(kw) {
            return TaskComplexity::Complex;
        }
    }

    // Check for multiple tool-related keywords
    let tool_count = task_lower.matches("查看").count()
        + task_lower.matches("记录").count()
        + task_lower.matches("查询").count()
        + task_lower.matches("列出").count();
    if tool_count >= 2 {
        return TaskComplexity::Complex;
    }

    TaskComplexity::Simple
}

/// Routes tasks to the most appropriate agent/model based on complexity.
pub struct TaskRouter {
    agents: Vec<ResolvedAgentConfig>,
    sub_agents: Vec<ResolvedAgentConfig>,
}

impl TaskRouter {
    pub fn new(agents: Vec<ResolvedAgentConfig>, sub_agents: Vec<ResolvedAgentConfig>) -> Self {
        Self { agents, sub_agents }
    }

    /// Select the best agent for a given task.
    /// Returns (agent_id, is_sub_agent) or None if only default is available.
    pub fn select_agent<'a>(
        &'a self,
        task: &str,
        _current_agent_id: &str,
        default_agent_id: &str,
    ) -> Option<(&'a str, bool)> {
        let complexity = classify_complexity(task);

        // Try sub_agents first (they have explicit capabilities)
        for agent in &self.sub_agents {
            if !agent.capabilities.is_empty() {
                for cap in &agent.capabilities {
                    if task.contains(cap.as_str()) {
                        return Some((&agent.agent_id, true));
                    }
                }
            }
        }

        // For complex/heavy tasks, try main agents with capabilities
        if complexity != TaskComplexity::Simple {
            for agent in &self.agents {
                if agent.agent_id == default_agent_id {
                    continue; // Skip default, look for specialized agents
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

        // Provider-based routing for heavy tasks
        if complexity == TaskComplexity::Heavy {
            for agent in &self.agents {
                if agent.agent_id == default_agent_id {
                    continue;
                }
                let provider = agent.provider.to_lowercase();
                // Anthropic/DeepSeek tend to be better at reasoning-heavy tasks
                if provider == "anthropic" {
                    return Some((&agent.agent_id, false));
                }
            }
            // Fallback: use a non-default agent if available
            if let Some(agent) = self.agents.iter().find(|a| a.agent_id != default_agent_id) {
                return Some((&agent.agent_id, false));
            }
        }

        None // Use default agent
    }

    /// Generate a routing hint for the system prompt.
    pub fn routing_hint(&self) -> String {
        let mut hint = String::from("## 多模型路由\n\n");
        if self.agents.len() > 1 {
            hint.push_str("可用主智能体:\n");
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
        }
        if !self.sub_agents.is_empty() {
            hint.push_str("\n可用子智能体:\n");
            for agent in &self.sub_agents {
                let caps = agent.capabilities.join(", ");
                hint.push_str(&format!(
                    "- {}: {} ({}) [{}]\n",
                    agent.agent_id, agent.model, agent.provider, caps
                ));
            }
            hint.push_str("\n可通过 delegate_task 委托给子智能体。");
        }
        hint
    }
}
