pub mod telegram;
pub mod wechat;

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::RwLock;
use tokio::sync::mpsc;

/// Event emitted by a platform adapter when a message is received or an error occurs.
#[derive(Debug)]
pub enum GatewayEvent {
    /// A user message from a social platform.
    #[allow(dead_code)]
    Message {
        /// Platform name (e.g. "telegram", "wechat").
        platform: String,
        /// Platform-specific chat/conversation ID.
        chat_id: String,
        /// Platform-specific user ID.
        user_id: String,
        /// The message text content.
        text: String,
        /// Agent profile to use for processing this message.
        agent_id: String,
    },
    /// An error from a platform adapter.
    #[allow(dead_code)]
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

    /// Send a typing indicator to a specific chat/conversation.
    ///
    /// The default implementation is a no-op for platforms that don't
    /// support typing indicators. For supported platforms, this shows
    /// "正在输入..." or similar to the user while the bot is processing.
    async fn send_typing(&self, _chat_id: &str) {}

    /// Stop the adapter and clean up resources.
    #[allow(dead_code)]
    async fn stop(&self);
}

/// Gateway server that bridges social platforms with the AppCore LLM engine.
///
/// Manages multiple `PlatformAdapter` instances, forwards user messages
/// to the LLM (with session continuity), and sends responses back.
pub struct GatewayServer {
    adapters: Vec<Arc<dyn PlatformAdapter>>,
}

impl GatewayServer {
    /// Create a new gateway server with no adapters.
    pub fn new() -> Self {
        Self {
            adapters: Vec::new(),
        }
    }

    /// Register a platform adapter.
    #[allow(dead_code)]
    pub fn register(&mut self, adapter: Arc<dyn PlatformAdapter>) {
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
    /// Handles SIGINT (Ctrl+C) for graceful shutdown.
    pub async fn run(self, core: Arc<RwLock<i_rs_claw_core::core::AppCore>>) {
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<GatewayEvent>();

        // Start all adapters
        for adapter in &self.adapters {
            adapter.start(event_tx.clone()).await;
        }

        // Setup SIGINT handler for graceful shutdown
        let mut sigint = match signal(SignalKind::interrupt()) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("[Gateway] 无法设置信号处理器: {}", e);
                return;
            }
        };

        // Main event loop with Ctrl+C handling
        loop {
            tokio::select! {
                _ = sigint.recv() => {
                    tracing::info!("[Gateway] 收到中断信号，正在优雅关闭...");
                    break;
                }
                event = event_rx.recv() => {
                    match event {
                        Some(event) => self.handle_event(core.clone(), event).await,
                        None => break, // all adapters disconnected
                    }
                }
            }
        }

