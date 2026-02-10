//! Slack notification handler

use axum::{extract::State, response::IntoResponse, Json};
use serde::Deserialize;
use tracing::{debug, error, info};

use kusanagi::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SlackNotification {
    pub message: String,
    pub channel: Option<String>,
}

/// Send Slack notification
pub async fn send_slack_notification(
    State(state): State<AppState>,
    Json(notification): Json<SlackNotification>,
) -> impl IntoResponse {
    // Get webhook URL from env
    let webhook_url = match std::env::var("SLACK_WEBHOOK_URL") {
        Ok(url) => url,
        Err(_) => {
            return Json(serde_json::json!({
                "success": false,
                "error": "SLACK_WEBHOOK_URL not configured"
            }));
        }
    };

    debug!("Sending Slack notification");

    // Build the payload
    let payload = serde_json::json!({
        "text": notification.message,
        "channel": notification.channel
    });

    // Send the notification using native-tls reqwest
    match state
        .http_client
        .post(&webhook_url)
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                info!("Slack notification sent successfully");
                Json(serde_json::json!({
                    "success": true,
                    "message": "Notification sent"
                }))
            } else {
                error!("Slack notification failed: {}", status);
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Slack returned status: {}", status)
                }))
            }
        }
        Err(e) => {
            error!("Failed to send Slack notification: {}", e);
            Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to send: {}", e)
            }))
        }
    }
}
