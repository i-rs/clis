use async_trait::async_trait;
use crate::config::Config;
use crate::provider::*;
use reqwest::Client;
use serde_json::{json, Value};

pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?;
        let base_url = config.base_url.as_deref()
            .unwrap_or("http://localhost:11434/v1")
            .trim_end_matches('/')
            .to_string();
        let model = config.effective_model().to_string();
        Ok(Self { client, base_url, model })
    }
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str { "ollama" }

    async fn stream(&self, messages: &[LlmMessage], tool_defs: &[Value]) -> StreamRx {
        let (tx, rx) = mpsc::channel(256);
        let client = self.client.clone();
        let url = format!("{}/chat/completions", self.base_url);
        let model = self.model.clone();
        let msgs = super::openai::OpenAiProvider::build_messages(messages);
        let tool_defs = tool_defs.to_vec();

        let body = json!({
            "model": model,
            "messages": msgs,
            "stream": true,
        });

        tokio::spawn(async move {
            let mut body = body;
            if !tool_defs.is_empty() {
                body["tools"] = json!(tool_defs);
            }

            let res = match client.post(&url).json(&body).send().await {
                Ok(r) => r,
                Err(e) => {
                    tx.send(StreamEvent { kind: StreamEventKind::Error(e.to_string()) }).await.ok();
                    return;
                }
            };

            let status = res.status();
            if !status.is_success() {
                let body_text = res.text().await.unwrap_or_default();
                tx.send(StreamEvent { kind: StreamEventKind::Error(format!("Ollama error ({}): {}", status, body_text)) }).await.ok();
                return;
            }

            let mut buf = String::new();
            let mut stream = res.bytes_stream();
            let mut final_usage: Option<Usage> = None;
            let mut tool_call_accum: std::collections::HashMap<u32, (String, String, String)> = std::collections::HashMap::new();
            use futures::StreamExt;
            while let Some(chunk) = stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        tx.send(StreamEvent { kind: StreamEventKind::Error(e.to_string()) }).await.ok();
                        return;
                    }
                };
                let chunk_str = String::from_utf8_lossy(&chunk);
                buf.push_str(&chunk_str);
                while let Some(pos) = buf.find('\n') {
                    let line = buf[..pos].trim_end_matches('\r').to_string();
                    buf = buf[pos + 1..].to_string();
                    if line.is_empty() { continue; }
                    if line == "data: [DONE]" { continue; }
                    if let Some(data) = line.strip_prefix("data: ")
                        && let Ok(val) = serde_json::from_str::<Value>(data) {
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
                            if val.get("usage").and_then(|u| u.as_object()).is_some() {
                                let input = val["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32;
                                let output = val["usage"]["completion_tokens"].as_u64().unwrap_or(0) as u32;
                                final_usage = Some(Usage { input_tokens: input, output_tokens: output });
                            }
                        }
                }
            }
            for idx in 0..tool_call_accum.len() as u32 {
                if let Some((id, name, args_str)) = tool_call_accum.remove(&idx)
                    && !id.is_empty() && !name.is_empty() {
                        let args_val = serde_json::from_str(&args_str).unwrap_or(json!({}));
                        tx.send(StreamEvent { kind: StreamEventKind::ToolCall { id, name, args: args_val } }).await.ok();
                    }
            }
            tx.send(StreamEvent { kind: StreamEventKind::Done { usage: final_usage } }).await.ok();
        });

        rx
    }

    async fn chat(&self, messages: &[LlmMessage], tool_defs: &[Value]) -> anyhow::Result<LlmResponse> {
        let url = format!("{}/chat/completions", self.base_url);
        let msgs = super::openai::OpenAiProvider::build_messages(messages);

        let mut body = json!({
            "model": self.model,
            "messages": msgs,
        });
        if !tool_defs.is_empty() {
            body["tools"] = json!(tool_defs);
        }

        let res = self.client.post(&url).json(&body).send().await?;
        if !res.status().is_success() {
            let status = res.status();
            let body_text = res.text().await.unwrap_or_default();
            anyhow::bail!("Ollama error ({}): {}", status, body_text);
        }

        let val: Value = res.json().await?;
        let choice = val["choices"][0]["message"].clone();
        let content = choice["content"].as_str().map(String::from);
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

        Ok(LlmResponse { content, reasoning: String::new(), tool_calls, usage })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let config = Config::default();
        let provider = OllamaProvider::new(&config).unwrap();
        assert_eq!(provider.name(), "ollama");
    }

    #[test]
    fn test_provider_base_url_default() {
        let config = Config::default();
        let provider = OllamaProvider::new(&config).unwrap();
        assert_eq!(provider.base_url, "http://localhost:11434/v1");
    }

    #[test]
    fn test_provider_base_url_custom() {
        let config = Config {
            base_url: Some("http://10.0.0.1:11434/v1".into()),
            ..Config::default()
        };
        let provider = OllamaProvider::new(&config).unwrap();
        assert_eq!(provider.base_url, "http://10.0.0.1:11434/v1");
    }

    #[test]
    fn test_provider_model_uses_effective() {
        let config = Config {
            model: Some("llama3".into()),
            ..Config::default()
        };
        let provider = OllamaProvider::new(&config).unwrap();
        assert_eq!(provider.model, "llama3");
    }
}
