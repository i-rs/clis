#![allow(dead_code)]

#[cfg(feature = "gateway-telegram")]
pub mod telegram;
#[cfg(feature = "gateway-discord")]
pub mod discord;
#[cfg(feature = "gateway-slack")]
pub mod slack;
#[cfg(feature = "gateway-wechat")]
pub mod wechat;

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Event emitted by a platform adapter when a message is received or an error occurs.
#[derive(Debug)]
pub enum GatewayEvent {
    /// A user message from a social platform.
    Message {
        /// Platform name (e.g. "telegram", "discord").
        platform: String,
        /// Platform-specific chat/conversation ID.
        chat_id: String,
        /// Platform-specific user ID.
        user_id: String,
        /// The message text content.
        text: String,
    },
    /// An error from a platform adapter.
    Error {
        /// Platform name.
        platform: String,
        /// Error description.
        error: String,
    },
}

/// Adapter interface for social platform integration.
///
/// Each platform (Telegram, Discord, Slack, WeChat) implements this trait
/// to receive and send messages through the gateway server.
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// Human-readable platform name (e.g. "telegram", "discord").
    fn name(&self) -> &str;

    /// Start listening for incoming messages from this platform.
    ///
    /// Implementations should spawn background tasks (e.g. long polling loops)
    /// and forward incoming messages via `event_tx` as `GatewayEvent::Message`.
    async fn start(&self, event_tx: mpsc::UnboundedSender<GatewayEvent>);

    /// Send a text message to a specific chat/conversation.
    async fn send_message(&self, chat_id: &str, text: &str);

    /// Stop the adapter and clean up resources.
    async fn stop(&self);
}

/// Gateway server that bridges social platforms with the AppCore LLM engine.
///
/// Manages multiple `PlatformAdapter` instances, forwards user messages
/// to the LLM, and sends responses back through the appropriate adapter.
pub struct GatewayServer {
    adapters: Vec<Box<dyn PlatformAdapter>>,
}

impl GatewayServer {
    /// Create a new gateway server with no adapters.
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    /// Register a platform adapter.
    pub fn register(&mut self, adapter: Box<dyn PlatformAdapter>) {
        self.adapters.push(adapter);
    }

    /// Get the number of registered adapters.
    pub fn adapter_count(&self) -> usize {
        self.adapters.len()
    }

    /// Run the gateway server.
    ///
    /// Starts all registered adapters, then enters the main event loop.
    /// For each incoming message, it processes the text through the LLM
    /// and sends the response back via the originating platform's adapter.
    /// Blocks until all adapters have stopped.
    pub async fn run(self, core: Arc<crate::core::AppCore>) {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<GatewayEvent>();

        // Start all adapters
        for adapter in &self.adapters {
            adapter.start(event_tx.clone()).await;
        }

        // Main event loop
        while let Some(event) = event_rx.recv().await {
            match event {
                GatewayEvent::Message {
                    platform,
                    chat_id,
                    user_id: _,
                    text,
                } => {
                    let core = core.clone();
                    // Process the message and get the LLM response
                    let response = Self::process_message(&core, &text).await;
                    // Find the originating adapter and send the response
                    for adapter in &self.adapters {
                        if adapter.name() == platform {
                            adapter.send_message(&chat_id, &response).await;
                            break;
                        }
                    }
                }
                GatewayEvent::Error { platform, error } => {
                    eprintln!("[Gateway/{}] Error: {}", platform, error);
                }
            }
        }
    }

    /// Process a single user message through the LLM.
    /// Returns the accumulated text response.
    async fn process_message(core: &crate::core::AppCore, text: &str) -> String {
        let provider = crate::provider::create_provider(&core.config);
        let messages = vec![serde_json::json!({"role": "user", "content": text})];

        let (tx, mut rx) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let _ = provider.stream_chat(&messages, &[], &tx).await;
        });

        let mut response = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                crate::llm::LlmEvent::Token(t) => response.push_str(&t),
                crate::llm::LlmEvent::Error(e) => {
                    response = format!("Error: {}", e);
                    break;
                }
                crate::llm::LlmEvent::Done(_, _) => break,
                _ => {}
            }
        }
        response
    }
}
