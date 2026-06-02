use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingMemory {
    pub entities: HashMap<String, String>,
    pub pending_actions: Vec<String>,
    pub current_task: Option<String>,
}

impl WorkingMemory {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            pending_actions: Vec::new(),
            current_task: None,
        }
    }

    pub fn set_entity(&mut self, key: &str, value: &str) {
        self.entities.insert(key.to_string(), value.to_string());
    }

    pub fn get_entity(&self, key: &str) -> Option<&str> {
        self.entities.get(key).map(|s| s.as_str())
    }

    pub fn set_current_task(&mut self, task: &str) {
        self.current_task = Some(task.to_string());
    }

    pub fn clear(&mut self) {
        self.entities.clear();
        self.pending_actions.clear();
        self.current_task = None;
    }

    pub fn format_for_prompt(&self) -> String {
        if self.entities.is_empty() && self.current_task.is_none() {
            return String::new();
        }
        let mut parts = Vec::new();
        parts.push("## 工作记忆 (当前会话)".to_string());
        if let Some(ref task) = self.current_task {
            parts.push(format!("当前任务：{}", task));
        }
        for (key, value) in &self.entities {
            parts.push(format!("{}：{}", key, value));
        }
        if !self.pending_actions.is_empty() {
            parts.push(format!("待执行：{}", self.pending_actions.join(", ")));
        }
        parts.join("\n")
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermFact {
    pub content: String,
    pub source: String,
    pub created_at: i64,
    pub last_accessed: i64,
    pub access_count: u32,
    pub category: FactCategory,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum FactCategory {
    UserPreference,
    UserHabit,
    ToolResult,
    Decision,
    General,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongTermMemory {
    pub facts: Vec<LongTermFact>,
    #[serde(default = "default_max_facts")]
    pub max_facts: usize,
}

fn default_max_facts() -> usize {
    200
}

impl LongTermMemory {
    pub fn new() -> Self {
        Self {
            facts: Vec::new(),
            max_facts: 200,
        }
    }

    pub fn add_fact(&mut self, content: &str, source: &str, category: FactCategory) {
        let now = chrono::Utc::now().timestamp();
        if let Some(existing) = self.facts.iter_mut().find(|f| f.content == content) {
            existing.last_accessed = now;
            existing.access_count += 1;
            return;
        }
        self.facts.push(LongTermFact {
            content: content.to_string(),
            source: source.to_string(),
            created_at: now,
            last_accessed: now,
            access_count: 1,
            category,
        });
        if self.facts.len() > self.max_facts {
            self.evict();
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<&LongTermFact> {
        let query_lower = query.to_lowercase();
        let mut scored: Vec<(i64, &LongTermFact)> = self
            .facts
            .iter()
            .filter_map(|fact| {
                if fact.content.to_lowercase().contains(&query_lower) {
                    let score = (fact.access_count as i64) * 10
                        + (fact.last_accessed / 3600)
                        - ((chrono::Utc::now().timestamp() - fact.last_accessed) / 86400);
                    Some((score, fact))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        scored.into_iter().take(limit).map(|(_, f)| f).collect()
    }

    pub fn search_by_category(&self, category: FactCategory, limit: usize) -> Vec<&LongTermFact> {
        self.facts
            .iter()
            .filter(|f| f.category == category)
            .take(limit)
            .collect()
    }

    fn evict(&mut self) {
        let now = chrono::Utc::now().timestamp();
        self.facts.sort_by_key(|f| {
            let age_days = (now - f.last_accessed) / 86400;
            -(f.access_count as i64 * 10 - age_days)
        });
        self.facts.truncate(self.max_facts);
    }

    pub fn decay(&mut self, max_age_days: i64) {
        let now = chrono::Utc::now().timestamp();
        self.facts.retain(|f| {
            let age_days = (now - f.last_accessed) / 86400;
            age_days < max_age_days || f.access_count > 3
        });
    }

    pub fn format_for_prompt(&self, max_items: usize) -> String {
        if self.facts.is_empty() {
            return String::new();
        }
        let mut parts = Vec::new();
        parts.push("## 长期记忆".to_string());
        let now = chrono::Utc::now().timestamp();
        let mut sorted_facts: Vec<_> = self.facts.iter().collect();
        sorted_facts.sort_by_key(|f| std::cmp::Reverse((f.access_count, -(now - f.last_accessed))));
        for fact in sorted_facts.iter().take(max_items) {
            let cat_label = match fact.category {
                FactCategory::UserPreference => "偏好",
                FactCategory::UserHabit => "习惯",
                FactCategory::ToolResult => "工具结果",
                FactCategory::Decision => "决策",
                FactCategory::General => "事实",
            };
            parts.push(format!("[{}] {} (来源: {})", cat_label, fact.content, fact.source));
        }
        parts.join("\n")
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySummary {
    pub summary: String,
    pub created_at: i64,
    pub fact_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredMemory {
    #[serde(default)]
    pub working: WorkingMemory,
    #[serde(default)]
    pub long_term: LongTermMemory,
    #[serde(default)]
    pub summaries: Vec<MemorySummary>,
}

impl LayeredMemory {
    pub fn new() -> Self {
        Self {
            working: WorkingMemory::new(),
            long_term: LongTermMemory::new(),
            summaries: Vec::new(),
        }
    }

    pub fn record_tool_result(&mut self, tool_name: &str, result: &str) {
        if result.len() < 10 || result.starts_with("错误") || result.starts_with("Error") {
            return;
        }
        let facts = extract_facts_from_result(tool_name, result);
        for fact in facts {
            self.long_term.add_fact(
                &fact,
                &format!("tool:{}", tool_name),
                FactCategory::ToolResult,
            );
        }
    }

    pub fn record_user_statement(&mut self, statement: &str) {
        let category = if statement.contains("喜欢") || statement.contains("偏好") || statement.contains("不喜欢") {
            FactCategory::UserPreference
        } else if statement.contains("通常") || statement.contains("总是") || statement.contains("习惯") {
            FactCategory::UserHabit
        } else {
            FactCategory::General
        };
        self.long_term.add_fact(statement, "user_statement", category);
    }

    pub fn add_summary(&mut self, summary: &str, fact_count: usize) {
        self.summaries.push(MemorySummary {
            summary: summary.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            fact_count,
        });
        if self.summaries.len() > 10 {
            self.summaries.remove(0);
        }
    }

    pub fn format_for_prompt(&self) -> String {
        let mut parts = Vec::new();
        let working = self.working.format_for_prompt();
        if !working.is_empty() {
            parts.push(working);
        }
        let long_term = self.long_term.format_for_prompt(10);
        if !long_term.is_empty() {
            parts.push(long_term);
        }
        if !self.summaries.is_empty() {
            parts.push("## 会话摘要".to_string());
            for summary in self.summaries.iter().rev().take(3) {
                parts.push(format!("- {}", summary.summary));
            }
        }
        parts.join("\n\n")
    }

    pub fn build_llm_summary_prompt(&self, conversation_snippet: &str) -> String {
        let existing_facts: Vec<&str> = self.long_term.facts.iter().take(20).map(|f| f.content.as_str()).collect();
        format!(
            "请从以下对话片段中提取关键事实和用户偏好。输出格式：每行一个事实，以 - 开头。\n\
             不要重复已有事实。\n\n\
             已有事实：\n{}\n\n\
             对话片段：\n{}\n\n\
             提取的关键事实：",
            if existing_facts.is_empty() {
                "(暂无)".to_string()
            } else {
                existing_facts.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
            },
            conversation_snippet
        )
    }

    pub fn end_session(&mut self) {
        self.working.clear();
        self.long_term.decay(90);
    }
}

impl Default for LayeredMemory {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_facts_from_result(tool_name: &str, result: &str) -> Vec<String> {
    let mut facts = Vec::new();
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(result) {
        if let Some(arr) = parsed.as_array() {
            let count = arr.len();
            if count > 0 && count <= 5 {
                facts.push(format!("{} 查询返回 {} 条记录", tool_name, count));
            }
        }
        if let Some(obj) = parsed.as_object() {
            if let Some(total) = obj.get("total").and_then(|t| t.as_f64()) {
                facts.push(format!("{} 当前总计: {}", tool_name, total));
            }
            if let Some(value) = obj.get("value").and_then(|v| v.as_f64()) {
                facts.push(format!("{} 最新值: {}", tool_name, value));
            }
        }
    }
    facts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_working_memory_basic() {
        let mut wm = WorkingMemory::new();
        assert!(wm.format_for_prompt().is_empty());
        wm.set_entity("体重", "70kg");
        wm.set_current_task("查看体重趋势");
        let prompt = wm.format_for_prompt();
        assert!(prompt.contains("70kg"));
        assert!(prompt.contains("查看体重趋势"));
    }

    #[test]
    fn test_working_memory_clear() {
        let mut wm = WorkingMemory::new();
        wm.set_entity("test", "value");
        wm.clear();
        assert!(wm.entities.is_empty());
    }

    #[test]
    fn test_long_term_memory_add() {
        let mut ltm = LongTermMemory::new();
        ltm.add_fact("用户喜欢咖啡", "user", FactCategory::UserPreference);
        assert_eq!(ltm.facts.len(), 1);
        assert_eq!(ltm.facts[0].access_count, 1);
    }

    #[test]
    fn test_long_term_memory_dedup() {
        let mut ltm = LongTermMemory::new();
        ltm.add_fact("用户喜欢咖啡", "user", FactCategory::UserPreference);
        ltm.add_fact("用户喜欢咖啡", "user", FactCategory::UserPreference);
        assert_eq!(ltm.facts.len(), 1);
        assert_eq!(ltm.facts[0].access_count, 2);
    }

    #[test]
    fn test_long_term_memory_search() {
        let mut ltm = LongTermMemory::new();
        ltm.add_fact("体重 70kg", "weight", FactCategory::ToolResult);
        ltm.add_fact("心情愉快", "mood", FactCategory::General);
        let results = ltm.search("体重", 5);
        assert_eq!(results.len(), 1);
        assert!(results[0].content.contains("70kg"));
    }

    #[test]
    fn test_long_term_memory_eviction() {
        let mut ltm = LongTermMemory::new();
        ltm.max_facts = 3;
        ltm.add_fact("fact1", "test", FactCategory::General);
        ltm.add_fact("fact2", "test", FactCategory::General);
        ltm.add_fact("fact3", "test", FactCategory::General);
        ltm.add_fact("fact4", "test", FactCategory::General);
        assert!(ltm.facts.len() <= 3);
    }

    #[test]
    fn test_layered_memory_record_tool() {
        let mut mem = LayeredMemory::new();
        mem.record_tool_result("weight", r#"{"total": 5, "value": 70.5}"#);
        assert!(!mem.long_term.facts.is_empty());
    }

    #[test]
    fn test_layered_memory_record_user() {
        let mut mem = LayeredMemory::new();
        mem.record_user_statement("我喜欢喝咖啡");
        let prefs = mem.long_term.search_by_category(FactCategory::UserPreference, 10);
        assert_eq!(prefs.len(), 1);
    }

    #[test]
    fn test_layered_memory_format() {
        let mut mem = LayeredMemory::new();
        mem.working.set_entity("当前任务", "查看体重");
        mem.long_term.add_fact("用户喜欢咖啡", "user", FactCategory::UserPreference);
        let prompt = mem.format_for_prompt();
        assert!(prompt.contains("工作记忆"));
        assert!(prompt.contains("长期记忆"));
    }

    #[test]
    fn test_layered_memory_end_session() {
        let mut mem = LayeredMemory::new();
        mem.working.set_entity("test", "value");
        mem.end_session();
        assert!(mem.working.entities.is_empty());
    }

    #[test]
    fn test_extract_facts() {
        let facts = extract_facts_from_result("weight", r#"{"total": 5, "value": 70.5}"#);
        assert!(facts.iter().any(|f| f.contains("70.5")));
    }

    #[test]
    fn test_decay() {
        let mut ltm = LongTermMemory::new();
        let old_ts = chrono::Utc::now().timestamp() - 100 * 86400;
        ltm.facts.push(LongTermFact {
            content: "old fact".to_string(),
            source: "test".to_string(),
            created_at: old_ts,
            last_accessed: old_ts,
            access_count: 1,
            category: FactCategory::General,
        });
        ltm.add_fact("recent fact", "test", FactCategory::General);
        ltm.decay(90);
        assert!(ltm.facts.iter().all(|f| f.content != "old fact"));
    }

    #[test]
    fn test_llm_summary_prompt() {
        let mem = LayeredMemory::new();
        let prompt = mem.build_llm_summary_prompt("用户说今天心情不好");
        assert!(prompt.contains("对话片段"));
        assert!(prompt.contains("提取的关键事实"));
    }
}
