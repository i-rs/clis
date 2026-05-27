use async_trait::async_trait;
use crate::config::Config;
use crate::provider::*;
use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::mpsc;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let api_key = config.api_key.clone()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok())
            .ok_or_else(|| anyhow::anyhow!("ANTHROPIC_API_KEY not set"))?;
        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: config.base_url.clone()
                .unwrap_or_else(|| "https://api.anthropic.com".to_string()),
            model: config.model.clone().unwrap_or_else(|| "claude-sonnet-4-20250514".into()),
        })
    }

    fn build_messages(msgs: &[LlmMessage]) -> (Option<String>, Vec<Value>) {
        let mut system = None;
        let mut anthro_msgs: Vec<Value> = Vec::new();

        for msg in msgs {
            match msg {
                LlmMessage::System(c) => {
                    system = Some(c.clone());
                }
                LlmMessage::User(c) => {
                    anthro_msgs.push(json!({"role": "user", "content": c}));
                }
                LlmMessage::Assistant(c) => {
                    anthro_msgs.push(json!({"role": "assistant", "content": c}));
                }
                LlmMessage::AssistantWithReasoning { content, reasoning, tool_calls } => {
                    let mut blocks = Vec::new();
                    if !reasoning.is_empty() {
                        blocks.push(json!({"type": "thinking", "thinking": reasoning}));
                    }
                    if !content.is_empty() {
                        blocks.push(json!({"type": "text", "text": content}));
                    }
                    for tc in tool_calls {
                        blocks.push(json!({
                            "type": "tool_use",
                            "id": tc.id,
                            "name": tc.name,
                            "input": tc.args,
                        }));
                    }
                    if blocks.is_empty() {
                        blocks.push(json!({"type": "text", "text": ""}));
                    }
                    anthro_msgs.push(json!({"role": "assistant", "content": blocks}));
                }
                LlmMessage::ToolCall { id, name, args } => {
                    anthro_msgs.push(json!({
                        "role": "assistant",
                        "content": [{
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": args
                        }]
                    }));
                }
                LlmMessage::Tool { name: _, content, call_id } => {
                    anthro_msgs.push(json!({
                        "role": "user",
                        "content": [{
                            "type": "tool_result",
                            "tool_use_id": call_id,
                            "content": content
                        }]
                    }));
                }
            }
        }
        (system, anthro_msgs)
    }

    fn convert_tool_schemas(tool_defs: &[Value]) -> Vec<Value> {
        tool_defs.iter().filter_map(|ts| {
            let func = ts.get("function")?;
            Some(json!({
                "name": func["name"].as_str()?,
                "description": func["description"].as_str().unwrap_or(""),
                "input_schema": func.get("parameters").cloned().unwrap_or(json!({"type": "object", "properties": {}})),
            }))
        }).collect()
    }

    fn parse_sse_event(event_type: &str, data: &str) -> Option<AnthropicEvent> {
        let parsed: Value = serde_json::from_str(data).ok()?;
        match event_type {
            "message_start" => {
                let msg = parsed.get("message")?;
                let usage = msg.get("usage").map(|u| Usage {
                    input_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
                    output_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
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
                        Some(AnthropicEvent::ContentBlockStart {
                            index,
                            block_type: "tool_use".to_string(),
                            tool_use_id: block["id"].as_str().map(|s| s.to_string()),
                            tool_use_name: block["name"].as_str().map(|s| s.to_string()),
                        })
                    }
                    _ => None,
                }
            }
            "content_block_delta" => {
                let index = parsed["index"].as_u64()? as usize;
                let delta = parsed.get("delta")?;
                match delta["type"].as_str() {
                    Some("text_delta") => Some(AnthropicEvent::ContentBlockDelta {
                        index,
                        text: delta["text"].as_str().map(|s| s.to_string()),
                        partial_json: None,
                    }),
                    Some("input_json_delta") => Some(AnthropicEvent::ContentBlockDelta {
                        index,
                        text: None,
                        partial_json: delta["partial_json"].as_str().map(|s| s.to_string()),
                    }),
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
                let usage = parsed.get("usage").map(|u| Usage {
                    input_tokens: 0,
                    output_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
                });
                Some(AnthropicEvent::MessageDelta { stop_reason, usage })
            }
            "message_stop" => Some(AnthropicEvent::MessageStop),
            "ping" => Some(AnthropicEvent::Ping),
            _ => None,
        }
    }
}

#[derive(Debug)]
enum AnthropicEvent {
    MessageStart { usage: Option<Usage> },
    ContentBlockStart { index: usize, block_type: String, tool_use_id: Option<String>, tool_use_name: Option<String> },
    ContentBlockDelta { index: usize, text: Option<String>, partial_json: Option<String> },
    ContentBlockStop { index: usize },
    MessageDelta { stop_reason: String, usage: Option<Usage> },
    MessageStop,
    Ping,
}

#[derive(Default, Clone)]
struct ContentBlock {
    block_type: String,
    text: String,
    tool_use_id: String,
    tool_use_name: String,
    partial_json: String,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str { "anthropic" }

