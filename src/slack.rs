use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, error, info, warn};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

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
    signing_secret: String,
    bot_user_id: String,
    client: reqwest::Client,
}

impl SlackClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let token = env::var("SLACK_BOT_TOKEN").unwrap_or_default();
        let channel_id = env::var("SLACK_CHANNEL_ID").unwrap_or_default();
        let signing_secret = env::var("SLACK_SIGNING_SECRET").unwrap_or_default();
        let bot_user_id = env::var("SLACK_BOT_USER_ID").unwrap_or_default();

        if token.is_empty() || channel_id.is_empty() {
            warn!("SLACK_BOT_TOKEN or SLACK_CHANNEL_ID not set, Slack integration will be limited");
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        Ok(Self {
            token,
            channel_id,
            signing_secret,
            bot_user_id,
            client,
        })
    }

    pub fn verify_signature(&self, timestamp: &str, body: &str, signature: &str) -> bool {
        if self.signing_secret.is_empty() {
            return true; // Skip if no secret set
        }

        debug!("Verifying Slack signature. Timestamp: {}, Body length: {}, Signature: {}", timestamp, body.len(), signature);
        let basestring = format!("v0:{}:{}", timestamp, body);
        let mut mac = Hmac::<Sha256>::new_from_slice(self.signing_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(basestring.as_bytes());
        let result = mac.finalize();
        let expected_signature = format!("v0={}", hex::encode(result.into_bytes()));

        let is_valid = signature == expected_signature;
        if !is_valid {
            warn!("Slack signature mismatch! Expected: {}, Received: {}", expected_signature, signature);
        } else {
            debug!("Slack signature verified successfully");
        }
        is_valid
    }

    pub async fn send_message(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.send_response(text, None, None).await
    }

    pub async fn send_response(&self, text: &str, channel: Option<&str>, thread_ts: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        if self.token.is_empty() {
            info!("[MOCK SLACK] Sending message to {}: {}", channel.unwrap_or(&self.channel_id), text);
            return Ok(());
        }

        let message = SlackMessage {
            channel: channel.unwrap_or(&self.channel_id).to_string(),
            text: text.to_string(),
            thread_ts: thread_ts.map(|s| s.to_string()),
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
    pub bot_id: Option<String>,
    pub channel: Option<String>,
    pub ts: Option<String>,
    pub thread_ts: Option<String>,
}

pub async fn handle_webhook(
    req: actix_web::HttpRequest,
    event_json: web::Json<SlackEvent>,
) -> Result<HttpResponse, actix_web::Error> {
    let slack_client = SlackClient::new().map_err(|e| {
        error!("Failed to create Slack client: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;

    // Log headers for debugging
    for (name, value) in req.headers().iter() {
        debug!("Slack Webhook Header: {}: {:?}", name, value);
    }

    // Log the event for debugging
    debug!("Received Slack event body: {:?}", event_json);

    if let Some(challenge) = &event_json.challenge {
        return Ok(HttpResponse::Ok().json(serde_json::json!({ "challenge": challenge })));
    }

    if let Some(detail) = &event_json.event {
        // Prevent infinite loops by ignoring bot messages
        if detail.bot_id.is_some() || detail.user.as_deref() == Some(&slack_client.bot_user_id) {
            return Ok(HttpResponse::Ok().finish());
        }

        if let Some(text) = &detail.text {
            // Forward Slack message to MQTT
            let topic = "kusanagi/slack/incoming";
            let _ = crate::mqtt::publish_message(topic, text).await;

            // Process message with AI if it's in a channel we care about
            if let Some(channel) = &detail.channel {
                let ai_request = crate::chat::ChatRequest {
                    message: text.clone(),
                };

                // Spawn the processing in a background task to avoid Slack timeouts
                let channel_clone = channel.clone();
                let thread_ts = detail.thread_ts.clone().or_else(|| detail.ts.clone());
                
                actix_rt::spawn(async move {
                    let client = match kube::Client::try_default().await {
                        Ok(c) => c,
                        Err(e) => {
                            error!("Failed to create K8s client for Slack AI: {}", e);
                            return;
                        }
                    };

                    let response = crate::chat::process_message(&client, ai_request).await;
                    
                    if let Ok(sc) = SlackClient::new() {
                        let _ = sc.send_response(&response.response, Some(&channel_clone), thread_ts.as_deref()).await;
                    }
                });
            }
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
