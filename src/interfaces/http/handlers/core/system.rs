use axum::{response::IntoResponse, Json};
use serde_json::json;

use crate::domain::services::system_service::{SystemService, SystemStatus};
use crate::interfaces::http::response::{api_error, api_success};

/// System status endpoint
#[utoipa::path(
    get,
    path = "/api/system/status",
    responses(
        (status = 200, description = "System status", body = SystemStatus)
    )
)]
pub async fn system_status() -> impl IntoResponse {
    let status = SystemService::get_status();
    api_success(json!(status))
}

/// System logs endpoint
#[utoipa::path(
    get,
    path = "/api/system/logs",
    responses(
        (status = 200, description = "System logs", body = String),
        (status = 500, description = "Internal Server Error")
    )
)]
pub async fn system_logs() -> impl IntoResponse {
    match SystemService::get_logs().await {
        Ok(logs) => api_success(json!({ "logs": logs })),
        Err(e) => api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
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

/// Force refresh news cache
#[utoipa::path(
    post,
    path = "/api/news/refresh",
    responses(
        (status = 200, description = "News refreshed successfully", body = serde_json::Value),
        (status = 500, description = "Failed to refresh news")
    )
)]
pub async fn news_refresh() -> impl IntoResponse {
    match crate::domain::services::news_service::force_refresh().await {
        Ok(news) => Json(json!({
            "status": "success",
            "message": "News refresh and translation started in background or completed",
            "items": news["items"],
            "cached_at": news["cached_at"],
            "sources": news["sources"]
        })).into_response(),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        })).into_response(),
    }
}
