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

        let body = json!({
            "model": model,
            "messages": msgs,
            "stream": true,
            "tools": if tool_defs.is_empty() { json!(null) } else { json!(tool_defs) },
        });

        let log_body = serde_json::to_string(&body).unwrap_or_default();
        let log_url = url.clone();

        tokio::spawn(async move {
            let start = std::time::Instant::now();

            let res = match client.post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    crate::debug::push_log(crate::debug::HttpLogEntry {
                        url: log_url.clone(),
                        request_body: log_body.clone(),
                        response_status: 0,
                        response_body_preview: e.to_string(),
                        duration_ms: start.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    });
                    tx.send(StreamEvent { kind: StreamEventKind::Error(e.to_string()) }).await.ok();
                    return;
                }
            };

            let status = res.status();

            if !status.is_success() {
                let status_code = status.as_u16();
                let body_text = res.text().await.unwrap_or_default();
                crate::debug::push_log(crate::debug::HttpLogEntry {
                    url: log_url.clone(),
                    request_body: log_body.clone(),
                    response_status: status_code,
                    response_body_preview: body_text.clone(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                });
                tx.send(StreamEvent { kind: StreamEventKind::Error(format!("API error ({}): {}", status_code, body_text)) }).await.ok();
                return;
            }

            let mut total_bytes = 0usize;
            let mut buf = String::new();
            let mut stream = res.bytes_stream();
            let mut final_usage: Option<Usage> = None;
            let mut tool_call_accum: std::collections::HashMap<u32, (String, String, String)> = std::collections::HashMap::new();
            use futures::StreamExt;
            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => {
                        total_bytes += c.len();
                        c
                    }
                    Err(e) => {
                        tx.send(StreamEvent { kind: StreamEventKind::Error(e.to_string()) }).await.ok();
                        return;
                    }
                };
                buf.push_str(&String::from_utf8_lossy(&chunk));
                for line in buf.lines() {
                    if line.is_empty() { continue; }
                    if line == "data: [DONE]" { continue; }
                    if let Some(data) = line.strip_prefix("data: ")
                        && let Ok(val) = serde_json::from_str::<Value>(data) {
                            if val.get("usage").and_then(|u| u.as_object()).is_some() {
                                let input = val["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
                                let output = val["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;
                                final_usage = Some(Usage { input_tokens: input, output_tokens: output });
                            }
                            if let Some(choices) = val["choices"].as_array() {
                                for choice in choices {
                                    let delta = &choice["delta"];
                                    if let Some(content) = delta["content"].as_str()
                                        && !content.is_empty() {
                                            tx.send(StreamEvent { kind: StreamEventKind::Token(content.to_string()) }).await.ok();
                                        }
                                    if let Some(tcs) = delta["tool_calls"].as_array() {
                                        for tc in tcs {
                                            let idx = tc["index"].as_u64().unwrap_or(0) as u32;
                                            let entry = tool_call_accum.entry(idx).or_insert_with(|| (String::new(), String::new(), String::new()));
                                            if let Some(id) = tc["id"].as_str() {
                                                entry.0 = id.to_string();
                                            }
                                            if let Some(name) = tc["function"]["name"].as_str() {
                                                entry.1 = name.to_string();
                                            }
                                            if let Some(args) = tc["function"]["arguments"].as_str() {
                                                entry.2.push_str(args);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                }
                buf.clear();
            }
            for idx in 0..tool_call_accum.len() as u32 {
                if let Some((id, name, args_str)) = tool_call_accum.remove(&idx)
                    && !id.is_empty() && !name.is_empty() {
                        let args_val = serde_json::from_str(&args_str).unwrap_or(serde_json::json!({}));
                        tx.send(StreamEvent { kind: StreamEventKind::ToolCall { id, name, args: args_val } }).await.ok();
                    }
            }

            crate::debug::push_log(crate::debug::HttpLogEntry {
                url: log_url,
                request_body: log_body,
                response_status: status.as_u16(),
                response_body_preview: format!("streamed {} bytes, {} rounds", total_bytes, 1),
                duration_ms: start.elapsed().as_millis() as u64,
                timestamp: chrono::Utc::now().to_rfc3339(),
            });

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
                body["tool_choice"] = json!("auto");
            }

        let log_body = serde_json::to_string(&body).unwrap_or_default();
        let log_url = url.clone();
        let start = std::time::Instant::now();

        let res = client.post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let body_text = res.text().await.unwrap_or_default();
            crate::debug::push_log(crate::debug::HttpLogEntry {
                url: log_url,
                request_body: log_body,
                response_status: status.as_u16(),
                response_body_preview: body_text.clone(),
                duration_ms: start.elapsed().as_millis() as u64,
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
            anyhow::bail!("API error ({}): {}", status, body_text);
        }

        let val: Value = res.json().await?;
        let body_text = serde_json::to_string(&val).unwrap_or_default();

        crate::debug::push_log(crate::debug::HttpLogEntry {
            url: log_url,
            request_body: log_body,
            response_status: 200,
            response_body_preview: body_text.chars().take(500).collect(),
            duration_ms: start.elapsed().as_millis() as u64,
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

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
