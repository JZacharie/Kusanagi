//! WebSocket Handler - Axum Migration

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use std::time::Duration;
use tracing::{debug, error, info};

use crate::state::AppState;

/// WebSocket notification message types
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum NotificationMessage {
    #[serde(rename = "connected")]
    Connected { message: String },
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: String },
    #[serde(rename = "alert")]
    Alert {
        severity: String,
        title: String,
        message: String,
        source: String,
        timestamp: String,
    },
}

/// WebSocket upgrade handler
pub async fn ws_notifications_handler(
    _state: State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handle the WebSocket connection
async fn handle_socket(mut socket: WebSocket) {
    info!("WebSocket client connected");

    // Send welcome message
    let welcome = NotificationMessage::Connected {
        message: "Connected to Kusanagi notifications".to_string(),
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        if socket.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }

    // Heartbeat interval
    let mut interval = tokio::time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let hb = NotificationMessage::Heartbeat {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };
                if let Ok(json) = serde_json::to_string(&hb) {
                    if socket.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
            }
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        debug!("Received WebSocket text: {}", text);
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    info!("WebSocket client disconnected");
}
