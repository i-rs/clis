// ── Sub-modules ──
pub(crate) mod sse;
mod openai;
mod ollama;
mod anthropic;

pub use openai::OpenaiProvider;
pub use ollama::OllamaProvider;
pub use anthropic::AnthropicProvider;
pub use super::llm::LlmEvent;
pub(crate) use super::llm::StreamResult;

use serde_json::Value;
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
            other => {
                if other != "openai" && !other.is_empty() {
                    tracing::warn!("未知 provider '{}', 回退到 OpenAI 兼容模式", other);
                }
                ProviderKind::OpenAI
            }
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

// =============================================
// Factory
// =============================================

pub fn shared_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

/// Create the appropriate provider based on configuration.
pub fn create_provider(client: &reqwest::Client, config: &crate::config::Config) -> Box<dyn LlmProvider> {
    create_provider_for(client, &config.provider, &config.api_key, &config.base_url, &config.model)
}

/// Create a provider from individual fields (provider type, api key, base url, model).
pub fn create_provider_for(
    client: &reqwest::Client,
    provider_type: &str,
    api_key: &str,
    base_url: &str,
    model: &str,
) -> Box<dyn LlmProvider> {
    match ProviderKind::from_str(provider_type) {
        ProviderKind::OpenAI => Box::new(OpenaiProvider::new(
            client.clone(),
            api_key.to_string(),
            base_url.to_string(),
            model.to_string(),
        )),
        ProviderKind::Anthropic => Box::new(AnthropicProvider::new(
            client.clone(),
            api_key.to_string(),
            base_url.to_string(),
            model.to_string(),
        )),
        ProviderKind::Ollama => Box::new(OllamaProvider::new(client.clone(), base_url.to_string(), model.to_string())),
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_kind_from_str_openai() {
        assert_eq!(ProviderKind::from_str("openai"), ProviderKind::OpenAI);
        assert_eq!(ProviderKind::from_str("OpenAI"), ProviderKind::OpenAI);
        assert_eq!(ProviderKind::from_str("OPENAI"), ProviderKind::OpenAI);
    }

    #[test]
    fn test_provider_kind_from_str_anthropic() {
        assert_eq!(ProviderKind::from_str("anthropic"), ProviderKind::Anthropic);
    }

    #[test]
    fn test_provider_kind_from_str_ollama() {
        assert_eq!(ProviderKind::from_str("ollama"), ProviderKind::Ollama);
    }

    #[test]
    fn test_provider_kind_from_str_unknown_defaults_to_openai() {
        assert_eq!(ProviderKind::from_str("unknown"), ProviderKind::OpenAI);
        assert_eq!(ProviderKind::from_str("zhipu"), ProviderKind::OpenAI);
        assert_eq!(ProviderKind::from_str(""), ProviderKind::OpenAI);
    }

    #[test]
    fn test_provider_kind_as_str() {
        assert_eq!(ProviderKind::OpenAI.as_str(), "openai");
        assert_eq!(ProviderKind::Anthropic.as_str(), "anthropic");
        assert_eq!(ProviderKind::Ollama.as_str(), "ollama");
    }

    #[test]
    fn test_provider_kind_all() {
        let all = ProviderKind::all();
        assert_eq!(all.len(), 3);
        assert!(all.contains(&ProviderKind::OpenAI));
        assert!(all.contains(&ProviderKind::Anthropic));
        assert!(all.contains(&ProviderKind::Ollama));
    }
}
