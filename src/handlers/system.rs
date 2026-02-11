//! System handlers

use axum::response::IntoResponse;
use axum::Json;

/// System status endpoint
pub async fn system_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "operational",
        "cpu_load": 0.0,
        "memory_usage": 0
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
