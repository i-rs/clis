use crate::llm::{LlmEvent, StreamResult};
use crate::providers::sse::openai_stream_chat_impl;
use crate::providers::{LlmProvider, ProviderKind};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

// =============================================
// Ollama Provider
// =============================================

pub struct OllamaProvider {
    client: reqwest::Client,
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(client: reqwest::Client, model: String) -> Self {
        Self {
            client,
            base_url: "http://localhost:11434/v1".to_string(),
            model,
        }
    }

    #[allow(dead_code)]
    pub fn with_url(client: reqwest::Client, base_url: String, model: String) -> Self {
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