    async fn stream(
        &self,
        messages: &[LlmMessage],
        tool_defs: &[Value],
    ) -> StreamRx {
        let (tx, rx) = mpsc::channel(256);
        let client = self.client.clone();
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let (system_prompt, anthro_msgs) = Self::build_messages(messages);
        let anthro_tools = Self::convert_tool_schemas(tool_defs);

        let mut body = json!({
            "model": model,
            "max_tokens": 8192,
            "stream": true,
            "messages": anthro_msgs,
        });

        if let Some(sys) = &system_prompt {
            body["system"] = json!(sys);
        }
        if !anthro_tools.is_empty() {
            body["tools"] = json!(anthro_tools);
        }

        tokio::spawn(async move {
            let res = match client.post(&url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    tx.send(StreamEvent { kind: StreamEventKind::Error(e.to_string()) }).await.ok();
                    return;
                }
            };

            if !res.status().is_success() {
                let status = res.status();
                let body_text = res.text().await.unwrap_or_default();
                tx.send(StreamEvent { kind: StreamEventKind::Error(format!("Anthropic API error ({}): {}", status, body_text)) }).await.ok();
                return;
            }

            let mut stream = res.bytes_stream();
            let mut buf = String::new();
            let mut current_event_type = String::new();
            let mut content_blocks: Vec<ContentBlock> = Vec::new();
            let mut total_usage: Option<Usage> = None;
            let mut stop_reason = String::new();

            use futures::StreamExt;
            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        tx.send(StreamEvent { kind: StreamEventKind::Error(e.to_string()) }).await.ok();
                        return;
                    }
                };
                buf.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(pos) = buf.find('\n') {
                    let line = buf[..pos].trim().to_string();
                    buf = buf[pos + 1..].to_string();
                    if line.is_empty() { continue; }

                    if let Some(evt) = line.strip_prefix("event: ") {
                        current_event_type = evt.to_string();
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        let event_type = std::mem::take(&mut current_event_type);
                        if event_type.is_empty() { continue; }

                        if let Some(event) = Self::parse_sse_event(&event_type, data) {
                            match event {
                                AnthropicEvent::MessageStart { usage } => {
                                    total_usage = usage;
                                }
                                AnthropicEvent::ContentBlockStart { index, block_type, tool_use_id, tool_use_name } => {
                                    if index >= content_blocks.len() {
                                        content_blocks.resize(index + 1, ContentBlock::default());
                                    }
                                    content_blocks[index].block_type = block_type;
                                    if let Some(id) = tool_use_id {
                                        content_blocks[index].tool_use_id = id;
                                    }
                                    if let Some(name) = tool_use_name {
                                        content_blocks[index].tool_use_name = name;
                                    }
                                }
                                AnthropicEvent::ContentBlockDelta { index, text, partial_json } => {
                                    if index >= content_blocks.len() {
                                        content_blocks.resize(index + 1, ContentBlock::default());
                                    }
                                    if let Some(t) = text {
                                        content_blocks[index].text.push_str(&t);
                                        tx.send(StreamEvent { kind: StreamEventKind::Token(t) }).await.ok();
                                    }
                                    if let Some(pj) = partial_json {
                                        content_blocks[index].partial_json.push_str(&pj);
                                    }
                                }
                                AnthropicEvent::ContentBlockStop { .. } => {}
                                AnthropicEvent::MessageDelta { stop_reason: sr, usage } => {
                                    stop_reason = sr;
                                    if let Some(u) = usage {
                                        if let Some(ref mut total) = total_usage {
                                            total.output_tokens = u.output_tokens;
                                        } else {
                                            total_usage = Some(u);
                                        }
                                    }
                                }
                                AnthropicEvent::MessageStop => {}
                                AnthropicEvent::Ping => {}
                            }
                        }
                    }
                }
            }

            if stop_reason == "tool_use" {
                for block in &content_blocks {
                    if block.block_type == "tool_use" {
                        let args: Value = serde_json::from_str(&block.partial_json)
                            .unwrap_or(serde_json::json!({}));
                        tx.send(StreamEvent { kind: StreamEventKind::ToolCall {
                            id: block.tool_use_id.clone(),
                            name: block.tool_use_name.clone(),
                            args,
                        }}).await.ok();
                    }
                }
            }

            tx.send(StreamEvent { kind: StreamEventKind::Done { content: None, usage: total_usage } }).await.ok();
        });

        rx
    }

    async fn chat(&self, messages: &[LlmMessage], tool_defs: &[Value]) -> anyhow::Result<LlmResponse> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let (system_prompt, anthro_msgs) = Self::build_messages(messages);
        let anthro_tools = Self::convert_tool_schemas(tool_defs);

        let mut body = json!({
            "model": self.model,
            "max_tokens": 8192,
            "messages": anthro_msgs,
        });

        if let Some(sys) = &system_prompt {
            body["system"] = json!(sys);
        }
        if !anthro_tools.is_empty() {
            body["tools"] = json!(anthro_tools);
        }

        let res = self.client.post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let body_text = res.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error ({}): {}", status, body_text);
        }

        let val: Value = res.json().await?;
        let _stop_reason = val["stop_reason"].as_str().unwrap_or("");

        let mut text_parts = Vec::new();
        let mut tool_calls = Vec::new();

        if let Some(content_blocks) = val["content"].as_array() {
            for block in content_blocks {
                match block["type"].as_str() {
                    Some("text") => {
                        if let Some(t) = block["text"].as_str() {
                            text_parts.push(t.to_string());
                        }
                    }
                    Some("tool_use") => {
                        tool_calls.push(ToolCall {
                            id: block["id"].as_str().unwrap_or("").to_string(),
                            name: block["name"].as_str().unwrap_or("").to_string(),
                            args: block["input"].clone(),
                        });
                    }
                    _ => {}
                }
            }
        }

        let content = if text_parts.is_empty() { None } else { Some(text_parts.join("")) };

        let usage = val.get("usage").map(|u| Usage {
            input_tokens: u["input_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: u["output_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(LlmResponse { content, reasoning: String::new(), tool_calls, usage })
    }
}
