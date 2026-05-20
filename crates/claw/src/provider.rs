use crate::llm::{LlmEvent, StreamResult, TokenUsage, ToolCallAcc};
use crate::stats::TokenRecord;
use futures_util::StreamExt;
use serde_json::Value;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

// ── Provider Kind ──

/// Provider identifier used in configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    OpenAI,
    Anthropic,
    Ollama,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::OpenAI => "openai",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Ollama => "ollama",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "anthropic" => ProviderKind::Anthropic,
            "ollama" => ProviderKind::Ollama,
            _ => ProviderKind::OpenAI,
        }
    }

    pub fn all() -> Vec<ProviderKind> {
        vec![ProviderKind::OpenAI, ProviderKind::Anthropic, ProviderKind::Ollama]
    }
}

// ── Abstract Trait ──

/// Abstract LLM provider that handles API-specific streaming logic.
/// Each provider converts the internal OpenAI-format messages
/// to its own API format internally.
#[async_trait::async_trait]
#[allow(dead_code)]
pub trait LlmProvider: Send + Sync {
    fn kind(&self) -> ProviderKind;
    fn model(&self) -> &str;

    /// Stream a chat completion, emitting events to `tx`.
    /// `messages` are in OpenAI-compatible format (role/content/tool_calls).
    /// `tool_schemas` are in OpenAI-compatible format.
    async fn stream_chat(
        &self,
        messages: &[Value],
        tool_schemas: &[Value],
        tx: &UnboundedSender<LlmEvent>,
    ) -> anyhow::Result<StreamResult>;
}

// ── Shared retry helper ──

/// Send an HTTP POST request with exponential backoff retry.
///
/// Retry policy:
/// - 429 Too Many Requests: wait `attempt * 1000 + 500` ms
/// - 5xx Server Error: wait `attempt * 2000` ms
/// - Network errors: wait `attempt * 1000` ms
/// - Max `max_attempts` attempts
///
/// Returns `Ok(response)` on success or when retries exhausted on HTTP errors.
/// Returns `Err(...)` when retries exhausted on network errors.
#[tracing::instrument(skip(client, body, headers))]
async fn send_with_retry(
    max_attempts: u32,
    client: &reqwest::Client,
    url: &str,
    body: &serde_json::Value,
    headers: &[(String, String)],
) -> anyhow::Result<reqwest::Response> {
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        let mut req = client.post(url).json(body);
        for (key, value) in headers {
            req = req.header(key.as_str(), value.as_str());
        }
        match req.send().await {
            Ok(r) if r.status().as_u16() == 429 => {
                let wait_ms = (attempt as u64) * 1000 + 500;
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                if attempt < max_attempts { continue; }
                return Ok(r);
            }
            Ok(r) if r.status().is_server_error() => {
                let wait_ms = (attempt as u64) * 2000;
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                if attempt < max_attempts { continue; }
                return Ok(r);
            }
            Ok(r) => return Ok(r),
            Err(e) => {
                if attempt < max_attempts {
                    let wait_ms = (attempt as u64) * 1000;
                    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                    continue;
                }
                return Err(anyhow::anyhow!("API 请求失败 (重试{}次): {}", attempt, e));
            }
        }
    }
}

// =============================================
// Shared OpenAI-compatible streaming
// =============================================

