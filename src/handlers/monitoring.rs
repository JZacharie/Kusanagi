//! Monitoring handlers

use axum::response::IntoResponse;
use axum::Json;

/// Alerts endpoint
pub async fn alerts() -> impl IntoResponse {
    Json(serde_json::json!({
        "alerts": [],
        "total": 0
    }))
}

/// Quotas endpoint
pub async fn quotas() -> impl IntoResponse {
    Json(serde_json::json!({
        "used": 0,
        "total": 100
    }))
}