        // Graceful shutdown: stop all adapters
        for adapter in &self.adapters {
            adapter.stop().await;
        }
        tracing::info!("[Gateway] 已关闭");
    }

    /// Handle a single gateway event (message or error).
    async fn handle_event(&self, core: Arc<RwLock<i_rs_claw_core::core::AppCore>>, event: GatewayEvent) {
        match event {
            GatewayEvent::Message {
                platform,
                chat_id,
                user_id: _,
                text,
                agent_id,
            } => {
                // Find the originating adapter by index
                let adapter_idx = self.adapters.iter().position(|a| a.name() == platform);

                // Spawn periodic typing indicator while processing (max 5 min)
                const MAX_TYPING_SECS: u64 = 300;
                let typing_handle = if let Some(idx) = adapter_idx {
                    let adapter = self.adapters[idx].clone();
                    let cid = chat_id.clone();
                    Some(tokio::spawn(async move {
                        let start = std::time::Instant::now();
                        loop {
                            if start.elapsed().as_secs() >= MAX_TYPING_SECS {
                                break;
                            }
                            adapter.send_typing(&cid).await;
                            tokio::time::sleep(Duration::from_secs(5)).await;
                        }
                    }))
                } else {
                    None
                };

                // Process the message with session continuity + tool execution
                let response =
                    Self::process_message(&core, &platform, &chat_id, &text, &agent_id).await;

                // Stop the typing indicator
                if let Some(h) = typing_handle {
                    h.abort();
                }

                // Find the originating adapter and send the response
                if let Some(idx) = adapter_idx {
                    self.adapters[idx].send_message(&chat_id, &response).await;
                }
            }
            GatewayEvent::Error { platform, error } => {
                tracing::error!("[Gateway/{}] Error: {}", platform, error);
            }
        }
    }

    /// Process a user message through the full LLM chat loop with session persistence.
    ///
    /// Each {platform}:{chat_id} pair gets its own session for conversation continuity.
    /// Uses `AppCore::build_messages()` and `engine::chat_loop()` for multi-round
    /// streaming with tool call execution.
    #[tracing::instrument(skip(core))]
    async fn process_message(
        core: &Arc<RwLock<i_rs_claw_core::core::AppCore>>,
        platform: &str,
        chat_id: &str,
        text: &str,
        agent_id: &str,
    ) -> String {
        let text_owned = text.to_string();
        let agent_id_owned = agent_id.to_string();

        let (session_uuid, _session_title, msgs, config, mcp) = {
            let mut core = core.write().await;
            let session_title = format!("gateway:{}:{}", platform, chat_id);

            let uuid = if let Some(found) = core
                .session_mgr
                .sessions()
                .iter()
                .find(|s| s.title == session_title)
                .map(|s| s.id.clone())
            {
                core.session_mgr.switch_to(&found);
                found
            } else {
                let new_id = core.session_mgr.create_session_for(&agent_id_owned, "default");
                core.session_mgr.rename_session(&new_id, &session_title);
                new_id
            };

            let saved = core.session_mgr.load_api_messages(&uuid);
            let msgs = core.build_messages_for(&[], &text_owned, &saved, None, &agent_id_owned);

            let resolved = core.config.agent_config(&agent_id_owned);
            let mut agent_config = core.config.clone();
            agent_config.enabled_tools = resolved.enabled_tools;
            let mcp = core.agent_store.mcp_registry_for(&agent_id_owned).clone();

            (uuid, session_title, msgs, agent_config, mcp)
        };

        // Spawn the multi-round chat loop (no lock held during streaming)
        let (tx, mut rx) = mpsc::unbounded_channel();
        let client = i_rs_claw_core::providers::shared_client();
        let provider = i_rs_claw_core::providers::create_provider_for(
            &client,
            config.provider,
            &config.api_key,
            &config.base_url,
            &config.model,
        );
        let http_client = i_rs_claw_core::providers::shared_client();
        tokio::spawn(async move {
            i_rs_claw_core::core::engine::chat_loop(
                provider,
                config,
                msgs,
                tx,
                mcp,
                Vec::new(),
                std::collections::HashMap::new(),
                http_client,
                None,
                None,
                std::sync::Arc::new(std::sync::Mutex::new(
                    i_rs_claw_core::core::checkpoint::CheckpointStore::new(20),
                )),
            )
            .await;
        });

        // Accumulate the response
        let mut response = String::new();
        while let Some(event) = rx.recv().await {
            match event {
                i_rs_claw_core::llm::LlmEvent::Token(t) => response.push_str(&t),
                i_rs_claw_core::llm::LlmEvent::Error(e) => {
                    response = format!("Error: {}", e);
                    break;
                }
                i_rs_claw_core::llm::LlmEvent::Done(api_msgs, _, _) => {
                    let mut core = core.write().await;
                    core.session_mgr.save_api_messages(&session_uuid, &api_msgs);
                    let msgs = vec![
                        crate::app::Message::User {
                            text: text_owned.clone(),
                        },
                        crate::app::Message::Assistant {
                            text: response.clone(),
                            reasoning: String::new(),
                            token_usage: None,
                        },
                    ];
                    if let Err(e) = core.session_mgr.persist_messages(&session_uuid, &msgs) {
                        tracing::error!("gateway persist_messages 失败: {}", e);
                    }
                    break;
                }
                _ => {}
            }
        }
        response
    }
}
