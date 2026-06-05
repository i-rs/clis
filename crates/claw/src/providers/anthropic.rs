use crate::llm::{LlmEvent, StreamResult, TokenUsage, ToolCallAcc};
use crate::providers::sse::send_with_retry;
use crate::providers::{LlmProvider, ProviderKind};
use futures_util::StreamExt;
use serde_json::Value;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

// =============================================
// Anthropic Provider
// =============================================

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(client: reqwest::Client, api_key: String, base_url: String, model: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            client,
            api_key,
            base_url,
            model,
        }
    }
}

/// Convert OpenAI-format messages to Anthropic Messages API format.
fn openai_to_anthropic_messages(messages: &[Value]) -> (Option<String>, Vec<Value>) {
    let mut system: Option<String> = None;
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
                let content = msg["content"].as_str().unwrap_or("");
                if let Some(ref mut s) = system {
                    s.push_str("\n\n");
                    s.push_str(content);
                } else {
                    system = Some(content.to_string());
                }
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
                    && !text.is_empty()
                    && text != "null"
                {
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
                            let args: Value = serde_json::from_str(args_str).unwrap_or_else(|e| {
                                tracing::warn!(
                                    "工具 '{}' 参数 JSON 解析失败 (消息转换): {}",
                                    name,
                                    e
                                );
                                serde_json::json!({})
                            });
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

/// Internal event types for Anthropic SSE stream parsing.
#[derive(Debug)]
pub(crate) enum AnthropicEvent {
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
        #[allow(dead_code)]
        index: usize,
    },
    MessageDelta {
        stop_reason: String,
        usage: Option<TokenUsage>,
    },
    MessageStop,
    Ping,
}

impl AnthropicProvider {
    /// SSE event line → typed event for the Anthropic stream.
    pub(crate) fn parse_anthropic_event(event_type: &str, data: &str) -> Option<AnthropicEvent> {
        let parsed: Value = serde_json::from_str(data).ok()?;
        match event_type {
            "message_start" => {
                let msg = parsed.get("message")?;
                let usage = msg.get("usage").map(|u| TokenUsage {
                    prompt_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
                    completion_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
                    estimated_cost_usd: None,
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
                    estimated_cost_usd: None,
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
        trace_id: &str,
    ) -> anyhow::Result<StreamResult> {
        let start = Instant::now();
        let (system_prompt, anthro_msgs) = openai_to_anthropic_messages(messages);
        let anthropic_tools = openai_to_anthropic_tools(tool_schemas);

        let mut body = serde_json::json!({
            "model": self.model,
            "max_tokens": 16384,
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
        let response = send_with_retry(
            3,
            &self.client,
            &format!("{}/messages", self.base_url),
            &body,
            &headers,
        )
        .await?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            let duration_ms = start.elapsed().as_millis() as u64;
            super::common::emit_http_log(
                tx,
                status,
                duration_ms,
                &self.model,
                0,
                0,
                Some(format!("HTTP {}: {}", status, text)),
                &body_json,
            );
            return Err(anyhow::anyhow!(
                "Anthropic API 返回错误 {}: {}",
                status,
                text
            ));
        }

        // Parse Anthropic SSE event stream
        let mut stream = response.bytes_stream();
        let mut buf: Vec<u8> = Vec::new();
        let mut current_event_type = String::new();

        // Track content blocks by index
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
            buf.extend_from_slice(&chunk);

            while let Some(pos) = buf.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = buf.drain(..=pos).collect();
                let line = String::from_utf8_lossy(&line_bytes).trim().to_string();

                if line.is_empty() {
                    continue;
                }

                if let Some(event_val) = line.strip_prefix("event: ") {
                    current_event_type = event_val.to_string();
                    continue;
                }

                if let Some(data_val) = line.strip_prefix("data: ") {
                    let raw_event_type = std::mem::take(&mut current_event_type);
                    let event_type = if raw_event_type.is_empty() {
                        "message"
                    } else {
                        &raw_event_type
                    };

                    if let Some(event) = Self::parse_anthropic_event(event_type, data_val) {
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
                            AnthropicEvent::ContentBlockStop { .. } => {}
                            AnthropicEvent::MessageDelta { stop_reason, usage } => {
                                final_stop_reason = stop_reason;
                                if let Some(u) = usage {
                                    let prompt = total_usage
                                        .as_ref()
                                        .map(|tu| tu.prompt_tokens)
                                        .unwrap_or(0);
                                    total_usage = Some(TokenUsage {
                                        prompt_tokens: prompt,
                                        completion_tokens: u.completion_tokens,
                                        estimated_cost_usd: None,
            total_tokens: prompt + u.completion_tokens,
                                    });
                                }
                            }
                            AnthropicEvent::MessageStop => {}
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
            content_blocks
                .iter()
                .filter(|b| b.block_type == "tool_use")
                .count() as u32
        } else {
            0
        };

        super::common::emit_usage_record(
            tx,
            &self.model,
            "anthropic",
            prompt_tokens,
            completion_tokens,
            has_tool_calls,
            tool_call_count,
            duration_ms,
            trace_id,
        );

        super::common::emit_http_log(
            tx,
            status,
            duration_ms,
            &self.model,
            prompt_tokens,
            completion_tokens,
            None,
            &body_json,
        );

        // Determine result type based on stop reason
        if has_tool_calls {
            let mut parsed = Vec::new();
            let mut prose = String::new();
            for block in &content_blocks {
                if block.block_type == "tool_use" {
                    let args: Value =
                        serde_json::from_str(&block.partial_json).unwrap_or_else(|e| {
                            tracing::warn!(
                                "工具 '{}' 参数 JSON 解析失败: {}",
                                block.tool_use_name,
                                e
                            );
                            serde_json::json!({})
                        });
                    parsed.push((
                        ToolCallAcc {
                            id: block.tool_use_id.clone(),
                            name: block.tool_use_name.clone(),
                            arguments: block.partial_json.clone(),
                        },
                        args,
                    ));
                } else {
                    prose.push_str(&block.text);
                }
            }
            Ok(StreamResult::ToolCalls(parsed, prose, String::new()))
        } else {
            let text: String = content_blocks
                .iter()
                .map(|b| b.text.clone())
                .collect::<Vec<_>>()
                .join("");
            Ok(StreamResult::Text(total_usage, text, String::new()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_anthropic_content_block_start() {
        let event = AnthropicProvider::parse_anthropic_event(
            "content_block_start",
            r#"{"index":0,"content_block":{"type":"text"}}"#,
        );
        assert!(
            matches!(
                event,
                Some(AnthropicEvent::ContentBlockStart {
                    index: 0,
                    ref block_type,
                    tool_use_id: None,
                    tool_use_name: None,
                }) if block_type == "text"
            ),
            "预期 ContentBlockStart(text)，得到 {:?}",
            event
        );
    }

    #[test]
    fn test_parse_anthropic_tool_use_start() {
        let event = AnthropicProvider::parse_anthropic_event(
            "content_block_start",
            r#"{"index":1,"content_block":{"type":"tool_use","id":"toolu_1","name":"get_weather"}}"#,
        );
        assert!(
            matches!(
                event,
                Some(AnthropicEvent::ContentBlockStart {
                    index: 1,
                    ref block_type,
                    tool_use_id: Some(ref id),
                    tool_use_name: Some(ref name),
                }) if block_type == "tool_use" && id == "toolu_1" && name == "get_weather"
            ),
            "预期 ContentBlockStart(tool_use)，得到 {:?}",
            event
        );
    }

    #[test]
    fn test_parse_anthropic_text_delta() {
        let event = AnthropicProvider::parse_anthropic_event(
            "content_block_delta",
            r#"{"index":0,"delta":{"type":"text_delta","text":"Hello"}}"#,
        );
        assert!(
            matches!(
                event,
                Some(AnthropicEvent::ContentBlockDelta {
                    index: 0,
                    text: Some(ref t),
                    partial_json: None,
                }) if t == "Hello"
            ),
            "预期 ContentBlockDelta(text=Hello)，得到 {:?}",
            event
        );
    }

    #[test]
    fn test_parse_anthropic_input_json_delta() {
        let event = AnthropicProvider::parse_anthropic_event(
            "content_block_delta",
            r#"{"index":0,"delta":{"type":"input_json_delta","partial_json":"{\"city\":\"Beijing\"}"}}"#,
        );
        assert!(
            matches!(
                event,
                Some(AnthropicEvent::ContentBlockDelta {
                    index: 0,
                    text: None,
                    partial_json: Some(ref pj),
                }) if pj == r#"{"city":"Beijing"}"#
            ),
            "预期 ContentBlockDelta(partial_json)，得到 {:?}",
            event
        );
    }

    #[test]
    fn test_parse_anthropic_message_start() {
        let event = AnthropicProvider::parse_anthropic_event(
            "message_start",
            r#"{"message":{"usage":{"input_tokens":15,"output_tokens":3}}}"#,
        );
        assert!(
            matches!(
                event,
                Some(AnthropicEvent::MessageStart { usage: Some(u) }) if u.prompt_tokens == 15 && u.completion_tokens == 3
            ),
            "预期 MessageStart(usage=15/3)，得到 {:?}",
            event
        );
    }

    #[test]
    fn test_parse_anthropic_message_stop() {
        let event = AnthropicProvider::parse_anthropic_event("message_stop", r#"{}"#);
        assert!(
            matches!(event, Some(AnthropicEvent::MessageStop)),
            "预期 MessageStop，得到 {:?}",
            event
        );
    }

    #[test]
    fn test_parse_anthropic_ping() {
        let event = AnthropicProvider::parse_anthropic_event("ping", r#"{}"#);
        assert!(
            matches!(event, Some(AnthropicEvent::Ping)),
            "预期 Ping，得到 {:?}",
            event
        );
    }

    #[test]
    fn test_parse_anthropic_message_delta() {
        let event = AnthropicProvider::parse_anthropic_event(
            "message_delta",
            r#"{"delta":{"stop_reason":"tool_use"},"usage":{"output_tokens":42}}"#,
        );
        assert!(
            matches!(
                event,
                Some(AnthropicEvent::MessageDelta {
                    ref stop_reason,
                    usage: Some(u),
                }) if stop_reason == "tool_use" && u.completion_tokens == 42
            ),
            "预期 MessageDelta，得到 {:?}",
            event
        );
    }

    #[test]
    fn test_parse_anthropic_unknown_event() {
        let event = AnthropicProvider::parse_anthropic_event("unknown_event", r#"{}"#);
        assert!(matches!(event, None), "未知事件类型应返回 None");
    }

    // ── openai_to_anthropic 消息转换测试 ──

    #[test]
    fn test_openai_to_anthropic_messages_with_system() {
        let msgs = vec![
            json!({"role": "system", "content": "You are a helpful assistant."}),
            json!({"role": "user", "content": "Hello"}),
        ];
        let (system, anthro) = openai_to_anthropic_messages(&msgs);
        assert_eq!(system.as_deref(), Some("You are a helpful assistant."));
        assert_eq!(anthro.len(), 1);
        assert_eq!(anthro[0]["role"], "user");
        assert_eq!(anthro[0]["content"][0]["type"], "text");
        assert_eq!(anthro[0]["content"][0]["text"], "Hello");
    }

    #[test]
    fn test_openai_to_anthropic_tool_result() {
        let msgs = vec![
            json!({"role": "assistant", "content": "", "tool_calls": [{"id": "call_1", "type": "function", "function": {"name": "get_weather", "arguments": "{}"}}]}),
            json!({"role": "tool", "tool_call_id": "call_1", "content": "\"Sunny\""}),
        ];
        let (system, anthro) = openai_to_anthropic_messages(&msgs);
        assert!(system.is_none());
        assert_eq!(anthro.len(), 2, "应有两个消息: assistant + tool_result");
        assert_eq!(anthro[1]["role"], "user");
        assert_eq!(anthro[1]["content"][0]["type"], "tool_result");
        assert_eq!(anthro[1]["content"][0]["tool_use_id"], "call_1");
    }

    #[test]
    fn test_openai_to_anthropic_tools_format() {
        let schemas = vec![json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather info",
                "parameters": {
                    "type": "object",
                    "properties": {"city": {"type": "string"}}
                }
            }
        })];
        let anthro_tools = openai_to_anthropic_tools(&schemas);
        assert_eq!(anthro_tools.len(), 1);
        assert_eq!(anthro_tools[0]["name"], "get_weather");
        assert_eq!(anthro_tools[0]["description"], "Get weather info");
        assert!(anthro_tools[0].get("input_schema").is_some());
    }
}
