use async_trait::async_trait;
use crate::config::Config;
use crate::provider::*;
use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::mpsc;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
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
            model: config.model.clone().unwrap_or_else(|| "claude-sonnet-4-20250514".into()),
        })
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str { "anthropic" }

    async fn stream(
        &self,
        _messages: &[LlmMessage],
        _tool_defs: &[Value],
    ) -> StreamRx {
        let (tx, rx) = mpsc::channel(256);
        tokio::spawn(async move {
            tx.send(StreamEvent { kind: StreamEventKind::Error("Anthropic streaming not yet implemented".into()) }).await.ok();
        });
        rx
    }

    async fn chat(&self, messages: &[LlmMessage], tool_defs: &[Value]) -> anyhow::Result<LlmResponse> {
        let url = "https://api.anthropic.com/v1/messages";
        let mut body = json!({
            "model": self.model,
            "max_tokens": 8192,
            "messages": [],
        });

        let mut msgs = Vec::new();
        for msg in messages {
            match msg {
                LlmMessage::System(c) => {
                    body["system"] = json!(c);
                }
                LlmMessage::User(c) => msgs.push(json!({"role": "user", "content": c})),
                LlmMessage::Assistant(c) => msgs.push(json!({"role": "assistant", "content": c})),
                _ => {}
            }
        }
        body["messages"] = json!(msgs);

        if !tool_defs.is_empty() {
            body["tools"] = json!(tool_defs);
        }

        let res = self.client.post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await?;

        let val: Value = res.json().await?;
        let content = val["content"][0]["text"].as_str().map(|s| s.to_string());

        Ok(LlmResponse { content, tool_calls: Vec::new(), usage: None })
    }
}
