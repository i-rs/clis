use crate::llm::TokenUsage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    User {
        text: String,
    },
    Assistant {
        text: String,
        #[serde(default)]
        reasoning: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        token_usage: Option<TokenUsage>,
    },
    ToolCall {
        name: String,
        args: String,
        result: String,
        #[serde(default)]
        step: usize,
        #[serde(default)]
        total_steps: usize,
    },
    Error {
        text: String,
    },
    Evaluation {
        tool: String,
        valid: bool,
        issues: Vec<String>,
    },
    Quality {
        score: Option<f64>,
        complete: bool,
        references_valid: u32,
        issues: Vec<String>,
    },
    Feedback {
        positive: bool,
        message: Option<String>,
    },
    Image {
        path: String,
        alt_text: String,
        width: u32,
        height: u32,
        format: String,
    },
}

pub fn message_to_jsonl(msg: &Message) -> Value {
    serde_json::to_value(msg).unwrap_or_default()
}

pub fn message_from_jsonl(v: Value) -> Option<Message> {
    serde_json::from_value(v).ok()
}

pub fn evaluate_response_heuristic(
    response_text: &str,
    tool_results: &[(&str, bool)],
    known_tools: &[&str],
) -> Message {
    let mut issues = Vec::new();
    let mut references_valid = 0u32;

    let executed_tools: HashSet<&str> = tool_results.iter().map(|(n, _)| *n).collect();
    for (name, success) in tool_results {
        if response_text.contains(*name) {
            references_valid += 1;
        }
        if !(*success || response_text.contains("错误") || response_text.contains("失败")) {
            issues.push(format!("工具 '{}' 执行失败，但回复未提及", name));
        }
    }

    let mut mentioned_but_not_executed: Vec<&str> = Vec::new();
    for pattern in known_tools {
        if response_text.contains(*pattern) && !executed_tools.contains(pattern) {
            mentioned_but_not_executed.push(pattern);
        }
    }
    if mentioned_but_not_executed.len() > 3 {
        // Only flag as an issue when the AI mentions many tools it didn't execute,
        // suggesting possible hallucination. Mentioning 1-3 tools is normal conversation.
        issues.push(format!(
            "回复提及未执行的工具: {}",
            mentioned_but_not_executed.join(", ")
        ));
    }

    let complete = !response_text.trim().is_empty();
    if !complete {
        issues.push("回复为空".to_string());
    }

    let relevance = compute_text_relevance(response_text);
    if relevance < 0.3 && !response_text.is_empty() {
        issues.push(format!(
            "回复信息密度较低 (相关度: {:.0}%)",
            relevance * 100.0
        ));
    }

    let has_errors = !issues.is_empty();
    let score = if has_errors {
        let raw = 1.0 - (issues.len() as f64 * 0.2).min(0.8);
        Some((raw * 100.0).round() / 100.0)
    } else if complete {
        Some(1.0)
    } else {
        Some(0.0)
    };

    Message::Quality {
        score,
        complete,
        references_valid,
        issues,
    }
}

fn compute_text_relevance(text: &str) -> f64 {
    if text.is_empty() {
        return 0.0;
    }

    let mut meaningful = 0usize;
    let mut total_chars = 0usize;
    for c in text.chars() {
        total_chars += 1;
        if c.is_alphanumeric() || c > '\x7f' {
            meaningful += 1;
        }
    }
    if total_chars == 0 {
        return 0.0;
    }

    static STOPWORDS: &[&str] = &[
        "的", "了", "在", "是", "我", "有", "和", "就", "不", "都", "the", "a", "an", "is", "are",
        "was", "were", "be", "been", "have", "has", "had", "do", "does", "did", "will", "would",
        "could", "should", "may", "might", "can", "shall", "to", "of", "in", "for", "on", "with",
        "at", "by", "from", "as", "into", "through", "during", "before", "after", "above", "below",
        "between", "and", "but", "or", "nor", "not", "so", "yet", "both", "either", "each",
        "every", "all", "any", "few", "more", "most", "other", "some", "such", "only", "own",
        "same", "than", "too", "very", "just", "because", "about", "up", "out", "if", "then",
        "now", "it", "its", "he", "she", "they", "them", "this", "that", "these", "those", "what",
        "which", "who", "whom", "how",
    ];

    let mut stopword_count = 0usize;
    let mut total_words = 0usize;
    for w in text.split_whitespace() {
        total_words += 1;
        if w.len() <= 4
            && STOPWORDS
                .iter()
                .any(|&s| w.eq_ignore_ascii_case(s) || w.contains(s))
        {
            stopword_count += 1;
        }
    }

    let meaningful_ratio = meaningful as f64 / total_chars as f64;
    let stopword_ratio = stopword_count as f64 / total_words.max(1) as f64;

    (meaningful_ratio * 0.6 + (1.0 - stopword_ratio) * 0.4).min(1.0)
}

#[derive(Clone)]
pub struct PluginEntry {
    pub name: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub description: String,
    pub done: bool,
}

#[derive(Debug, Clone)]
pub struct HttpLog {
    pub timestamp: String,
    pub status: u16,
    pub duration_ms: u64,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub error: Option<String>,
    pub request_body: String,
    pub msg_count: usize,
}
