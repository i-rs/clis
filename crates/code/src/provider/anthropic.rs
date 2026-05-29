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
                                    if let Some(ref t) = text {
                                        content_blocks[index].text.push_str(t);
                                        tx.send(StreamEvent { kind: StreamEventKind::Token(t.clone()) }).await.ok();
                                    }
                                    if let Some(pj) = partial_json {
                                        content_blocks[index].partial_json.push_str(&pj);
                                    }
                                    if content_blocks[index].block_type == "thinking" {
                                        if let Some(ref t) = text {
                                            tx.send(StreamEvent { kind: StreamEventKind::Reasoning(t.clone()) }).await.ok();
                                        }
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

    async fn chat(
        &self,
        messages: &[LlmMessage],
        tool_defs: &[Value],
    ) -> anyhow::Result<LlmResponse> {
        let mut rx = self.stream(messages, tool_defs).await;
        let mut content = String::new();
        let mut reasoning = String::new();
        let mut tool_calls = Vec::new();
        let mut usage = None;
        while let Some(event) = rx.recv().await {
            match event.kind {
                StreamEventKind::Token(t) => content.push_str(&t),
                StreamEventKind::Reasoning(r) => reasoning.push_str(&r),
                StreamEventKind::ToolCall { id, name, args } => tool_calls.push(ToolCall { id, name, args }),
                StreamEventKind::Done { usage: u, .. } => usage = u,
                StreamEventKind::Error(e) => anyhow::bail!("chat error: {}", e),
            }
        }
        Ok(LlmResponse {
            content: if content.is_empty() { None } else { Some(content) },
            reasoning,
            tool_calls,
            usage,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_messages_simple_user() {
        let msgs = vec![LlmMessage::User("hello".into())];
        let (system, anthro_msgs) = AnthropicProvider::build_messages(&msgs);
        assert!(system.is_none());
        assert_eq!(anthro_msgs.len(), 1);
        assert_eq!(anthro_msgs[0]["role"], "user");
        assert_eq!(anthro_msgs[0]["content"], "hello");
    }

    #[test]
    fn test_build_messages_with_system() {
        let msgs = vec![
            LlmMessage::System("You are Claude".into()),
            LlmMessage::User("hello".into()),
        ];
        let (system, anthro_msgs) = AnthropicProvider::build_messages(&msgs);
        assert_eq!(system.unwrap(), "You are Claude");
        assert_eq!(anthro_msgs.len(), 1);
    }

    #[test]
    fn test_build_messages_with_tool_use() {
        let msgs = vec![
            LlmMessage::User("run cargo check".into()),
            LlmMessage::ToolCall {
                id: "call-1".into(),
                name: "bash".into(),
                args: serde_json::json!({"command": "cargo check"}),
            },
            LlmMessage::Tool {
                name: "bash".into(),
                content: "Compiling...\nFinished".into(),
                call_id: "call-1".into(),
            },
        ];
        let (_system, anthro_msgs) = AnthropicProvider::build_messages(&msgs);
        assert_eq!(anthro_msgs.len(), 3);
        assert_eq!(anthro_msgs[0]["role"], "user");
        assert_eq!(anthro_msgs[1]["role"], "assistant");
        assert_eq!(anthro_msgs[1]["content"][0]["type"], "tool_use");
        assert_eq!(anthro_msgs[1]["content"][0]["id"], "call-1");
        assert_eq!(anthro_msgs[2]["role"], "user");
        assert_eq!(anthro_msgs[2]["content"][0]["type"], "tool_result");
        assert_eq!(anthro_msgs[2]["content"][0]["tool_use_id"], "call-1");
    }

    #[test]
    fn test_build_messages_assistant_with_reasoning() {
        let msgs = vec![LlmMessage::AssistantWithReasoning {
            content: "Let me check".into(),
            reasoning: "I need to find the bug".into(),
            tool_calls: vec![],
        }];
        let (_system, anthro_msgs) = AnthropicProvider::build_messages(&msgs);
        assert_eq!(anthro_msgs.len(), 1);
        let content = anthro_msgs[0]["content"].as_array().unwrap();
        assert_eq!(content[0]["type"], "thinking");
        assert_eq!(content[1]["type"], "text");
        assert_eq!(content[1]["text"], "Let me check");
    }

    #[test]
    fn test_convert_tool_schemas_empty() {
        let result = AnthropicProvider::convert_tool_schemas(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_convert_tool_schemas_single() {
        let defs = vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": "read",
                "description": "Read a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "file_path": {"type": "string"}
                    }
                }
            }
        })];
        let result = AnthropicProvider::convert_tool_schemas(&defs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0]["name"], "read");
        assert_eq!(result[0]["description"], "Read a file");
        assert!(result[0]["input_schema"].is_object());
    }

    #[test]
    fn test_parse_sse_message_start() {
        let data = r#"{"message": {"usage": {"input_tokens": 10, "output_tokens": 5}}}"#;
        let event = AnthropicProvider::parse_sse_event("message_start", data);
        assert!(event.is_some());
        match event.unwrap() {
            AnthropicEvent::MessageStart { usage } => {
                let u = usage.unwrap();
                assert_eq!(u.input_tokens, 10);
                assert_eq!(u.output_tokens, 5);
            }
            _ => panic!("expected MessageStart"),
        }
    }

    #[test]
    fn test_parse_sse_content_block_start_text() {
        let data = r#"{"index": 0, "content_block": {"type": "text"}}"#;
        let event = AnthropicProvider::parse_sse_event("content_block_start", data);
        assert!(event.is_some());
        match event.unwrap() {
            AnthropicEvent::ContentBlockStart { index, block_type, tool_use_id, tool_use_name } => {
                assert_eq!(index, 0);
                assert_eq!(block_type, "text");
                assert!(tool_use_id.is_none());
                assert!(tool_use_name.is_none());
            }
            _ => panic!("expected ContentBlockStart"),
        }
    }

    #[test]
    fn test_parse_sse_content_block_start_tool_use() {
        let data = r#"{"index": 1, "content_block": {"type": "tool_use", "id": "tu-1", "name": "bash"}}"#;
        let event = AnthropicProvider::parse_sse_event("content_block_start", data);
        assert!(event.is_some());
        match event.unwrap() {
            AnthropicEvent::ContentBlockStart { index, block_type, tool_use_id, tool_use_name } => {
                assert_eq!(index, 1);
                assert_eq!(block_type, "tool_use");
                assert_eq!(tool_use_id.unwrap(), "tu-1");
                assert_eq!(tool_use_name.unwrap(), "bash");
            }
            _ => panic!("expected ContentBlockStart"),
        }
    }

    #[test]
    fn test_parse_sse_text_delta() {
        let data = r#"{"index": 0, "delta": {"type": "text_delta", "text": "Hello"}}"#;
        let event = AnthropicProvider::parse_sse_event("content_block_delta", data);
        assert!(event.is_some());
        match event.unwrap() {
            AnthropicEvent::ContentBlockDelta { index, text, partial_json } => {
                assert_eq!(index, 0);
                assert_eq!(text.unwrap(), "Hello");
                assert!(partial_json.is_none());
            }
            _ => panic!("expected ContentBlockDelta"),
        }
    }

    #[test]
    fn test_parse_sse_input_json_delta() {
        let data = r#"{"index": 1, "delta": {"type": "input_json_delta", "partial_json": "{\"command\":\"check\"}"}}"#;
        let event = AnthropicProvider::parse_sse_event("content_block_delta", data);
        assert!(event.is_some());
        match event.unwrap() {
            AnthropicEvent::ContentBlockDelta { index, text, partial_json } => {
                assert_eq!(index, 1);
                assert!(text.is_none());
                assert!(partial_json.unwrap().contains("command"));
            }
            _ => panic!("expected ContentBlockDelta"),
        }
    }

    #[test]
    fn test_parse_sse_content_block_stop() {
        let data = r#"{"index": 0}"#;
        let event = AnthropicProvider::parse_sse_event("content_block_stop", data);
        assert!(event.is_some());
        match event.unwrap() {
            AnthropicEvent::ContentBlockStop { index } => {
                assert_eq!(index, 0);
            }
            _ => panic!("expected ContentBlockStop"),
        }
    }

    #[test]
    fn test_parse_sse_message_delta() {
        let data = r#"{"delta": {"stop_reason": "end_turn"}, "usage": {"output_tokens": 50}}"#;
        let event = AnthropicProvider::parse_sse_event("message_delta", data);
        assert!(event.is_some());
        match event.unwrap() {
            AnthropicEvent::MessageDelta { stop_reason, usage } => {
                assert_eq!(stop_reason, "end_turn");
                assert_eq!(usage.unwrap().output_tokens, 50);
            }
            _ => panic!("expected MessageDelta"),
        }
    }

    #[test]
    fn test_parse_sse_message_stop() {
        let data = r#"{}"#;
        let event = AnthropicProvider::parse_sse_event("message_stop", data);
        assert!(matches!(event.unwrap(), AnthropicEvent::MessageStop));
    }

    #[test]
    fn test_parse_sse_ping() {
        let data = r#"{}"#;
        let event = AnthropicProvider::parse_sse_event("ping", data);
        assert!(matches!(event.unwrap(), AnthropicEvent::Ping));
    }

    #[test]
    fn test_parse_sse_unknown_event_returns_none() {
        let data = r#"{}"#;
        let event = AnthropicProvider::parse_sse_event("unknown_event", data);
        assert!(event.is_none());
    }

    #[test]
    fn test_parse_sse_invalid_json_returns_none() {
        let event = AnthropicProvider::parse_sse_event("message_start", "not json");
        assert!(event.is_none());
    }

    #[test]
    fn test_build_messages_unknown_variant_skipped() {
        let msgs = vec![LlmMessage::System("test".into())];
        let (system, anthro_msgs) = AnthropicProvider::build_messages(&msgs);
        assert_eq!(system.unwrap(), "test");
        assert!(anthro_msgs.is_empty());
    }
}