/// Internal streaming logic shared by OpenAI-compatible providers
/// (OpenAI, Ollama, and any other OpenAI-format endpoints).
async fn openai_stream_chat_impl(
    client: &reqwest::Client,
    url: &str,
    api_key: Option<&str>,
    model: &str,
    provider_kind: &str,
    messages: &[Value],
    tool_schemas: &[Value],
    tx: &UnboundedSender<LlmEvent>,
) -> anyhow::Result<StreamResult> {
    let start = Instant::now();
    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "stream_options": { "include_usage": true },
    });

    if !tool_schemas.is_empty() {
        body["tools"] = Value::Array(tool_schemas.to_vec());
        body["parallel_tool_calls"] = serde_json::Value::Bool(true);
    }

    let body_json = serde_json::to_string(&body).unwrap_or_default();

    let headers: Vec<(String, String)> = if let Some(key) = api_key {
        vec![
            ("Authorization".to_string(), format!("Bearer {}", key)),
            ("HTTP-Referer".to_string(), "https://github.com/i-rs/clis".to_string()),
            ("X-Title".to_string(), "i-rs-claw".to_string()),
        ]
    } else {
        Vec::new()
    };
    let response = send_with_retry(3, client, url, &body, &headers).await?;

    let status = response.status().as_u16();

    if response.status().is_success() {
        let mut stream = response.bytes_stream();
        let mut buf = String::new();
        let mut tool_calls: Vec<ToolCallAcc> = Vec::new();
        let mut reasoning_buf = String::new();
        let mut content_buf = String::new();
        let mut usage: Option<TokenUsage> = None;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("流读取失败: {}", e))?;
            buf.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete SSE lines
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim().to_string();
                buf = buf[pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    if data.trim() == "[DONE]" {
                        continue;
                    }

                    if let Ok(parsed) = serde_json::from_str::<Value>(data.trim()) {
                        // Usage data (final chunk with include_usage)
                        if let Some(usage_data) = parsed.get("usage")
                            && !usage_data.is_null() {
                                usage = Some(TokenUsage {
                                    prompt_tokens: usage_data["prompt_tokens"].as_u64().unwrap_or(0) as u32,
                                    completion_tokens: usage_data["completion_tokens"].as_u64().unwrap_or(0) as u32,
                                    total_tokens: usage_data["total_tokens"].as_u64().unwrap_or(0) as u32,
                                });
                            }

                        if let Some(choices) = parsed["choices"].as_array()
                            && let Some(choice) = choices.first()
                                && let Some(delta) = choice.get("delta") {
                                    // Accumulate reasoning_content (DeepSeek)
                                    if let Some(rc) = delta.get("reasoning_content").and_then(|r| r.as_str()) {
                                        reasoning_buf.push_str(rc);
                                        let _ = tx.send(LlmEvent::Reasoning(rc.to_string()));
                                    }

                                    // Text content
                                    if let Some(text) = delta.get("content").and_then(|c| c.as_str())
                                        && !text.is_empty() {
                                            content_buf.push_str(text);
                                            let _ = tx.send(LlmEvent::Token(text.to_string()));
                                        }

                                    // Tool calls (streaming delta)
                                    if let Some(tcs) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                        for tc in tcs {
                                            let idx = tc
                                                .get("index")
                                                .and_then(|i| i.as_i64())
                                                .unwrap_or(0)
                                                as usize;
                                            if idx >= tool_calls.len() {
                                                tool_calls.resize(idx + 1, ToolCallAcc::default());
                                            }
                                            if let Some(id) = tc.get("id").and_then(|i| i.as_str()) {
                                                tool_calls[idx].id = id.to_string();
                                            }
                                            if let Some(func) = tc.get("function") {
                                                if let Some(name) = func.get("name").and_then(|n| n.as_str()) {
                                                    tool_calls[idx].name = name.to_string();
                                                }
                                                if let Some(args) = func.get("arguments").and_then(|a| a.as_str()) {
                                                    tool_calls[idx].arguments.push_str(args);
                                                }
                                            }
                                        }
                                    }
                                }
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let prompt_tokens = usage.map(|u| u.prompt_tokens).unwrap_or(0);
        let completion_tokens = usage.map(|u| u.completion_tokens).unwrap_or(0);
        let has_tool_calls = !tool_calls.is_empty()
            && tool_calls.iter().any(|tc| !tc.id.is_empty());
        let tool_call_count = if has_tool_calls { tool_calls.len() as u32 } else { 0 };

        // Emit usage record for statistics
        let _ = tx.send(LlmEvent::UsageRecord(TokenRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Local::now().timestamp(),
            agent_id: "default".to_string(),
            model: model.to_string(),
            provider: provider_kind.to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            has_tool_calls,
            tool_call_count,
            react_rounds: 0, // Will be updated by chat_loop if needed
            success: true,
            latency_ms: duration_ms,
            estimated_cost_usd: 0.0, // Estimated by StatsManager on consumption
        }));

        let _ = tx.send(LlmEvent::HttpLog {
            status,
            duration_ms,
            model: model.to_string(),
            prompt_tokens,
            completion_tokens,
            error: None,
            request_body: body_json.clone(),
        });

        if has_tool_calls {
            let mut parsed = Vec::new();
            for tc in &tool_calls {
                let args: Value =
                    serde_json::from_str(&tc.arguments).unwrap_or(serde_json::json!({}));
                parsed.push((
                    ToolCallAcc {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        arguments: tc.arguments.clone(),
                    },
                    args,
                ));
            }
            return Ok(StreamResult::ToolCalls(parsed, reasoning_buf));
        }

        Ok(StreamResult::Text(usage, content_buf))
    } else {
        let text = response.text().await.unwrap_or_default();
        let duration_ms = start.elapsed().as_millis() as u64;
        let _ = tx.send(LlmEvent::HttpLog {
            status,
            duration_ms,
            model: model.to_string(),
            prompt_tokens: 0,
            completion_tokens: 0,
            error: Some(format!("HTTP {}: {}", status, text)),
            request_body: body_json.clone(),
        });
        Err(anyhow::anyhow!("API 返回错误 {}: {}", status, text))
    }
}

