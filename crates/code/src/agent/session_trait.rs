use crate::provider::{LlmMessage, Usage};
use super::event::AgentEvent;
use tokio::sync::mpsc;
use std::future::Future;
use std::pin::Pin;

#[allow(dead_code)]
pub struct ChatInput {
    pub prompt: String,
    pub history: Vec<LlmMessage>,
}

#[allow(dead_code)]
pub struct ChatOutput {
    pub text: String,
    pub messages: Vec<LlmMessage>,
    pub usage: Option<Usage>,
}

#[allow(dead_code)]
pub trait ChatSession {
    fn run(
        &mut self,
        input: ChatInput,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ChatOutput>> + Send + '_>>;
}

#[allow(dead_code)]
pub trait StreamingChatSession {
    fn run_streaming(
        &mut self,
        input: ChatInput,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ChatOutput>> + Send + '_>>;
}
