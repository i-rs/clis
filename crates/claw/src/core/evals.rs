use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalCase {
    pub id: String,
    pub name: String,
    pub user_input: String,
    pub expected_tools: Vec<String>,
    pub expected_keywords: Vec<String>,
    pub forbidden_keywords: Vec<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub case_id: String,
    pub passed: bool,
    pub tool_accuracy: f64,
    pub keyword_match: f64,
    pub forbidden_violations: Vec<String>,
    pub actual_tools: Vec<String>,
    pub actual_response: String,
    pub score: f64,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalSuite {
    pub name: String,
    pub cases: Vec<EvalCase>,
}

impl EvalSuite {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            cases: Vec::new(),
        }
    }

    pub fn add_case(mut self, case: EvalCase) -> Self {
        self.cases.push(case);
        self
    }

    pub fn evaluate(
        &self,
        results: &[(String, String)],
    ) -> Vec<EvalResult> {
        self.cases
            .iter()
            .map(|case| {
                let (tools_used, response) = extract_tools_and_response(results);
                let tool_accuracy = compute_tool_accuracy(
                    &case.expected_tools,
                    &tools_used,
                );
                let keyword_match = compute_keyword_match(
                    &case.expected_keywords,
                    &response,
                );
                let forbidden_violations: Vec<String> = case
                    .forbidden_keywords
                    .iter()
                    .filter(|kw| response.to_lowercase().contains(&kw.to_lowercase()))
                    .cloned()
                    .collect();
                let mut notes = Vec::new();
                for expected in &case.expected_tools {
                    if !tools_used.contains(&expected.to_string()) {
                        notes.push(format!("缺少预期工具: {}", expected));
                    }
                }
                for kw in &case.expected_keywords {
                    if !response.to_lowercase().contains(&kw.to_lowercase()) {
                        notes.push(format!("缺少预期关键词: {}", kw));
                    }
                }
                let score = tool_accuracy * 0.4
                    + keyword_match * 0.3
                    + if forbidden_violations.is_empty() { 0.3 } else { 0.0 };
                let passed = score >= 0.6 && forbidden_violations.is_empty();
                EvalResult {
                    case_id: case.id.clone(),
                    passed,
                    tool_accuracy,
                    keyword_match,
                    forbidden_violations,
                    actual_tools: tools_used,
                    actual_response: truncate_str(&response, 500),
                    score,
                    notes,
                }
            })
            .collect()
    }

    pub fn summary(&self, results: &[EvalResult]) -> EvalSummary {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let avg_score = if total > 0 {
            results.iter().map(|r| r.score).sum::<f64>() / total as f64
        } else {
            0.0
        };
        let by_category: HashMap<String, Vec<&EvalResult>> = results
            .iter()
            .filter_map(|r| {
                self.cases
                    .iter()
                    .find(|c| c.id == r.case_id)
                    .map(|c| (c.category.clone(), r))
            })
            .fold(HashMap::new(), |mut acc, (cat, result)| {
                acc.entry(cat).or_default().push(result);
                acc
            });
        let category_scores: HashMap<String, f64> = by_category
            .iter()
            .map(|(cat, results)| {
                let avg = results.iter().map(|r| r.score).sum::<f64>() / results.len() as f64;
                (cat.clone(), avg)
            })
            .collect();
        EvalSummary {
            suite_name: self.name.clone(),
            total_cases: total,
            passed,
            failed: total - passed,
            pass_rate: if total > 0 { passed as f64 / total as f64 } else { 0.0 },
            avg_score,
            category_scores,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalSummary {
    pub suite_name: String,
    pub total_cases: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,
    pub avg_score: f64,
    pub category_scores: HashMap<String, f64>,
}

impl std::fmt::Display for EvalSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "📊 评估报告: {}", self.suite_name)?;
        writeln!(f, "   通过: {}/{} ({:.0}%)", self.passed, self.total_cases, self.pass_rate * 100.0)?;
        writeln!(f, "   平均分: {:.2}", self.avg_score)?;
        for (cat, score) in &self.category_scores {
            writeln!(f, "   {}: {:.2}", cat, score)?;
        }
        Ok(())
    }
}

fn extract_tools_and_response(results: &[(String, String)]) -> (Vec<String>, String) {
    let mut tools = Vec::new();
    let mut response_parts = Vec::new();
    for (key, value) in results {
        if key == "tool_call" {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(value) {
                if let Some(name) = parsed.get("name").and_then(|n| n.as_str()) {
                    tools.push(name.to_string());
                }
            }
        } else {
            response_parts.push(value.clone());
        }
    }
    (tools, response_parts.join("\n"))
}

fn compute_tool_accuracy(expected: &[String], actual: &[String]) -> f64 {
    if expected.is_empty() {
        return 1.0;
    }
    let matched = expected
        .iter()
        .filter(|e| actual.iter().any(|a| a.contains(e.as_str())))
        .count();
    matched as f64 / expected.len() as f64
}

