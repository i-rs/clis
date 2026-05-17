#[cfg(feature = "gateway-telegram")]
pub mod telegram;
#[cfg(feature = "gateway-wechat")]
pub mod wechat;

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Event emitted by a platform adapter when a message is received or an error occurs.
#[derive(Debug)]
pub enum GatewayEvent {
    /// A user message from a social platform.
    Message {
        /// Platform name (e.g. "telegram", "wechat").
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
/// Each platform (Telegram, WeChat) implements this trait to receive
/// and send messages through the gateway server.
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// Human-readable platform name (e.g. "telegram", "wechat").
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
/// to the LLM (with session continuity), and sends responses back.
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
    /// For each incoming message, it uses the full session-aware chat loop
    /// (with tool execution) and sends the response back via the originating
    /// platform's adapter. Blocks until all adapters have stopped.
    pub async fn run(self, core: Arc<Mutex<crate::core::AppCore>>) {
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
                    // Process the message with session continuity + tool execution
                    let response =
                        Self::process_message(&core, &platform, &chat_id, &text).await;
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

    /// Process a user message through the full LLM chat loop with session persistence.
    ///
    /// Each {platform}:{chat_id} pair gets its own session for conversation continuity.
    /// Uses `AppCore::build_messages()` and `engine::chat_loop()` for multi-round
    /// streaming with tool call execution.
    async fn process_message(
        core: &Arc<Mutex<crate::core::AppCore>>,
        platform: &str,
        chat_id: &str,
        text: &str,
    ) -> String {
        let text_owned = text.to_string();

        // Build messages with session context (lock held briefly)
        let (session_id, msgs, config) = {
            let mut core = core.lock().unwrap();
            let session_id = format!("gateway:{}:{}", platform, chat_id);

            // Try to switch to existing session, or create a new one
            if !core.session_mgr.switch_to(&session_id) {
                let new_id = core.session_mgr.create_session();
                core.session_mgr.rename_session(&new_id, &session_id);
                // Do NOT switch again -- create_session already made it current.
                // But switch_to didn't find it, so current is now the new session.
            }

            let saved = core.session_mgr.load_api_messages(&session_id);
            let msgs = core.build_messages(&[], &text_owned, &saved, None);
            let config = core.config.clone();
            (session_id, msgs, config)
        };

        // Spawn the multi-round chat loop (no lock held during streaming)
        let (tx, mut rx) = mpsc::unbounded_channel();
        let provider = crate::provider::create_provider(&config);
        tokio::spawn(async move {
            crate::core::engine::chat_loop(provider, config, msgs, tx).await;
        });

        // Accumulate the response
        let mut response = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                crate::llm::LlmEvent::Token(t) => response.push_str(&t),
                crate::llm::LlmEvent::Error(e) => {
                    response = format!("Error: {}", e);
                    break;
                }
                crate::llm::LlmEvent::Done(api_msgs, _) => {
                    // Lock again only for persistence
                    let mut core = core.lock().unwrap();
                    core.session_mgr
                        .save_api_messages(&session_id, &api_msgs);
                    core.session_mgr.append_message("user", &text_owned, None);
                    core.session_mgr
                        .append_message("assistant", &response, None);
                    break;
                }
                _ => {}
            }
        }
        response
    }
}
