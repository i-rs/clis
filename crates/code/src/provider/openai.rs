use async_trait::async_trait;
use crate::config::Config;
use crate::provider::*;
use reqwest::Client;
use serde_json::{json, Value};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let api_key = config.api_key.clone()
            .or_else(|| std::env::var("OPENAI_API_KEY").ok())
            .ok_or_else(|| anyhow::anyhow!("OPENAI_API_KEY not set"))?;
        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: config.effective_base_url().trim_end_matches('/').to_string(),
            model: config.effective_model().to_string(),
        })
    }

    fn build_messages(msgs: &[LlmMessage]) -> Vec<Value> {
        let mut out = Vec::new();
        for msg in msgs {
            match msg {
                LlmMessage::System(c) => out.push(json!({"role": "system", "content": c})),
                LlmMessage::User(c) => out.push(json!({"role": "user", "content": c})),
                LlmMessage::Assistant(c) => out.push(json!({"role": "assistant", "content": c})),
                LlmMessage::Tool { name, content, call_id } => {
                    out.push(json!({"role": "tool", "tool_call_id": call_id, "name": name, "content": content}));
                }
                LlmMessage::ToolCall { id, name, args } => {
                    out.push(json!({"role": "assistant", "content": null, "tool_calls": [{"id": id, "type": "function", "function": {"name": name, "arguments": args.to_string()}}]}));
                }
            }
        }
        out
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    fn name(&self) -> &str { "openai" }

    async fn stream(
        &self,
        messages: &[LlmMessage],
        tool_defs: &[Value],
    ) -> StreamRx {
        let (tx, rx) = mpsc::channel(256);
        let client = self.client.clone();
        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let msgs = Self::build_messages(messages);
        let tool_defs = tool_defs.to_vec();

        tokio::spawn(async move {
            let mut body = json!({
                "model": model,
                "messages": msgs,
                "stream": true,
            });
            if !tool_defs.is_empty() {
                body["tools"] = json!(tool_defs);
            }

            let res = match client.post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
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
                let error_msg = format!("API error ({}): {}", status, body_text);
                tx.send(StreamEvent { kind: StreamEventKind::Error(error_msg) }).await.ok();
                return;
            }

            let mut buf = String::new();
            let mut stream = res.bytes_stream();
            let mut final_usage: Option<Usage> = None;
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
                for line in buf.lines() {
                    if line.is_empty() { continue; }
                    if line == "data: [DONE]" { continue; }
                    if let Some(data) = line.strip_prefix("data: ") {
                        if let Ok(val) = serde_json::from_str::<Value>(data) {
                            // Extract usage from the final chunk (empty choices + usage field)
                            if val.get("usage").and_then(|u| u.as_object()).is_some() {
                                let input = val["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
                                let output = val["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;
                                final_usage = Some(Usage { input_tokens: input, output_tokens: output });
                            }
                            if let Some(choices) = val["choices"].as_array() {
                                for choice in choices {
                                    let delta = &choice["delta"];
                                    if let Some(content) = delta["content"].as_str() {
                                        if !content.is_empty() {
                                            tx.send(StreamEvent { kind: StreamEventKind::Token(content.to_string()) }).await.ok();
                                        }
                                    }
                                    if let Some(tcs) = delta["tool_calls"].as_array() {
                                        for tc in tcs {
                                            if let (Some(id), Some(name), Some(args)) = (
                                                tc["id"].as_str(),
                                                tc["function"]["name"].as_str(),
                                                tc["function"]["arguments"].as_str(),
                                            ) {
                                                if let Ok(args_val) = serde_json::from_str(args) {
                                                    tx.send(StreamEvent { kind: StreamEventKind::ToolCall { id: id.to_string(), name: name.to_string(), args: args_val } }).await.ok();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                buf.clear();
            }
            tx.send(StreamEvent { kind: StreamEventKind::Done { content: None, usage: final_usage } }).await.ok();
        });

        rx
    }

    async fn chat(&self, messages: &[LlmMessage], tool_defs: &[Value]) -> anyhow::Result<LlmResponse> {
        let client = self.client.clone();
        let url = format!("{}/chat/completions", self.base_url);
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let msgs = Self::build_messages(messages);

        let mut body = json!({
            "model": model,
            "messages": msgs,
        });
        if !tool_defs.is_empty() {
            body["tools"] = json!(tool_defs);
        }

        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let body_text = res.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body_text);
        }

        let val: Value = res.json().await?;
        let choice = val["choices"][0]["message"].clone();
        let content = choice["content"].as_str().map(|s| s.to_string());
        let tool_calls = if let Some(tcs) = choice["tool_calls"].as_array() {
            tcs.iter().map(|tc| ToolCall {
                id: tc["id"].as_str().unwrap_or("").to_string(),
                name: tc["function"]["name"].as_str().unwrap_or("").to_string(),
                args: serde_json::from_str(tc["function"]["arguments"].as_str().unwrap_or("{}")).unwrap_or_default(),
            }).collect()
        } else {
            Vec::new()
        };
        let usage = val.get("usage").map(|u| Usage {
            input_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            output_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(LlmResponse { content, tool_calls, usage })
    }
}
