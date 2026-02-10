//! WebSocket Handler - Axum Migration
//!
//! Interface layer for WebSocket notifications.
//! Migrated from actix-web-actors to axum::ws.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use std::time::Duration;
use tracing::{debug, error, info};

use kusanagi::state::AppState;

/// WebSocket notification message types
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum NotificationMessage {
    #[serde(rename = "alert")]
    Alert {
        severity: String,
        title: String,
        message: String,
        source: String,
        timestamp: String,
    },
    #[serde(rename = "stats_update")]
    StatsUpdate {
        argocd_issues: usize,
        error_pods: usize,
        warning_events: usize,
    },
    #[serde(rename = "connected")]
    Connected { message: String },
    #[serde(rename = "heartbeat")]
    Heartbeat { timestamp: String },
    #[serde(rename = "mqtt_message")]
    MqttMessage {
        topic: String,
        payload: String,
        timestamp: String,
    },
}

/// WebSocket upgrade handler
pub async fn ws_notifications_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle the WebSocket connection
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    info!("WebSocket client connected");

    // Send welcome message
    let welcome = NotificationMessage::Connected {
        message: "Connected to Kusanagi notifications".to_string(),
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        if socket.send(Message::Text(json)).await.is_err() {
            error!("Failed to send welcome message");
            return;
        }
    }

    // Create channel for internal communication
    let (tx, mut rx) = tokio::sync::mpsc::channel::<NotificationMessage>(100);

    // Spawn heartbeat task
    let heartbeat_tx = tx.clone();
    let heartbeat_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            let hb = NotificationMessage::Heartbeat {
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            if heartbeat_tx.send(hb).await.is_err() {
                break;
            }
        }
    });

    // Spawn alert checking task
    let alerts_tx = tx.clone();
    let state_clone = state.clone();
    let _alerts_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if let Some(notification) = check_for_new_alerts(&state_clone).await {
                if alerts_tx.send(notification).await.is_err() {
                    break;
                }
            }
        }
    });

    // Main message loop
    loop {
        tokio::select! {
            // Handle messages from client
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        debug!("Received WebSocket text: {}", text);
                        handle_client_message(&text, &tx, &state).await;
                    }
                    Ok(Message::Binary(_)) => {
                        debug!("Received WebSocket binary message");
                    }
                    Ok(Message::Ping(ping)) => {
                        if socket.send(Message::Pong(ping)).await.is_err() {
                            break;
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        debug!("Received WebSocket pong");
                    }
                    Ok(Message::Close(_)) => {
                        info!("WebSocket client sent close frame");
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
            // Handle internal notifications
            Some(notification) = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&notification) {
                    if socket.send(Message::Text(json)).await.is_err() {
                        break;
                    }
                }
            }
            else => break,
        }
    }

    // Cleanup
    heartbeat_handle.abort();
    info!("WebSocket client disconnected");
}

/// Handle client messages
async fn handle_client_message(
    text: &str,
    tx: &tokio::sync::mpsc::Sender<NotificationMessage>,
    _state: &AppState,
) {
    match text.trim() {
        "ping" => {
            let hb = NotificationMessage::Heartbeat {
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            let _ = tx.send(hb).await;
        }
        "stats" => {
            // Request immediate stats update
            if let Some(stats) = get_current_stats(_state).await {
                let _ = tx.send(stats).await;
            }
        }
        _ => {
            debug!("Unknown WebSocket command: {}", text);
        }
    }
}

/// Check for new alerts that should be sent to clients
async fn check_for_new_alerts(_state: &AppState) -> Option<NotificationMessage> {
    // Get current stats and check for critical issues
    let mut alerts = Vec::new();

    // Check ArgoCD status via cache or API
    // Simplified version - in production, use the actual services
    alerts.push(NotificationMessage::Alert {
        severity: "info".to_string(),
        title: "WebSocket Active".to_string(),
        message: "Notification system is running".to_string(),
        source: "websocket".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    });

    // Return first alert if any
    alerts.into_iter().next()
}

/// Get current cluster stats for WebSocket update
async fn get_current_stats(_state: &AppState) -> Option<NotificationMessage> {
    Some(NotificationMessage::StatsUpdate {
        argocd_issues: 0,
        error_pods: 0,
        warning_events: 0,
    })
}
