pub mod error;
pub mod openai;
pub mod anthropic;
pub mod ollama;

use crate::config::Config;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::mpsc;

pub type StreamRx = mpsc::Receiver<StreamEvent>;

#[derive(Debug, Clone)]
pub enum LlmMessage {
    System(String),
    User(String),
    Assistant(String),
    AssistantWithReasoning { content: String, reasoning: String, tool_calls: Vec<ToolCall> },
    Tool { name: String, content: String, call_id: String },
    ToolCall { id: String, name: String, args: Value },
}

pub struct LlmResponse {
    pub content: Option<String>,
    pub reasoning: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: Value,
}

#[derive(Debug, Clone)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct StreamEvent {
    pub kind: StreamEventKind,
}

#[derive(Debug, Clone)]
pub enum StreamEventKind {
    Token(String),
    Reasoning(String),
    ToolCall { id: String, name: String, args: Value },
    Done { content: Option<String>, usage: Option<Usage> },
    Error(String),
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    async fn stream(&self, messages: &[LlmMessage], tool_defs: &[Value]) -> StreamRx;
    async fn chat(&self, messages: &[LlmMessage], tool_defs: &[Value]) -> anyhow::Result<LlmResponse>;
}

pub fn create_provider(config: &Config) -> anyhow::Result<Box<dyn LlmProvider>> {
    match config.provider.as_str() {
        "openai" => Ok(Box::new(openai::OpenAiProvider::new(config)?)),
        "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::new(config)?)),
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(config)?)),
        name => anyhow::bail!("Unknown provider: {}. Supported: openai, anthropic, ollama", name),
    }
}
