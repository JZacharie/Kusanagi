use super::{kubernetes_service, monitoring_service};
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct UnifiedEvent {
    pub source: String,     // "alertmanager" or "kubernetes"
    pub event_type: String, // "alert" or "event"
    pub severity: String,   // "critical", "warning", "info", "normal"
    pub name: String,       // alertname or Reason
    pub namespace: String,
    pub message: String,   // summary or message
    pub timestamp: String, // ISO8601
    pub details: Value,    // Original payload
}

pub async fn get_fusion_events() -> Result<Vec<UnifiedEvent>, String> {
    let mut unified_events = Vec::new();

    // 1. Fetch Alerts
    match monitoring_service::get_alerts().await {
        Ok(alerts_value) => {
            if let Some(alerts) = alerts_value.as_array() {
                for alert in alerts {
                    let severity = alert
                        .get("severity")
                        .and_then(|s| s.as_str())
                        .unwrap_or("info")
                        .to_string();

                    let name = alert
                        .get("alertname")
                        .and_then(|s| s.as_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let summary = alert
                        .get("summary")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();

                    let timestamp = chrono::Utc::now().to_rfc3339(); // Alerts don't always have timestamp in the simplified view, use now or try to extract

                    unified_events.push(UnifiedEvent {
                        source: "alertmanager".to_string(),
                        event_type: "alert".to_string(),
                        severity,
                        name,
                        namespace: "monitoring".to_string(), // Default for alerts if not specified
                        message: summary,
                        timestamp,
                        details: alert.clone(),
                    });
                }
            }
        }
        Err(e) => tracing::error!("Failed to fetch alerts for fusion: {}", e),
    }

    // 2. Fetch Kubernetes Events
    match kubernetes_service::get_events().await {
        Ok(events_value) => {
            if let Some(events) = events_value.as_array() {
                for event in events {
                    let type_ = event
                        .get("type")
                        .and_then(|s| s.as_str())
                        .unwrap_or("Normal");

                    let severity = if type_ == "Warning" {
                        "warning".to_string()
                    } else {
                        "info".to_string()
                    };

                    let reason = event
                        .get("reason")
                        .and_then(|s| s.as_str())
                        .unwrap_or("Unknown")
                        .to_string();

                    let message = event
                        .get("message")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();

                    let namespace = event
                        .get("namespace")
                        .and_then(|s| s.as_str())
                        .unwrap_or("default")
                        .to_string();

                    let timestamp = event
                        .get("timestamp")
                        .and_then(|t| t.as_str())
                        .unwrap_or("")
                        .to_string();

                    // Convert timestamp to string if it's an object (k8s Time)
                    let timestamp_str = if timestamp.is_empty() {
                        chrono::Utc::now().to_rfc3339()
                    } else {
                        timestamp
                    };

                    unified_events.push(UnifiedEvent {
                        source: "kubernetes".to_string(),
                        event_type: "event".to_string(),
                        severity,
                        name: reason,
                        namespace,
                        message,
                        timestamp: timestamp_str,
                        details: event.clone(),
                    });
                }
            }
        }
        Err(e) => tracing::error!("Failed to fetch k8s events for fusion: {}", e),
    }

    // 3. Sort by timestamp descending
    unified_events.sort_by(|a, b| {
        let time_a = DateTime::parse_from_rfc3339(&a.timestamp).unwrap_or_default();
        let time_b = DateTime::parse_from_rfc3339(&b.timestamp).unwrap_or_default();
        time_b.cmp(&time_a)
    });

    Ok(unified_events)
}

// Axum Handler
use axum::{response::IntoResponse, Json};

pub async fn fusion_handler() -> impl IntoResponse {
    match get_fusion_events().await {
        Ok(events) => Json(serde_json::json!({
            "status": "success",
            "count": events.len(),
            "data": events
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e
        }))
        .into_response(),
    }
}
