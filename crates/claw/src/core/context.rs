#![allow(dead_code)]

use serde_json::Value;
use std::collections::HashMap;

/// Estimate token count for a string, accounting for CJK characters.
/// CJK characters are ~1-2 tokens each vs ~0.25 tokens per ASCII char.
fn estimate_tokens(text: &str) -> usize {
    let mut cjk_count = 0usize;
    let mut ascii_len = 0usize;
    for ch in text.chars() {
        if is_cjk(ch) {
            cjk_count += 1;
        } else {
            ascii_len += ch.len_utf8();
        }
    }
    // CJK: ~1.5 tokens per character, ASCII: ~4 chars per token
    cjk_count + (cjk_count / 2) + (ascii_len / 4)
}

fn is_cjk(ch: char) -> bool {
    let cp = ch as u32;
    (0x4E00..=0x9FFF).contains(&cp)      // CJK Unified Ideographs
        || (0x3400..=0x4DBF).contains(&cp) // CJK Extension A
        || (0x3000..=0x303F).contains(&cp) // CJK Symbols and Punctuation
        || (0x3040..=0x309F).contains(&cp) // Hiragana
        || (0x30A0..=0x30FF).contains(&cp) // Katakana
        || (0xAC00..=0xD7AF).contains(&cp) // Hangul Syllables
        || (0xFF00..=0xFFEF).contains(&cp) // Fullwidth Forms
}

/// Manages context window with token-aware compression.
///
/// Enhances the existing smart_compress with:
/// - Token counting for adaptive window sizing
/// - Budget-driven compression (adjust windows based on total tokens)
/// - Summarization fallback when context exceeds limits
pub struct ContextManager {
    /// Model-dependent context limit (estimated).
    pub max_tokens: usize,
    /// Maximum teach documents to preserve.
    pub teach_window: usize,
    /// Number of recent messages to preserve intact.
    pub recent_window: usize,
    /// Token ratio threshold for triggering summarization (0.0-1.0).
    pub summarizer_threshold: f64,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self {
            // Most models have 8K-128K context; use conservative 8K as baseline
            max_tokens: 8192,
            teach_window: 5,
            recent_window: 15,
            summarizer_threshold: 0.85,
        }
    }
}

impl ContextManager {
    /// Create a manager tuned for a specific model context size.
    pub fn for_model(model: &str) -> Self {
        let m = model.to_lowercase();
        let max_tokens = if m.contains("gemini") {
            1_000_000
        } else if m.contains("gpt-4o")
            || m.contains("claude-3.5")
            || m.contains("claude-4")
            || m.contains("sonnet-4")
            || m.contains("opus-4")
        {
            128_000
        } else if m.contains("deepseek")
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

    /// Count approximate tokens in a message list.
    /// Uses heuristic: ~4 chars per token for ASCII, ~1.5 chars per token for CJK,
    /// plus message overhead.
    pub fn count_tokens(msgs: &[Value]) -> usize {
        let mut total = 0;
        for msg in msgs {
            total += 25;

            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                total += estimate_tokens(content);
            }
            if let Some(role) = msg.get("role").and_then(|r| r.as_str()) {
                total += role.len() / 4;
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

    /// Adaptive compression based on token budget.
    ///
    /// Tunes teach_window and recent_window based on total token count,
    /// then delegates to the existing smart_compress logic.
    pub fn compress(&self, msgs: &mut Vec<Value>, tool_frequency: &HashMap<String, usize>) {
        if msgs.len() <= 2 {
            return;
        }

        let total_tokens = Self::count_tokens(msgs);

        // Adapt windows based on token budget
        let (teach_window, recent_window) = if total_tokens > self.max_tokens {
            // Over budget: reduce windows
            let reduction = (total_tokens as f64 / self.max_tokens as f64).min(3.0);
            (
                (self.teach_window as f64 / reduction).max(2.0) as usize,
                (self.recent_window as f64 / reduction).max(5.0) as usize,
            )
        } else {
            (self.teach_window, self.recent_window)
        };

        // Delegate to the existing smart_compress with adapted windows
        crate::core::engine::smart_compress(msgs, tool_frequency, teach_window, recent_window);
    }

    /// Check if the message list is approaching the context limit.
    pub fn is_near_limit(&self, msgs: &[Value]) -> bool {
        let total = Self::count_tokens(msgs);
        total as f64 > self.max_tokens as f64 * self.summarizer_threshold
    }

    /// Build a compression advisory for the system prompt.
    /// Makes the LLM aware of context management.
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
