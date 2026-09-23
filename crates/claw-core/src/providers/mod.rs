// ── Sub-modules ──
pub mod anthropic;
pub mod common;
pub mod image_gen;
pub mod ollama;
pub mod openai;
pub mod sse;

pub use super::llm::LlmEvent;
pub use super::llm::StreamResult;
pub use anthropic::AnthropicProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenaiProvider;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

// ── Provider Kind ──

/// Provider identifier used in configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    OpenAI,
    Anthropic,
    Ollama,
    Zhipu,
}

impl std::fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for ProviderKind {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "anthropic" => ProviderKind::Anthropic,
            "ollama" => ProviderKind::Ollama,
            "zhipu" => ProviderKind::Zhipu,
            other => {
                if other != "openai" && !other.is_empty() {
                    tracing::warn!("未知 provider '{}', 回退到 OpenAI 兼容模式", other);
                }
                ProviderKind::OpenAI
            }
        })
    }
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderKind::OpenAI => "openai",
            ProviderKind::Anthropic => "anthropic",
            ProviderKind::Ollama => "ollama",
            ProviderKind::Zhipu => "zhipu",
        }
    }

    pub fn all() -> Vec<ProviderKind> {
        vec![
            ProviderKind::OpenAI,
            ProviderKind::Anthropic,
            ProviderKind::Ollama,
            ProviderKind::Zhipu,
        ]
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
    /// `trace_id` links this API call to stats / checkpoints for auditing.
    async fn stream_chat(
        &self,
        messages: &[Value],
        tool_schemas: &[Value],
        tx: &UnboundedSender<LlmEvent>,
        trace_id: &str,
    ) -> anyhow::Result<StreamResult>;
}

// =============================================
// OpenCode Go gateway (opencode.ai)
// =============================================

/// OpenCode Go 网关 (opencode.ai) 要求每个会话携带稳定的 `x-opencode-session`
/// 请求头，否则返回 400 MissingSessionID。同时要求用自定义 User-Agent 标识客户端。
/// 见 https://opencode.ai/docs/go/#where-can-i-use-it
pub(crate) fn is_opencode_gateway(base_url: &str) -> bool {
    let rest = base_url.split_once("://").map(|(_, r)| r).unwrap_or(base_url);
    let host = rest.split('/').next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or(host); // 去掉 userinfo
    let host = host.split(':').next().unwrap_or(host); // 去掉端口
    host == "opencode.ai" || host.ends_with(".opencode.ai")
}

/// 生成 OpenCode 网关所需的请求头（会话 ID + 客户端标识）。
/// `session_id` 在 provider 实例生命周期内保持稳定（一次 chat_loop 的所有
/// ReAct 轮次复用同一个值），满足网关的路由/缓存亲和要求。
pub(crate) fn opencode_session_headers(session_id: &str) -> Vec<(String, String)> {
    vec![
        ("x-opencode-session".to_string(), session_id.to_string()),
        (
            "User-Agent".to_string(),
            format!("i-rs-claw/{}", env!("CARGO_PKG_VERSION")),
        ),
    ]
}

/// 每个 provider 实例生成一次的稳定会话 ID（OpenCode 网关用）。
pub(crate) fn new_opencode_session_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// =============================================
// Factory
// =============================================

