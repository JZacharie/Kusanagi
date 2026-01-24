use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct SlackMessage {
    pub channel: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_ts: Option<String>,
}

pub struct SlackClient {
    token: String,
    channel_id: String,
    client: reqwest::Client,
}

impl SlackClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let token = env::var("SLACK_BOT_TOKEN").unwrap_or_default();
        let channel_id = env::var("SLACK_CHANNEL_ID").unwrap_or_default();

        if token.is_empty() || channel_id.is_empty() {
            warn!("SLACK_BOT_TOKEN or SLACK_CHANNEL_ID not set, Slack integration will be mocked");
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        Ok(Self {
            token,
            channel_id,
            client,
        })
    }

    pub async fn send_message(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.token.is_empty() {
            info!("[MOCK SLACK] Sending message: {}", text);
            return Ok(());
        }

        let message = SlackMessage {
            channel: self.channel_id.clone(),
            text: text.to_string(),
            thread_ts: None,
        };

        let response = self.client.post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&message)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            error!("Slack API error: {}", error_text);
            return Err(format!("Slack API returned status {}: {}", status, error_text).into());
        }

        let result: serde_json::Value = response.json().await?;
        if result["ok"].as_bool().unwrap_or(false) {
            info!("Successfully sent message to Slack");
            Ok(())
        } else {
            let error = result["error"].as_str().unwrap_or("unknown error");
            error!("Slack API error: {}", error);
            Err(format!("Slack API error: {}", error).into())
        }
    }

    pub async fn notify_alert(&self, title: &str, message: &str, severity: &str) -> Result<(), Box<dyn std::error::Error>> {
        let emoji = match severity.to_lowercase().as_str() {
            "critical" | "error" => "🔴",
            "warning" => "🟠",
            _ => "🔵",
        };

        let text = format!("{} *{} Alert*\n*{}*\n{}", emoji, severity.to_uppercase(), title, message);
        self.send_message(&text).await
    }
}

#[derive(Debug, Deserialize)]
pub struct SlackEvent {
    pub token: String,
    pub challenge: Option<String>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub event: Option<SlackEventDetail>,
}

#[derive(Debug, Deserialize)]
pub struct SlackEventDetail {
    pub text: Option<String>,
    pub user: Option<String>,
    pub channel: Option<String>,
}

pub async fn handle_webhook(event: web::Json<SlackEvent>) -> Result<HttpResponse, actix_web::Error> {
    info!("Received Slack event: {:?}", event);

    if let Some(challenge) = &event.challenge {
        return Ok(HttpResponse::Ok().json(serde_json::json!({ "challenge": challenge })));
    }

    if let Some(detail) = &event.event {
        if let Some(text) = &detail.text {
            // Forward Slack message to MQTT
            let topic = "kusanagi/slack/incoming";
            let _ = crate::mqtt::publish_message(topic, text).await;
        }
    }

    Ok(HttpResponse::Ok().finish())
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/slack")
            .route("/events", web::post().to(handle_webhook)),
    );
}

/// Start a background task to monitor cluster health and send Slack alerts
pub async fn start_alert_monitoring_task(client: kube::Client) {
    actix_rt::spawn(async move {
        info!("Starting Slack alert monitoring task");
        let mut last_error_pods = 0;
        let mut last_unhealthy_apps = 0;

        let slack_client = match SlackClient::new() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to initialize Slack client for monitoring: {}", e);
                return;
            }
        };

        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;

            // Check Pods
            if let Ok(pods_status) = crate::pods::get_pods_status(&client).await {
                if pods_status.error_pods > last_error_pods {
                    let _ = slack_client.notify_alert(
                        "Infrastructure Issue",
                        &format!("Detected {} pods in error state.", pods_status.error_pods),
                        "error"
                    ).await;
                }
                last_error_pods = pods_status.error_pods;
            }

            // Check ArgoCD
            if let Ok(argocd_status) = crate::argocd::get_argocd_status(&client).await {
                if argocd_status.unhealthy > last_unhealthy_apps {
                    let _ = slack_client.notify_alert(
                        "ArgoCD Sync Alert",
                        &format!("Detected {} unhealthy applications.", argocd_status.unhealthy),
                        "warning"
                    ).await;
                }
                last_unhealthy_apps = argocd_status.unhealthy;
            }
        }
    });
}
