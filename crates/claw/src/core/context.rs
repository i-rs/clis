#![allow(dead_code)]

use serde_json::Value;
use std::collections::HashMap;

fn estimate_tokens(text: &str) -> usize {
    let mut tokens = 0usize;
    let mut in_word = false;
    let mut word_len = 0usize;
    for ch in text.chars() {
        if is_cjk(ch) {
            if in_word && word_len > 0 {
                tokens += word_len.div_ceil(4);
                word_len = 0;
                in_word = false;
            }
            tokens += 2;
        } else if ch.is_whitespace() {
            if in_word && word_len > 0 {
                tokens += word_len.div_ceil(4);
                word_len = 0;
            }
            in_word = false;
            tokens += 1;
        } else if ch.is_ascii_punctuation() {
            if in_word && word_len > 0 {
                tokens += word_len.div_ceil(4);
                word_len = 0;
            }
            tokens += 1;
            in_word = false;
        } else {
            word_len += 1;
            in_word = true;
        }
    }
    if in_word && word_len > 0 {
        tokens += word_len.div_ceil(4);
    }
    tokens.max(text.chars().count() / 2).max(1)
}

fn is_cjk(ch: char) -> bool {
    let cp = ch as u32;
    (0x4E00..=0x9FFF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0x3000..=0x303F).contains(&cp)
        || (0x3040..=0x309F).contains(&cp)
        || (0x30A0..=0x30FF).contains(&cp)
        || (0xAC00..=0xD7AF).contains(&cp)
        || (0xFF00..=0xFFEF).contains(&cp)
}

pub struct ContextManager {
    pub max_tokens: usize,
    pub teach_window: usize,
    pub recent_window: usize,
    pub summarizer_threshold: f64,
    /// Minimum messages retained regardless of token budget.
    pub min_retain: usize,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self {
            max_tokens: 8192,
            teach_window: 5,
            recent_window: 15,
            summarizer_threshold: 0.85,
            min_retain: 6,
        }
    }
}

impl ContextManager {
    pub fn for_model(model: &str) -> Self {
        let m = model.to_lowercase();
        let max_tokens = if m.contains("gemini") {
            1_000_000
        } else if m.contains("gpt-4o")
            || m.contains("claude-3.5")
            || m.contains("claude-4")
            || m.contains("sonnet-4")
            || m.contains("opus-4")
            || m.contains("deepseek")
            || m.contains("glm-4")
            || m.contains("qwen")
            || m.contains("llama-3")
            || m.contains("mistral-large")
        {
            128_000
        } else if m.contains("gpt-4") || m.contains("claude-3") || m.contains("command-r") {
            32_000
        } else if m.contains("gpt-3.5") || m.contains("mistral-7b") {
            16_000
        } else {
            32_000
        };
        Self {
            max_tokens,
            ..Default::default()
        }
    }

    pub fn count_tokens(msgs: &[Value]) -> usize {
        let mut total = 0;
        for msg in msgs {
            total += 25;

            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                total += estimate_tokens(content);
            }
            if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                total += role.chars().count().div_ceil(4);
            }
            if let Some(tcs) = msg.get("tool_calls").and_then(|t| t.as_array()) {
                for tc in tcs {
                    if let Some(func) = tc.get("function") {
                        if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                            total += estimate_tokens(name);
                        }
                        if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                            total += estimate_tokens(args);
                        }
                    }
                }
            }
        }
        total.max(100)
    }

    pub fn compress(&self, msgs: &mut Vec<Value>, tool_frequency: &HashMap<String, usize>) {
        if msgs.len() <= 2 {
            return;
        }

        let total_tokens = Self::count_tokens(msgs);

        let (teach_window, recent_window) = if total_tokens > self.max_tokens {
            let reduction = (total_tokens as f64 / self.max_tokens as f64).min(3.0);
            (
                (self.teach_window as f64 / reduction).max(1.0) as usize,
                (self.recent_window as f64 / reduction).max(self.min_retain as f64) as usize,
            )
        } else {
            (self.teach_window, self.recent_window)
        };

        crate::core::engine::smart_compress(
            msgs,
            tool_frequency,
            teach_window,
            recent_window,
            self.min_retain,
        );
    }

    pub fn is_near_limit(&self, msgs: &[Value]) -> bool {
        let total = Self::count_tokens(msgs);
        total as f64 > self.max_tokens as f64 * self.summarizer_threshold
    }

    /// Compute a lightweight structural summary of older messages.
    ///
    /// Extracts key facts (tool results with data, user corrections, decisions)
    /// from messages outside the recent window. This is a pure heuristic summary
    /// that runs without LLM calls, suitable for injecting as a system context note.
    pub fn structural_summary(&self, msgs: &[Value], keep_recent: usize) -> Option<String> {
        if msgs.len() <= keep_recent + 4 {
            return None;
        }

        let mut facts: Vec<String> = Vec::new();

        for msg in &msgs[1..msgs.len().saturating_sub(keep_recent)] {
            let Some(content) = msg.get("content").and_then(|c| c.as_str()) else {
                continue;
            };

            if content.len() < 8 || content.len() > 2000 {
                continue;
            }

            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("");

            match role {
                "tool" => {
                    if content.starts_with("错误") || content.starts_with("Error") {
                        continue;
                    }
                    facts.push(format!("[工具结果] {}", content));
                }
                "user" => {
                    if crate::utils::is_correction_message(content) {
                        facts.push(format!("[用户纠正] {}", content));
                    }
                }
                "assistant" if content.contains("确认") || content.contains("明确") => {
                    facts.push(format!("[决策] {}", content));
                }
                _ => {}
            }
        }

        if facts.is_empty() {
            return None;
        }

        facts.truncate(10);
        Some(format!("[先前上下文摘要]\n{}", facts.join("\n")))
    }

    pub fn context_advisory(&self, msgs: &[Value]) -> String {
        let total = Self::count_tokens(msgs);
        let ratio = total as f64 / self.max_tokens as f64;
        if ratio > 0.9 {
            format!(
                "(上下文占用 {:.0}%，接近限制。建议简化回复，优先引用近期内容。)",
                ratio * 100.0
            )
        } else if ratio > 0.7 {
            format!("(上下文占用 {:.0}%)", ratio * 100.0)
        } else {
            String::new()
        }
    }
}
