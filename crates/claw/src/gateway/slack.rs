use crate::gateway::{GatewayEvent, PlatformAdapter};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Configuration for the Slack bot adapter.
pub struct SlackConfig {
    pub bot_token: String,
    pub app_token: String,
    pub enabled: bool,
}

/// Slack bot adapter using the Web API via reqwest.
pub struct SlackAdapter {
    config: SlackConfig,
    client: reqwest::Client,
}

impl SlackAdapter {
    pub fn new(config: SlackConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self { config, client }
    }
}

#[async_trait]
impl PlatformAdapter for SlackAdapter {
    fn name(&self) -> &str {
        "slack"
    }

    async fn start(&self, event_tx: mpsc::UnboundedSender<GatewayEvent>) {
        let name = self.name().to_string();
        let _ = event_tx.send(GatewayEvent::Error {
            platform: name.clone(),
            error: "Slack adapter requires Socket Mode WebSocket integration for event reception. \
                    Use Slack Events API with a public endpoint for production.".to_string(),
        });
    }

    async fn send_message(&self, chat_id: &str, text: &str) {
        let url = "https://slack.com/api/chat.postMessage";
        let body = serde_json::json!({
            "channel": chat_id,
            "text": text,
            "mrkdwn": true,
        });
        let _ = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .json(&body)
            .send()
            .await;
    }

    async fn stop(&self) {}
}