// =============================================
// OpenAI Provider
// =============================================

pub struct OpenaiProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenaiProvider {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client, api_key, base_url, model }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenaiProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::OpenAI
    }

    fn model(&self) -> &str {
        &self.model
    }

    #[tracing::instrument(skip(self, messages, tool_schemas, tx))]
    async fn stream_chat(
        &self,
        messages: &[Value],
        tool_schemas: &[Value],
        tx: &UnboundedSender<LlmEvent>,
    ) -> anyhow::Result<StreamResult> {
        let url = format!("{}/chat/completions", self.base_url);
        openai_stream_chat_impl(
            &self.client,
            &url,
            Some(&self.api_key),
            &self.model,
            "openai",
            messages,
            tool_schemas,
            tx,
        )
        .await
    }
}

// =============================================
// Ollama Provider
// =============================================

pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            base_url: "http://localhost:11434/v1".to_string(),
            model,
        }
    }

    #[allow(dead_code)]
    pub fn with_url(base_url: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client, base_url, model }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OllamaProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Ollama
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn stream_chat(
        &self,
        messages: &[Value],
        tool_schemas: &[Value],
        tx: &UnboundedSender<LlmEvent>,
    ) -> anyhow::Result<StreamResult> {
        let url = format!("{}/chat/completions", self.base_url);
        openai_stream_chat_impl(
            &self.client,
            &url,
            None::<&str>,
            &self.model,
            "ollama",
            messages,
            tool_schemas,
            tx,
        )
        .await
    }
}

// =============================================
// Anthropic Provider
// =============================================

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client, api_key, model }
    }
}

/// Convert OpenAI-format messages to Anthropic Messages API format.
fn openai_to_anthropic_messages(messages: &[Value]) -> (Option<String>, Vec<Value>) {
    let mut system = None;
    let mut anthro_msgs: Vec<Value> = Vec::new();
    let mut user_blocks: Vec<Value> = Vec::new();

    // Helper to flush accumulated user content blocks
    let flush_user = |blocks: &mut Vec<Value>, msgs: &mut Vec<Value>| {
        if !blocks.is_empty() {
            msgs.push(serde_json::json!({
                "role": "user",
                "content": blocks
            }));
            blocks.clear();
        }
    };

    for msg in messages {
        let role = msg["role"].as_str().unwrap_or("");
        match role {
            "system" => {
                system = msg["content"].as_str().map(|s| s.to_string());
            }
            "user" => {
                flush_user(&mut user_blocks, &mut anthro_msgs);
                user_blocks.push(serde_json::json!({
                    "type": "text",
                    "text": msg["content"].as_str().unwrap_or("")
                }));
            }
            "assistant" => {
                flush_user(&mut user_blocks, &mut anthro_msgs);
                let mut blocks = Vec::new();
                // Text content
                if let Some(text) = msg["content"].as_str()
                    && !text.is_empty() && text != "null" {
                        blocks.push(serde_json::json!({
                            "type": "text",
                            "text": text
                        }));
                    }
                // Tool use content blocks
                if let Some(tcs) = msg["tool_calls"].as_array() {
                    for tc in tcs {
                        if let Some(func) = tc.get("function") {
                            let name = func["name"].as_str().unwrap_or("");
                            let args_str = func["arguments"].as_str().unwrap_or("{}");
                            let args: Value =
                                serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));
                            blocks.push(serde_json::json!({
                                "type": "tool_use",
                                "id": tc["id"].as_str().unwrap_or(""),
                                "name": name,
                                "input": args
                            }));
                        }
                    }
                }
                // Anthropic requires at least one content block in assistant messages
                if blocks.is_empty() {
                    blocks.push(serde_json::json!({"type": "text", "text": ""}));
                }
                anthro_msgs.push(serde_json::json!({
                    "role": "assistant",
                    "content": blocks
                }));
            }
            "tool" => {
                flush_user(&mut user_blocks, &mut anthro_msgs);
                anthro_msgs.push(serde_json::json!({
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": msg["tool_call_id"].as_str().unwrap_or(""),
                        "content": msg["content"].as_str().unwrap_or("")
                    }]
                }));
            }
            _ => {}
        }
    }

    flush_user(&mut user_blocks, &mut anthro_msgs);
    (system, anthro_msgs)
}