fn compute_keyword_match(expected: &[String], response: &str) -> f64 {
    if expected.is_empty() {
        return 1.0;
    }
    let lower = response.to_lowercase();
    let matched = expected
        .iter()
        .filter(|kw| lower.contains(&kw.to_lowercase()))
        .count();
    matched as f64 / expected.len() as f64
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max).collect();
        format!("{}...", truncated)
    }
}

pub fn builtin_eval_suite() -> EvalSuite {
    EvalSuite::new("i-rs-claw 内置评估集")
        .add_case(EvalCase {
            id: "weight_record".to_string(),
            name: "体重记录".to_string(),
            user_input: "记录体重 70kg".to_string(),
            expected_tools: vec!["i_rs".to_string()],
            expected_keywords: vec!["weight".to_string(), "70".to_string()],
            forbidden_keywords: vec!["错误".to_string(), "失败".to_string()],
            category: "健康追踪".to_string(),
        })
        .add_case(EvalCase {
            id: "mood_stats".to_string(),
            name: "心情统计".to_string(),
            user_input: "这周心情怎么样".to_string(),
            expected_tools: vec!["i_rs".to_string()],
            expected_keywords: vec!["mood".to_string()],
            forbidden_keywords: vec![],
            category: "健康追踪".to_string(),
        })
        .add_case(EvalCase {
            id: "multi_tool".to_string(),
            name: "多工具调用".to_string(),
            user_input: "记录体重70kg，同时记录心情愉快".to_string(),
            expected_tools: vec!["i_rs".to_string()],
            expected_keywords: vec!["weight".to_string(), "mood".to_string()],
            forbidden_keywords: vec![],
            category: "多工具".to_string(),
        })
        .add_case(EvalCase {
            id: "web_search".to_string(),
            name: "网络搜索".to_string(),
            user_input: "今天天气怎么样".to_string(),
            expected_tools: vec!["web_search".to_string()],
            expected_keywords: vec!["天气".to_string(), "温度".to_string()],
            forbidden_keywords: vec![],
            category: "搜索".to_string(),
        })
        .add_case(EvalCase {
            id: "greeting".to_string(),
            name: "闲聊问候".to_string(),
            user_input: "你好".to_string(),
            expected_tools: vec![],
            expected_keywords: vec!["你好".to_string(), "嗨".to_string()],
            forbidden_keywords: vec!["错误".to_string()],
            category: "对话".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_suite_evaluate() {
        let suite = builtin_eval_suite();
        let results = vec![
            ("tool_call".to_string(), r#"{"name": "i_rs"}"#.to_string()),
            ("response".to_string(), "已记录体重 70kg，weight 工具调用成功".to_string()),
        ];
        let eval_results = suite.evaluate(&results);
        assert_eq!(eval_results.len(), suite.cases.len());
    }

    #[test]
    fn test_tool_accuracy() {
        assert_eq!(compute_tool_accuracy(&["i_rs".to_string()], &["i_rs".to_string()]), 1.0);
        assert_eq!(compute_tool_accuracy(&["web_search".to_string()], &["i_rs".to_string()]), 0.0);
        assert_eq!(compute_tool_accuracy(&[], &["i_rs".to_string()]), 1.0);
    }

    #[test]
    fn test_keyword_match() {
        assert_eq!(compute_keyword_match(&["体重".to_string()], "已记录体重 70kg"), 1.0);
        assert_eq!(compute_keyword_match(&["不存在的关键词".to_string()], "一些文本"), 0.0);
        assert_eq!(compute_keyword_match(&[], "任何文本"), 1.0);
    }

    #[test]
    fn test_eval_summary() {
        let suite = builtin_eval_suite();
        let results = vec![
            ("tool_call".to_string(), r#"{"name": "i_rs"}"#.to_string()),
            ("response".to_string(), "已记录体重 70kg，心情愉快".to_string()),
        ];
        let eval_results = suite.evaluate(&results);
        let summary = suite.summary(&eval_results);
        assert_eq!(summary.total_cases, suite.cases.len());
        let display = format!("{}", summary);
        assert!(display.contains("评估报告"));
    }

    #[test]
    fn test_builtin_suite() {
        let suite = builtin_eval_suite();
        assert!(!suite.cases.is_empty());
        assert!(suite.cases.iter().any(|c| c.id == "weight_record"));
    }

    #[test]
    fn test_eval_case_passed() {
        let suite = EvalSuite::new("test")
            .add_case(EvalCase {
                id: "t1".to_string(),
                name: "test".to_string(),
                user_input: "hello".to_string(),
                expected_tools: vec![],
                expected_keywords: vec!["hello".to_string()],
                forbidden_keywords: vec!["error".to_string()],
                category: "test".to_string(),
            });
        let results = vec![("response".to_string(), "hello there".to_string())];
        let eval_results = suite.evaluate(&results);
        assert!(eval_results[0].passed);
    }
}
