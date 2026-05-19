use crate::gateway::{GatewayEvent, PlatformAdapter};
use async_trait::async_trait;
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

const WECHAT_API_BASE: &str = "https://ilinkai.weixin.qq.com";

/// Configuration for the WeChat iLink Bot adapter.
#[allow(dead_code)]
pub struct WeChatConfig {
    pub enabled: bool,
    pub agent_id: String,
}

/// Persisted credentials from QR login.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WeChatCredentials {
    bot_token: String,
    base_url: String,
}

/// WeChat iLink Bot adapter using the official Bot API (personal WeChat).
///
/// Authentication is via QR code scan on first run instead of a static token.
/// Credentials are persisted to `~/.i-rs-claw/claw/wechat_credentials.json`.
pub struct WeChatAdapter {
    client: reqwest::Client,
    credentials: Arc<Mutex<Option<WeChatCredentials>>>,
    /// Map of user_id -> context_token for tracking reply context.
    reply_tokens: Arc<Mutex<HashMap<String, String>>>,
    /// Map of user_id -> typing_ticket from getConfig (cached per-user).
    typing_tickets: Arc<Mutex<HashMap<String, String>>>,
    credentials_path: PathBuf,
}

fn make_x_wechat_uin() -> String {
    let uin: u32 = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        & 0xFFFF_FFFF) as u32;
    base64::engine::general_purpose::STANDARD.encode(uin.to_string())
}

