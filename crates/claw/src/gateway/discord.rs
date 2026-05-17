use crate::gateway::{GatewayEvent, PlatformAdapter};
use async_trait::async_trait;
use tokio::sync::mpsc;

/// Configuration for the Discord bot adapter.
pub struct DiscordConfig {
    pub bot_token: String,
    pub enabled: bool,
}

/// Discord bot adapter using the REST API via reqwest.
pub struct DiscordAdapter {
    config: DiscordConfig,
    client: reqwest::Client,
}

impl DiscordAdapter {
    pub fn new(config: DiscordConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        Self { config, client }
    }
}

#[async_trait]
impl PlatformAdapter for DiscordAdapter {
    fn name(&self) -> &str {
        "discord"
    }

    async fn start(&self, event_tx: mpsc::UnboundedSender<GatewayEvent>) {
        let client = self.client.clone();
        let _token = self.config.bot_token.clone();
        let name = self.name().to_string();

        tokio::spawn(async move {
            // Register for gateway intents
            let url = "https://discord.com/api/v10/gateway";
            match client.get(url).send().await {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        // For now, log the gateway URL and use a simple polling approach
                        // Full WebSocket gateway integration would use tokio-tungstenite
                        eprintln!("[Gateway/Discord] Gateway URL: {:?}", json.get("url"));
                    }
                    let _ = event_tx.send(GatewayEvent::Error {
                        platform: name.clone(),
                        error: "Discord adapter requires WebSocket gateway integration. \
                                Use the REST API for slash commands.".to_string(),
                    });
                }
                Err(e) => {
                    let _ = event_tx.send(GatewayEvent::Error {
                        platform: name.clone(),
                        error: format!("Discord gateway failed: {}", e),
                    });
                }
            }
        });
    }

    async fn send_message(&self, chat_id: &str, text: &str) {
        let url = format!("https://discord.com/api/v10/channels/{}/messages", chat_id);
        let body = serde_json::json!({
            "content": text,
        });
        let _ = self
            .client
            .post(&url)
            .header("Authorization", format!("Bot {}", self.config.bot_token))
            .json(&body)
            .send()
            .await;
    }

    async fn stop(&self) {}
}
