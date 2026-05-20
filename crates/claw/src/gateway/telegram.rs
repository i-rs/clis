use crate::gateway::{GatewayEvent, PlatformAdapter};
use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

const TELEGRAM_API_BASE: &str = "https://api.telegram.org/bot";

/// Configuration for the Telegram bot adapter.
#[allow(dead_code)]
pub struct TelegramConfig {
    pub bot_token: String,
    pub enabled: bool,
    pub agent_id: String,
}

/// Telegram bot adapter using the Bot API via reqwest.
pub struct TelegramAdapter {
    config: TelegramConfig,
    client: reqwest::Client,
    /// Handle for the background polling task, aborted on stop.
    task_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl TelegramAdapter {
    pub fn new(config: TelegramConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|e| {
                tracing::error!("[Gateway/Telegram] 创建 HTTP 客户端失败: {}", e);
                reqwest::Client::new()
            });
        Self {
            config,
            client,
            task_handle: Mutex::new(None),
        }
    }

    fn api_url(&self, method: &str) -> String {
        format!("{}{}/{}", TELEGRAM_API_BASE, self.config.bot_token, method)
    }
}

#[async_trait]
impl PlatformAdapter for TelegramAdapter {
    fn name(&self) -> &str {
        "telegram"
    }

    async fn start(&self, event_tx: mpsc::UnboundedSender<GatewayEvent>) {
        let client = self.client.clone();
        let token = self.config.bot_token.clone();
        let name = self.name().to_string();
        let agent_id = self.config.agent_id.clone();

        let handle = tokio::spawn(async move {
            let mut offset: i64 = 0;
            loop {
                let url = format!(
                    "https://api.telegram.org/bot{}/getUpdates?timeout=30&offset={}",
                    token, offset
                );

                match client.get(&url).send().await {
                    Ok(resp) => {
                        if let Ok(json) = resp.json::<serde_json::Value>().await {
                            if let Some(updates) = json["result"].as_array() {
                                for update in updates {
                                    if let Some(msg) = update.get("message") {
                                        let chat_id = msg["chat"]["id"].to_string();
                                        let user_id = msg["from"]["id"].to_string();
                                        let text = msg["text"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string();

                                        if !text.is_empty() {
                                            let _ = event_tx.send(GatewayEvent::Message {
                                                platform: name.clone(),
                                                chat_id,
                                                user_id,
                                                text,
                                                agent_id: agent_id.clone(),
                                            });
                                        }
                                    }
                                    // Update offset to acknowledge this update
                                    if let Some(update_id) = update["update_id"].as_i64() {
                                        offset = update_id + 1;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = event_tx.send(GatewayEvent::Error {
                            platform: name.clone(),
                            error: format!("Telegram getUpdates failed: {}", e),
                        });
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });

        *self.task_handle.lock().await = Some(handle);
    }

    async fn send_message(&self, chat_id: &str, text: &str) {
        let url = self.api_url("sendMessage");
        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown",
        });
        let _ = self.client.post(&url).json(&body).send().await;
    }

    async fn stop(&self) {
        if let Some(handle) = self.task_handle.lock().await.take() {
            handle.abort();
        }
    }
}