impl WeChatAdapter {
    pub fn new(_config: WeChatConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");
        let credentials_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".i-rs-claw")
            .join("claw")
            .join("wechat_credentials.json");
        Self {
            client,
            credentials: Arc::new(Mutex::new(None)),
            reply_tokens: Arc::new(Mutex::new(HashMap::new())),
            typing_tickets: Arc::new(Mutex::new(HashMap::new())),
            credentials_path,
        }
    }

    /// Load saved credentials from disk.
    fn load_credentials(&self) -> Option<WeChatCredentials> {
        if !self.credentials_path.exists() {
            return None;
        }
        std::fs::read_to_string(&self.credentials_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    }

    /// Save credentials to disk.
    fn save_credentials(&self, creds: &WeChatCredentials) {
        if let Some(parent) = self.credentials_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(creds) {
            let _ = std::fs::write(&self.credentials_path, json);
        }
    }

    /// Fetch and cache the typing_ticket from getConfig endpoint.
    async fn get_typing_ticket(&self, user_id: &str) -> Option<String> {
        // Return cached ticket if available for this user
        if let Some(ticket) = self.typing_tickets.lock().unwrap().get(user_id).cloned() {
            return Some(ticket);
        }

        // Need credentials to make the request
        let bot_token = self.credentials.lock().unwrap().clone()?.bot_token;

        let url = format!("{}/ilink/bot/getconfig", WECHAT_API_BASE);
        let uin = make_x_wechat_uin();
        // getConfig requires ilink_user_id in the request body
        let body = serde_json::json!({
            "ilink_user_id": user_id,
        });

        match self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("AuthorizationType", "ilink_bot_token")
            .header("X-WECHAT-UIN", &uin)
            .header("Authorization", format!("Bearer {}", bot_token))
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                match resp.text().await {
                    Ok(raw) => {
                        // Try to parse as JSON
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                            if let Some(ticket) = json["typing_ticket"].as_str() {
                                let ticket = ticket.to_string();
                                self.typing_tickets.lock().unwrap().insert(user_id.to_string(), ticket.clone());
                                Some(ticket)
                            } else {
                                eprintln!("[Gateway/WeChat] getConfig ({}): no typing_ticket in {}",
                                    status, raw);
                                None
                            }
                        } else {
                            eprintln!("[Gateway/WeChat] getConfig ({}): parse error: {}",
                                status, raw);
                            None
                        }
                    }
                    Err(e) => {
                        eprintln!("[Gateway/WeChat] getConfig read body error: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("[Gateway/WeChat] getConfig HTTP error: {}", e);
                None
            }
        }
    }

    /// Perform QR code login flow.
    async fn qr_login(&self) -> Result<WeChatCredentials, String> {
        let client = &self.client;

        // Step 1: Get QR code
        eprintln!("[Gateway/WeChat] Requesting QR code for login...");
        let qr_resp: serde_json::Value = client
            .get(format!("{}/ilink/bot/get_bot_qrcode?bot_type=3", WECHAT_API_BASE))
            .send()
            .await
            .map_err(|e| format!("Failed to get QR code: {}", e))?
            .json()
            .await
            .map_err(|e| format!("Failed to parse QR response: {}", e))?;

        let qrcode = qr_resp["qrcode"]
            .as_str()
            .ok_or("Missing qrcode in response")?
            .to_string();
        let qrcode_img = qr_resp["qrcode_img_content"]
            .as_str()
            .unwrap_or("(no image)");

        // Step 2: Display QR to user
        eprintln!("[Gateway/WeChat] ================================================");
        eprintln!("[Gateway/WeChat]  Scan the QR code with WeChat to log in");
        eprintln!("[Gateway/WeChat]  Open this link in your browser and scan:");
        eprintln!("[Gateway/WeChat]  {}", qrcode_img);
        eprintln!("[Gateway/WeChat] ================================================");
        eprintln!("[Gateway/WeChat] Waiting for QR scan...");

        // Step 3: Poll until scan confirmed
        loop {
            let status_resp: serde_json::Value = client
                .get(format!(
                    "{}/ilink/bot/get_qrcode_status?qrcode={}",
                    WECHAT_API_BASE, qrcode
                ))
                .send()
                .await
                .map_err(|e| format!("QR status poll failed: {}", e))?
                .json()
                .await
                .map_err(|e| format!("QR status parse failed: {}", e))?;

            let status = status_resp["status"].as_str().unwrap_or("pending");
            if status == "confirmed" {
                let bot_token = status_resp["bot_token"]
                    .as_str()
                    .ok_or("Missing bot_token")?
                    .to_string();
                let base_url = status_resp["baseurl"]
                    .as_str()
                    .unwrap_or(WECHAT_API_BASE)
                    .to_string();
                let creds = WeChatCredentials { bot_token, base_url };
                self.save_credentials(&creds);
                eprintln!("[Gateway/WeChat] Login confirmed. Starting message polling.");
                return Ok(creds);
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
}

#[async_trait]
impl PlatformAdapter for WeChatAdapter {
    fn name(&self) -> &str {
        "wechat"
    }

    async fn start(&self, event_tx: mpsc::UnboundedSender<GatewayEvent>) {
        let client = self.client.clone();
        let name = self.name().to_string();

        // QR login (lazy: on first start, or load saved credentials)
        let credentials = match self.load_credentials() {
            Some(c) => {
                eprintln!("[Gateway/WeChat] Loaded saved credentials.");
                c
            }
            None => match self.qr_login().await {
                Ok(c) => c,
                Err(e) => {
                    let _ = event_tx.send(GatewayEvent::Error {
                        platform: name,
                        error: format!("WeChat login failed: {}", e),
                    });
                    return;
                }
            },
        };

        // Set credentials for send_message to use
        {
            *self.credentials.lock().unwrap() = Some(credentials.clone());
        }

        let bot_token = credentials.bot_token;
        let reply_tokens = self.reply_tokens.clone();

        tokio::spawn(async move {
            // The get_updates_buf cursor - similar to Telegram's offset
            let mut get_updates_buf = String::new();

            loop {
                // Build headers
                let uin = make_x_wechat_uin();
                let url = format!("{}/ilink/bot/getupdates", WECHAT_API_BASE);
                let body = serde_json::json!({
                    "get_updates_buf": get_updates_buf,
                    "base_info": { "channel_version": "1.0.2" }
                });

                match client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .header("AuthorizationType", "ilink_bot_token")
                    .header("X-WECHAT-UIN", &uin)
                    .header("Authorization", format!("Bearer {}", bot_token))
                    .json(&body)
                    .send()
                    .await
                {
                    Ok(resp) => {
                        match resp.json::<serde_json::Value>().await {
                            Ok(json) => {
                                // Update cursor
                                if let Some(buf) = json["get_updates_buf"].as_str() {
                                    get_updates_buf = buf.to_string();
                                }

                                // Process messages
                                if let Some(msgs) = json["msgs"].as_array() {
                                    for msg in msgs {
                                        let msg_type = msg["message_type"].as_i64().unwrap_or(0);
                                        // message_type 1 = user message
                                        if msg_type != 1 {
                                            continue;
                                        }

                                        let from_user_id = msg["from_user_id"]
                                            .as_str()
                                            .unwrap_or("")
                                            .to_string();
                                        if from_user_id.is_empty() {
                                            continue;
                                        }

                                        // Extract text content
                                        let text = msg["item_list"]
                                            .as_array()
                                            .and_then(|items| items.first())
                                            .and_then(|item| item["text_item"]["text"].as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        if text.is_empty() {
                                            continue;
                                        }

                                        eprintln!(
                                            "[Gateway/WeChat] Msg from {}: {}",
                                            from_user_id, text
                                        );

                                        // Store context_token for reply mapping
                                        if let Some(token) = msg["context_token"].as_str() {
                                            reply_tokens.lock().unwrap().insert(
                                                from_user_id.clone(),
                                                token.to_string(),
                                            );
                                            eprintln!(
                                                "[Gateway/WeChat] Stored context_token for {}",
                                                from_user_id
                                            );
                                        } else {
                                            eprintln!(
                                                "[Gateway/WeChat] No context_token for msg from {}",
                                                from_user_id
                                            );
                                        }

                                        let _ = event_tx.send(GatewayEvent::Message {
                                            platform: name.clone(),
                                            chat_id: from_user_id,
                                            user_id: msg["from_user_id"]
                                                .as_str()
                                                .unwrap_or("")
                                                .to_string(),
                                            text,
                                            agent_id: "default".to_string(),
                                        });
                                    }
                                }

                                // If there's a timeout_ms hint, use it for next poll spacing
                                // Server holds connection for up to 35s
                            }
                            Err(e) => {
                                let _ = event_tx.send(GatewayEvent::Error {
                                    platform: name.clone(),
                                    error: format!("WeChat getupdates parse error: {}", e),
                                });
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = event_tx.send(GatewayEvent::Error {
                            platform: name.clone(),
                            error: format!("WeChat getupdates failed: {}", e),
                        });
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });
    }

    async fn send_message(&self, chat_id: &str, text: &str) {
        let credentials = match self.credentials.lock().unwrap().clone() {
            Some(c) => c,
            None => {
                eprintln!("[Gateway/WeChat] No credentials to send message");
                return;
            }
        };

        // Look up context_token for this conversation
        let context_token = self.reply_tokens.lock().unwrap().get(chat_id).cloned();

        let url = format!("{}/ilink/bot/sendmessage", WECHAT_API_BASE);
        let uin = make_x_wechat_uin();

        let mut body = serde_json::json!({
            "msg": {
                "from_user_id": "",
                "to_user_id": chat_id,
                "client_id": format!("i-rs-{}", uuid::Uuid::new_v4().simple()),
                "message_type": 2,
                "message_state": 2,
                "item_list": [
                    { "type": 1, "text_item": { "text": text } }
                ]
            },
            "base_info": {
                "channel_version": "1.0.3"
            }
        });

        // context_token is required to associate reply with the correct conversation
        if let Some(token) = &context_token {
            body["msg"]["context_token"] = serde_json::Value::String(token.clone());
        }

        eprintln!(
            "[Gateway/WeChat] send_message to_user={} has_token={} uin_len={} text_len={}",
            chat_id,
            context_token.is_some(),
            uin.len(),
            text.len()
        );

        match self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("AuthorizationType", "ilink_bot_token")
            .header("X-WECHAT-UIN", &uin)
            .header("Authorization", format!("Bearer {}", credentials.bot_token))
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                match resp.text().await {
                    Ok(response_body) => {
                        eprintln!(
                            "[Gateway/WeChat] send_message ({}): {}",
                            status, response_body
                        );
                    }
                    Err(e) => {
                        eprintln!(
                            "[Gateway/WeChat] send_message ({}) but read body failed: {}",
                            status, e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("[Gateway/WeChat] send_message HTTP error: {}", e);
            }
        }
    }

    async fn send_typing(&self, chat_id: &str) {
        let Some(ticket) = self.get_typing_ticket(chat_id).await else {
            return;
        };

        let Some(credentials) = self.credentials.lock().unwrap().clone() else {
            return;
        };

        let url = format!("{}/ilink/bot/sendtyping", WECHAT_API_BASE);
        let uin = make_x_wechat_uin();
        // sendTyping uses ilink_user_id, typing_ticket, and status=1
        let body = serde_json::json!({
            "ilink_user_id": chat_id,
            "typing_ticket": ticket,
            "status": 1,
        });

        match self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("AuthorizationType", "ilink_bot_token")
            .header("X-WECHAT-UIN", &uin)
            .header("Authorization", format!("Bearer {}", credentials.bot_token))
            .json(&body)
            .send()
            .await
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    let raw = resp.text().await.unwrap_or_default();
                    eprintln!(
                        "[Gateway/WeChat] send_typing ({}): {}",
                        status, raw
                    );
                }
            }
            Err(e) => {
                eprintln!("[Gateway/WeChat] send_typing HTTP error: {}", e);
            }
        }
    }

    async fn stop(&self) {
        // No persistent connections to close
    }
}
