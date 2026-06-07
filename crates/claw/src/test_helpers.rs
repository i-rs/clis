//! Test helpers for i-rs-claw.
//!
//! Provides shared test utilities used across multiple test modules:
//! - `test_config()`: minimal Config without file/env dependencies
//! - `MockProvider`: deterministic mock for chat_loop testing
//! - `test_core()`: AppCore with temporary data directory
//!
//! NOTE: This file is compiled only under #[cfg(test)] (see main.rs).

use i_rs_claw_core::config::Config;
use i_rs_claw_core::llm::{LlmEvent, StreamResult, TokenUsage};
use i_rs_claw_core::providers::ProviderKind;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

/// Create a minimal test Config that doesn't read from filesystem or environment vars.
pub fn test_config() -> Config {
    let mut c = Config::new();
    c.api_key = "test-key".to_string();
    c.provider = ProviderKind::OpenAI;
    c.base_url = "http://localhost:9999/v1".to_string();
    c.model = "test-model".to_string();
    c
}

/// Mock LLM provider that returns a predetermined sequence of events.
///
/// Usage:
/// ```
/// let provider = MockProvider::new(vec![
///     LlmEvent::Token("hello".to_string()),
/// ]);
/// ```
#[allow(dead_code)]
pub struct MockProvider {
    kind: ProviderKind,
    model: String,
    events: Vec<LlmEvent>,
    /// If Some, return this result instead of sending events.
    result_override: Option<anyhow::Result<StreamResult>>,
}

impl MockProvider {
    pub fn new(events: Vec<LlmEvent>) -> Self {
        Self {
            kind: ProviderKind::OpenAI,
            model: "mock".to_string(),
            events,
            result_override: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_kind(mut self, kind: ProviderKind) -> Self {
        self.kind = kind;
        self
    }

    #[allow(dead_code)]
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Force the provider to return this result instead of sending events.
    /// Useful for simulating errors.
    pub fn with_result(mut self, result: anyhow::Result<StreamResult>) -> Self {
        self.result_override = Some(result);
        self
    }
}

#[async_trait::async_trait]
impl i_rs_claw_core::providers::LlmProvider for MockProvider {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn stream_chat(
        &self,
        _messages: &[Value],
        _tool_schemas: &[Value],
        tx: &UnboundedSender<LlmEvent>,
        _trace_id: &str,
    ) -> anyhow::Result<StreamResult> {
        if let Some(ref result) = self.result_override {
            let result: anyhow::Result<StreamResult> = match result {
                Ok(r) => Ok(r.clone()),
                Err(e) => Err(anyhow::anyhow!("{}", e)),
            };
            return result;
        }

        // Send all events
        for event in &self.events {
            let _ = tx.send(event.clone());
        }

        // Determine the StreamResult based on events
        let has_tool_calls = self
            .events
            .iter()
            .any(|e| matches!(e, LlmEvent::ToolExecuted { .. }));
        if has_tool_calls {
            Ok(StreamResult::ToolCalls(
                Vec::new(),
                String::new(),
                String::new(),
            ))
        } else {
            Ok(StreamResult::Text(None, String::new(), String::new()))
        }
    }
}

/// Create a test AppCore with a temporary data directory.
///
/// Uses `AppCore::with_claw_dir()` with a temp dir so tests
/// don't touch the real user's data. The temp dir is cleaned up
/// on drop.
#[allow(dead_code)]
pub fn test_core() -> (Config, i_rs_claw_core::core::AppCore) {
    let dir = tempfile::tempdir().expect("创建临时目录失败");
    let claw_dir = dir.path().join(".i-rs").join("claw");
    std::fs::create_dir_all(&claw_dir).expect("创建 claw 数据目录失败");

    let config = test_config();
    let core =
        i_rs_claw_core::core::AppCore::with_claw_dir(config.clone(), claw_dir).expect("AppCore 初始化失败");

    (config, core)
}

/// Parse a single OpenAI-format SSE JSON chunk and produce events.
///
/// Returns parsed delta events (Token, Reasoning) plus optional
/// usage data and tool call accumulation info. This is a pure
/// function version of the parsing logic inside `openai_stream_chat_impl`.
#[allow(dead_code)]
pub fn parse_openai_sse_chunk(data: &Value) -> ParseResult {
    let mut result = ParseResult::default();

    // Usage data (final chunk with include_usage)
    if let Some(usage_data) = data
        .get("usage")
        .and_then(|u| if u.is_null() { None } else { Some(u) })
    {
        result.usage = Some(TokenUsage {
            prompt_tokens: usage_data["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: usage_data["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: usage_data["total_tokens"].as_u64().unwrap_or(0) as u32,
            estimated_cost_usd: None,
        });
    }

    if let Some(choices) = data["choices"].as_array()
        && let Some(choice) = choices.first()
        && let Some(delta) = choice.get("delta")
    {
        // Reasoning content (DeepSeek)
        if let Some(rc) = delta.get("reasoning_content").and_then(|r| r.as_str()) {
            result.reasoning.push_str(rc);
            result.events.push(LlmEvent::Reasoning(rc.to_string()));
        }

        // Text content
        if let Some(text) = delta.get("content").and_then(|c| c.as_str())
            && !text.is_empty()
        {
            result.content.push_str(text);
            result.events.push(LlmEvent::Token(text.to_string()));
        }

        // Tool calls
        if let Some(tcs) = delta.get("tool_calls").and_then(|t| t.as_array()) {
            for tc in tcs {
                let tc_idx = tc.get("index").and_then(|i| i.as_i64()).unwrap_or(0) as usize;
                if tc_idx >= result.tool_calls.len() {
                    result
                        .tool_calls
                        .resize(tc_idx + 1, i_rs_claw_core::llm::ToolCallAcc::default());
                }
                if let Some(id) = tc.get("id").and_then(|i| i.as_str()) {
                    result.tool_calls[tc_idx].id = id.to_string();
                }
                if let Some(func) = tc.get("function") {
                    if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                        result.tool_calls[tc_idx].name = name.to_string();
                    }
                    if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                        result.tool_calls[tc_idx].arguments.push_str(args);
                    }
                }
            }
        }
    }

    result
}

/// Result of parsing a single OpenAI SSE chunk.
#[derive(Default)]
pub struct ParseResult {
    pub events: Vec<LlmEvent>,
    pub reasoning: String,
    pub content: String,
    pub usage: Option<TokenUsage>,
    pub tool_calls: Vec<i_rs_claw_core::llm::ToolCallAcc>,
}

/// Parse an OpenAI SSE stream text and return all emitted LlmEvents.
///
/// Processes `data: ...` lines from the SSE text, simulating the
/// parsing logic in `openai_stream_chat_impl`.
#[allow(dead_code)]
pub fn parse_openai_sse(sse_text: &str) -> (Vec<LlmEvent>, Option<TokenUsage>) {
    let mut all_events = Vec::new();
    let mut final_usage = None;

    for line in sse_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Strip "data: " prefix
        let data = match trimmed.strip_prefix("data: ") {
            Some(d) => d.trim(),
            None => continue,
        };

        // Skip [DONE] signal
        if data == "[DONE]" {
            continue;
        }

        // Parse JSON
        if let Ok(parsed) = serde_json::from_str::<Value>(data) {
            let chunk_result = parse_openai_sse_chunk(&parsed);
            all_events.extend(chunk_result.events);
            if chunk_result.usage.is_some() {
                final_usage = chunk_result.usage;
            }
        }
    }

    (all_events, final_usage)
}