pub fn shared_client() -> reqwest::Client {
    crate::utils::sync_block_on(async {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

/// Create the appropriate provider based on configuration.
pub fn create_provider(
    client: &reqwest::Client,
    config: &crate::config::Config,
) -> Box<dyn LlmProvider> {
    create_provider_for(
        client,
        config.provider,
        &config.api_key,
        &config.base_url,
        &config.model,
    )
}

/// Create a provider from individual fields (provider type, api key, base url, model).
pub fn create_provider_for(
    client: &reqwest::Client,
    provider_type: ProviderKind,
    api_key: &str,
    base_url: &str,
    model: &str,
) -> Box<dyn LlmProvider> {
    match provider_type {
        ProviderKind::OpenAI | ProviderKind::Zhipu => Box::new(OpenaiProvider::new(
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
        ProviderKind::Ollama => Box::new(OllamaProvider::new(
            client.clone(),
            base_url.to_string(),
            model.to_string(),
        )),
    }
}

/// Create a provider by name from the config's [providers] section.
#[allow(dead_code)]
pub fn create_provider_for_name(
    client: &reqwest::Client,
    name: &str,
    config: &crate::config::Config,
) -> Option<Box<dyn LlmProvider>> {
    let pc = config.providers.get(name)?;
    Some(create_provider_for(
        client,
        pc.provider,
        &pc.api_key,
        &pc.base_url,
        &pc.model,
    ))
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_kind_from_str_openai() {
        assert_eq!(
            "openai".parse::<ProviderKind>().unwrap(),
            ProviderKind::OpenAI
        );
        assert_eq!(
            "OpenAI".parse::<ProviderKind>().unwrap(),
            ProviderKind::OpenAI
        );
        assert_eq!(
            "OPENAI".parse::<ProviderKind>().unwrap(),
            ProviderKind::OpenAI
        );
    }

    #[test]
    fn test_provider_kind_from_str_anthropic() {
        assert_eq!(
            "anthropic".parse::<ProviderKind>().unwrap(),
            ProviderKind::Anthropic
        );
    }

    #[test]
    fn test_provider_kind_from_str_ollama() {
        assert_eq!(
            "ollama".parse::<ProviderKind>().unwrap(),
            ProviderKind::Ollama
        );
    }

    #[test]
    fn test_provider_kind_from_str_unknown_defaults_to_openai() {
        assert_eq!(
            "unknown".parse::<ProviderKind>().unwrap(),
            ProviderKind::OpenAI
        );
        assert_eq!("".parse::<ProviderKind>().unwrap(), ProviderKind::OpenAI);
    }

    #[test]
    fn test_provider_kind_as_str() {
        assert_eq!(ProviderKind::OpenAI.as_str(), "openai");
        assert_eq!(ProviderKind::Anthropic.as_str(), "anthropic");
        assert_eq!(ProviderKind::Ollama.as_str(), "ollama");
        assert_eq!(ProviderKind::Zhipu.as_str(), "zhipu");
    }

    #[test]
    fn test_provider_kind_all() {
        let all = ProviderKind::all();
        assert_eq!(all.len(), 4);
        assert!(all.contains(&ProviderKind::OpenAI));
        assert!(all.contains(&ProviderKind::Anthropic));
        assert!(all.contains(&ProviderKind::Ollama));
        assert!(all.contains(&ProviderKind::Zhipu));
    }

    #[test]
    fn test_provider_kind_from_str_zhipu() {
        assert_eq!(
            "zhipu".parse::<ProviderKind>().unwrap(),
            ProviderKind::Zhipu
        );
        assert_eq!(
            "Zhipu".parse::<ProviderKind>().unwrap(),
            ProviderKind::Zhipu
        );
    }

    #[test]
    fn test_is_opencode_gateway() {
        assert!(is_opencode_gateway("https://opencode.ai/zen/go/v1"));
        assert!(is_opencode_gateway("https://opencode.ai"));
        assert!(is_opencode_gateway("https://api.opencode.ai/v1"));
        assert!(is_opencode_gateway("http://opencode.ai:8080/v1"));
        // 域名后缀匹配，不能误伤
        assert!(!is_opencode_gateway("https://api.deepseek.com"));
        assert!(!is_opencode_gateway("https://notopencode.ai"));
        assert!(!is_opencode_gateway("https://opencode.ai.evil.com"));
        assert!(!is_opencode_gateway("https://api.openai.com"));
    }

    #[test]
    fn test_opencode_session_headers() {
        let headers = opencode_session_headers("sess-123");
        assert!(headers
            .iter()
            .any(|(k, v)| k == "x-opencode-session" && v == "sess-123"));
        assert!(headers
            .iter()
            .any(|(k, v)| k == "User-Agent" && v.starts_with("i-rs-claw/")));
        assert_ne!(new_opencode_session_id(), new_opencode_session_id());
    }
}
