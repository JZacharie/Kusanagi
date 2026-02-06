use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use reqwest::Client;
use std::env;

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

impl SlackService {
    pub fn new() -> Self {
        let token = env::var("SLACK_BOT_TOKEN").unwrap_or_default();
        let channel_id = env::var("SLACK_CHANNEL_ID").unwrap_or_default();
        
        if token.is_empty() {
            warn!("SLACK_BOT_TOKEN is not set. Slack integration disabled.");
        }

        Self {
            token,
            channel_id,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
        }
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

        let text = format!("{} *{} Alert*\n*{}*\n{}", emoji, severity.to_uppercase(), title, message);
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

        match self.client.post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&msg)
            .send()
            .await 
        {
            Ok(res) => {
                if res.status().is_success() {
                    true
                } else {
                    error!("Slack API returned error: {:?}", res.status());
                    false
                }
            },
            Err(e) => {
                error!("Failed to send Slack message: {}", e);
                false
            }
        }
    }
}
