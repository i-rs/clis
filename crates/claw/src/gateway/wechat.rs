use crate::gateway::{GatewayEvent, PlatformAdapter};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Configuration for the WeCom (WeChat for Enterprise) bot adapter.
pub struct WeChatConfig {
    pub webhook_url: String,
    pub secret: Option<String>,
    pub enabled: bool,
}

/// WeCom bot adapter using the Bot API via reqwest.
pub struct WeChatAdapter {
    config: WeChatConfig,
    client: reqwest::Client,
}

impl WeChatAdapter {
    pub fn new(config: WeChatConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self { config, client }
    }
}

#[async_trait]
impl PlatformAdapter for WeChatAdapter {
    fn name(&self) -> &str {
        "wechat"
    }

    async fn start(&self, event_tx: mpsc::UnboundedSender<GatewayEvent>) {
        let name = self.name().to_string();
        let _ = event_tx.send(GatewayEvent::Error {
            platform: name,
            error: "WeChat adapter requires webhook configuration in WeCom admin panel. \
                    Messages are received via HTTP callback, not polling.".to_string(),
        });
    }

    async fn send_message(&self, chat_id: &str, text: &str) {
        // WeCom bot sends messages via webhook URL
        // If a specific chat_id is provided, use the group bot API
        let url = if chat_id.is_empty() || chat_id == "default" {
            self.config.webhook_url.clone()
        } else {
            // For specific conversations, would need access_token from WeCom API
            // https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=KEY
            self.config.webhook_url.clone()
        };

        let body = serde_json::json!({
            "msgtype": "markdown",
            "markdown": {
                "content": text,
            }
        });
        let _ = self.client.post(&url).json(&body).send().await;
    }

    async fn stop(&self) {}
}
