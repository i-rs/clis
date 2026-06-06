use i_rs_claw_core::llm::{LlmEvent, StreamResult};
use i_rs_claw_core::providers::sse::openai_stream_chat_impl;
use i_rs_claw_core::providers::{LlmProvider, ProviderKind};
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
    pub fn new(client: reqwest::Client, base_url: String, model: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            client,
            base_url,
            model,
        }
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
        trace_id: &str,
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
            trace_id,
        )
        .await
    }
}
