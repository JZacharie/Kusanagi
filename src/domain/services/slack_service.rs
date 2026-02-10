use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{error, info, warn};

// Static flag to track if SSL errors have been encountered
static SSL_ERROR_LOGGED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackMessage {
    pub channel: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
}

#[derive(Clone)]
pub struct SlackService {
    token: String,
    channel_id: String,
    client: Client,
}

impl Default for SlackService {
    fn default() -> Self {
        Self::new()
    }
}

impl SlackService {
    pub fn new() -> Self {
        let token = env::var("SLACK_BOT_TOKEN").unwrap_or_default();
        let channel_id = env::var("SLACK_CHANNEL_ID").unwrap_or_default();

        if token.is_empty() {
            warn!("⚠️ Slack: Token not set. Integration disabled.");
        } else {
            info!("💬 Slack: Integration enabled for channel {}", channel_id);
        }

        // Create HTTP client with native certificates loaded from system
        let client = match Self::create_client_with_native_certs() {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "⚠️ Failed to create client with native certs: {}, using default",
                    e
                );
                Client::default()
            }
        };

        Self {
            token,
            channel_id,
            client,
        }
    }

    /// Create HTTP client with system certificates
    fn create_client_with_native_certs() -> Result<Client, reqwest::Error> {
        // Build client - native-tls will automatically use system certificates
        Client::builder()
            .user_agent("Kusanagi/0.3.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
    }

    pub async fn send_alert(&self, title: &str, message: &str, severity: &str) -> bool {
        if self.token.is_empty() {
            return false;
        }

        let emoji = match severity.to_lowercase().as_str() {
            "critical" | "error" => "🔴",
            "warning" => "🟠",
            "success" | "good" => "🟢",
            _ => "🔵",
        };

        let text = format!(
            "{} *{} Alert*\n*{}*\n{}",
            emoji,
            severity.to_uppercase(),
            title,
            message
        );
        self.post_message(&text).await
    }

    pub async fn post_message(&self, text: &str) -> bool {
        if self.token.is_empty() || self.channel_id.is_empty() {
            return false;
        }

        let msg = SlackMessage {
            channel: self.channel_id.clone(),
            text: text.to_string(),
            thread_ts: None,
        };

        match self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&msg)
            .send()
            .await
        {
            Ok(res) => {
                if res.status().is_success() {
                    true
                } else {
                    let status = res.status();
                    let body = res
                        .text()
                        .await
                        .unwrap_or_else(|_| "Could not read body".to_string());
                    error!("Slack API returned error: {} - Body: {}", status, body);
                    false
                }
            }
            Err(e) => {
                // Check if it's an SSL/TLS error
                let err_str = format!("{:?}", e);
                if err_str.contains("ssl")
                    || err_str.contains("tls")
                    || err_str.contains("handshake")
                {
                    // Only log the SSL error once to avoid spam
                    if !SSL_ERROR_LOGGED.load(Ordering::Relaxed) {
                        warn!("⚠️ Slack SSL/TLS error (this will be logged only once). This usually means CA certificates are missing in the Docker image.");
                        warn!("💡 To fix: Install ca-certificates in your Dockerfile: RUN apt-get update && apt-get install -y ca-certificates");
                        SSL_ERROR_LOGGED.store(true, Ordering::Relaxed);
                    }
                } else {
                    error!("Failed to send Slack message: {:?}", e);
                }
                false
            }
        }
    }
}