/// Convert OpenAI-compatible tool schemas to Anthropic tool format.
fn openai_to_anthropic_tools(tool_schemas: &[Value]) -> Vec<Value> {
    tool_schemas
        .iter()
        .filter_map(|ts| {
            let func = ts.get("function")?;
            Some(serde_json::json!({
                "name": func["name"].as_str().unwrap_or(""),
                "description": func["description"].as_str().unwrap_or(""),
                "input_schema": func["parameters"],
            }))
        })
        .collect()
}

impl AnthropicProvider {
    /// SSE event line → typed event for the Anthropic stream.
    fn parse_anthropic_event(
        event_type: &str,
        data: &str,
    ) -> Option<AnthropicEvent> {
        let parsed: Value = serde_json::from_str(data).ok()?;
        match event_type {
            "message_start" => {
                let msg = parsed.get("message")?;
                let usage = msg.get("usage").map(|u| TokenUsage {
                        prompt_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
                        completion_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
                        total_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32
                            + u["output_tokens"].as_u64().unwrap_or(0) as u32,
                    });
                Some(AnthropicEvent::MessageStart { usage })
            }
            "content_block_start" => {
                let index = parsed["index"].as_u64()? as usize;
                let block = parsed.get("content_block")?;
                match block["type"].as_str() {
                    Some("text") => Some(AnthropicEvent::ContentBlockStart {
                        index,
                        block_type: "text".to_string(),
                        tool_use_id: None,
                        tool_use_name: None,
                    }),
                    Some("tool_use") => {
                        let id = block["id"].as_str().unwrap_or("").to_string();
                        let name = block["name"].as_str().unwrap_or("").to_string();
                        Some(AnthropicEvent::ContentBlockStart {
                            index,
                            block_type: "tool_use".to_string(),
                            tool_use_id: Some(id),
                            tool_use_name: Some(name),
                        })
                    }
                    _ => None,
                }
            }
            "content_block_delta" => {
                let index = parsed["index"].as_u64()? as usize;
                let delta = parsed.get("delta")?;
                match delta["type"].as_str() {
                    Some("text_delta") => {
                        let text = delta["text"].as_str().unwrap_or("");
                        Some(AnthropicEvent::ContentBlockDelta {
                            index,
                            text: Some(text.to_string()),
                            partial_json: None,
                        })
                    }
                    Some("input_json_delta") => {
                        let partial = delta["partial_json"].as_str().unwrap_or("");
                        Some(AnthropicEvent::ContentBlockDelta {
                            index,
                            text: None,
                            partial_json: Some(partial.to_string()),
                        })
                    }
                    _ => None,
                }
            }
            "content_block_stop" => {
                let index = parsed["index"].as_u64().unwrap_or(0) as usize;
                Some(AnthropicEvent::ContentBlockStop { index })
            }
            "message_delta" => {
                let delta = parsed.get("delta")?;
                let stop_reason = delta["stop_reason"].as_str().unwrap_or("").to_string();
                let usage = parsed.get("usage").map(|u| TokenUsage {
                        prompt_tokens: 0, // Only shown in message_start
                        completion_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
                        total_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
                    });
                Some(AnthropicEvent::MessageDelta { stop_reason, usage })
            }
            "message_stop" => Some(AnthropicEvent::MessageStop),
            "ping" => Some(AnthropicEvent::Ping),
            _ => None,
        }
    }
}

