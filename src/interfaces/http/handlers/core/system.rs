use axum::{response::IntoResponse, Json};
use serde_json::json;

use crate::domain::services::system_service::SystemService;

/// System status endpoint
pub async fn system_status() -> impl IntoResponse {
    let status = SystemService::get_status();
    Json(status)
}

/// System logs endpoint
pub async fn system_logs() -> impl IntoResponse {
    match SystemService::get_logs().await {
        Ok(logs) => logs.into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to fetch logs: {}", e),
        )
            .into_response(),
    }
}

/// News endpoint
pub async fn news() -> impl IntoResponse {
    match crate::domain::services::news_service::get_news().await {
        Ok(news) => Json(news).into_response(),
        Err(e) => Json(json!({
            "status": "error",
            "message": e,
            "items": []
        }))
        .into_response(),
    }
}
