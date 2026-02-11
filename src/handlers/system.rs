//! System handlers

use axum::response::IntoResponse;
use axum::Json;

/// System status endpoint
pub async fn system_status() -> impl IntoResponse {
    // Mock data for now, ideally retrieved from system metrics or app state
    Json(serde_json::json!({
        "status": "operational",
        "uptime_secs": 3600, // Mock 1 hour
        "cpu_usage": 15.5,
        "memory_usage_mb": 256.0,
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// System logs endpoint
pub async fn system_logs() -> impl IntoResponse {
    "System logs not available"
}

/// News endpoint
pub async fn news() -> impl IntoResponse {
    match crate::domain::services::news_service::get_news().await {
        Ok(news) => Json(news).into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "items": []
        }))
        .into_response(),
    }
}