/// Internal event types for Anthropic SSE stream parsing.
#[allow(dead_code)]
enum AnthropicEvent {
    MessageStart {
        usage: Option<TokenUsage>,
    },
    ContentBlockStart {
        index: usize,
        block_type: String,
        tool_use_id: Option<String>,
        tool_use_name: Option<String>,
    },
    ContentBlockDelta {
        index: usize,
        text: Option<String>,
        partial_json: Option<String>,
    },
    ContentBlockStop {
        index: usize,
    },
    MessageDelta {
        stop_reason: String,
        usage: Option<TokenUsage>,
    },
    MessageStop,
    Ping,
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    fn model(&self) -> &str {
        &self.model
    }

    #[tracing::instrument(skip(self, messages, tool_schemas, tx))]
    async fn stream_chat(
        &self,
        messages: &[Value],
        tool_schemas: &[Value],
        tx: &UnboundedSender<LlmEvent>,
    ) -> anyhow::Result<StreamResult> {
        let start = Instant::now();
        let (system_prompt, anthro_msgs) = openai_to_anthropic_messages(messages);
        let anthropic_tools = openai_to_anthropic_tools(tool_schemas);

        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": 8192,
            "stream": true,
            "messages": anthro_msgs,
        });

        if let Some(sys) = &system_prompt {
            body["system"] = Value::String(sys.clone());
        }

        if !anthropic_tools.is_empty() {
            body["tools"] = Value::Array(anthropic_tools);
            body["tool_choice"] = serde_json::json!({"type": "auto"});
        }

        let body_json = serde_json::to_string(&body).unwrap_or_default();

        let headers = vec![
            ("x-api-key".to_string(), self.api_key.clone()),
            ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ];
        let response = send_with_retry(3, &self.client, "https://api.anthropic.com/v1/messages", &body, &headers).await?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            let duration_ms = start.elapsed().as_millis() as u64;
            let _ = tx.send(LlmEvent::HttpLog {
                status,
                duration_ms,
                model: self.model.clone(),
                prompt_tokens: 0,
                completion_tokens: 0,
                error: Some(format!("HTTP {}: {}", status, text)),
                request_body: body_json.clone(),
            });
            return Err(anyhow::anyhow!("Anthropic API 返回错误 {}: {}", status, text));
        }

        // Parse Anthropic SSE event stream
        let mut stream = response.bytes_stream();
        let mut buf = String::new();
        let mut current_event_type = String::new();

        // Track content blocks by index
        // For text blocks: store accumulated text
        // For tool_use blocks: store ToolCallAcc
        #[derive(Default, Clone)]
        struct ContentBlock {
            block_type: String,
            text: String,
            tool_use_id: String,
            tool_use_name: String,
            partial_json: String,
        }
        let mut content_blocks: Vec<ContentBlock> = Vec::new();
        let mut total_usage: Option<TokenUsage> = None;
        let mut final_stop_reason = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| anyhow::anyhow!("流读取失败: {}", e))?;
            buf.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete SSE lines
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim().to_string();
                buf = buf[pos + 1..].to_string();

                if line.is_empty() {
                    continue;
                }

                if let Some(event_val) = line.strip_prefix("event: ") {
                    current_event_type = event_val.to_string();
                    continue;
                }

                if let Some(data_val) = line.strip_prefix("data: ") {
                    let event_type = std::mem::take(&mut current_event_type);
                    if event_type.is_empty() {
                        continue;
                    }

                    if let Some(event) = Self::parse_anthropic_event(&event_type, data_val) {
                        match event {
                            AnthropicEvent::MessageStart { usage } => {
                                total_usage = usage;
                            }
                            AnthropicEvent::ContentBlockStart {
                                index,
                                block_type,
                                tool_use_id,
                                tool_use_name,
                            } => {
                                if index >= content_blocks.len() {
                                    content_blocks.resize(index + 1, ContentBlock::default());
                                }
                                content_blocks[index].block_type = block_type.clone();
                                if block_type == "tool_use" {
                                    content_blocks[index].tool_use_id =
                                        tool_use_id.unwrap_or_default();
                                    content_blocks[index].tool_use_name =
                                        tool_use_name.unwrap_or_default();
                                }
                            }
                            AnthropicEvent::ContentBlockDelta {
                                index,
                                text,
                                partial_json,
                            } => {
                                if index >= content_blocks.len() {
                                    content_blocks.resize(index + 1, ContentBlock::default());
                                }
                                if let Some(t) = text {
                                    content_blocks[index].text.push_str(&t);
                                    let _ = tx.send(LlmEvent::Token(t));
                                }
                                if let Some(pj) = partial_json {
                                    content_blocks[index].partial_json.push_str(&pj);
                                }
                            }
                            AnthropicEvent::ContentBlockStop { .. } => {
                                // Content block complete — nothing special needed
                            }
                            AnthropicEvent::MessageDelta {
                                stop_reason,
                                usage,
                            } => {
                                final_stop_reason = stop_reason;
                                if let Some(u) = usage {
                                    let prompt = total_usage
                                        .as_ref()
                                        .map(|tu| tu.prompt_tokens)
                                        .unwrap_or(0);
                                    total_usage = Some(TokenUsage {
                                        prompt_tokens: prompt,
                                        completion_tokens: u.completion_tokens,
                                        total_tokens: prompt + u.completion_tokens,
                                    });
                                }
                            }
                            AnthropicEvent::MessageStop => {
                                // Stream complete
                            }
                            AnthropicEvent::Ping => {}
                        }
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        let usage = total_usage.as_ref();
        let prompt_tokens = usage.map(|u| u.prompt_tokens).unwrap_or(0);
        let completion_tokens = usage.map(|u| u.completion_tokens).unwrap_or(0);
        let has_tool_calls = final_stop_reason == "tool_use";
        let tool_call_count = if has_tool_calls {
            content_blocks.iter().filter(|b| b.block_type == "tool_use").count() as u32
        } else {
            0
        };

        // Emit usage record for statistics
        let _ = tx.send(LlmEvent::UsageRecord(TokenRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Local::now().timestamp(),
            agent_id: "default".to_string(),
            model: self.model.clone(),
            provider: "anthropic".to_string(),
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            has_tool_calls,
            tool_call_count,
            react_rounds: 0,
            success: true,
            latency_ms: duration_ms,
            estimated_cost_usd: 0.0,
        }));

        let _ = tx.send(LlmEvent::HttpLog {
            status,
            duration_ms,
            model: self.model.clone(),
            prompt_tokens,
            completion_tokens,
            error: None,
            request_body: body_json.clone(),
        });

        // Determine result type based on stop reason
        if has_tool_calls {
            // Extract tool_use blocks
            let mut parsed = Vec::new();
            for block in &content_blocks {
                if block.block_type == "tool_use" {
                    let args: Value = serde_json::from_str(&block.partial_json)
                        .unwrap_or(serde_json::json!({}));
                    parsed.push((
                        ToolCallAcc {
                            id: block.tool_use_id.clone(),
                            name: block.tool_use_name.clone(),
                            arguments: block.partial_json.clone(),
                        },
                        args,
                    ));
                }
            }
            Ok(StreamResult::ToolCalls(parsed, String::new()))
        } else {
            // Accumulate all text content blocks
            let text: String = content_blocks
                .iter()
                .map(|b| b.text.clone())
                .collect::<Vec<_>>()
                .join("");
            Ok(StreamResult::Text(total_usage, text))
        }
    }
}

