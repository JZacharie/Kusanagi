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
        "antigravity_percentage": 15,
        "notebooklm_percentage": 30,
        "storage_used_gb": 45.5,
        "storage_total_gb": 100.0,
        "last_updated": chrono::Utc::now().to_rfc3339()
    }))
}