// =============================================
// Factory
// =============================================

/// Create the appropriate provider based on configuration.
pub fn create_provider(config: &crate::config::Config) -> Box<dyn LlmProvider> {
    match ProviderKind::from_str(&config.provider) {
        ProviderKind::OpenAI => Box::new(OpenaiProvider::new(
            config.api_key.clone(),
            config.base_url.clone(),
            config.model.clone(),
        )),
        ProviderKind::Anthropic => Box::new(AnthropicProvider::new(
            config.api_key.clone(),
            config.model.clone(),
        )),
        ProviderKind::Ollama => Box::new(OllamaProvider::new(config.model.clone())),
    }
}

/// Create a provider from a resolved agent config.
pub fn create_provider_for(
    provider_type: &str,
    api_key: &str,
    base_url: &str,
    model: &str,
) -> Box<dyn LlmProvider> {
    match ProviderKind::from_str(provider_type) {
        ProviderKind::OpenAI => Box::new(OpenaiProvider::new(
            api_key.to_string(),
            base_url.to_string(),
            model.to_string(),
        )),
        ProviderKind::Anthropic => Box::new(AnthropicProvider::new(
            api_key.to_string(),
            model.to_string(),
        )),
        ProviderKind::Ollama => Box::new(OllamaProvider::new(model.to_string())),
    }
}
