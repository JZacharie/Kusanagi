use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::interfaces::http::response::{api_error, api_success};

/// Streaming data endpoint
pub async fn streaming() -> impl IntoResponse {
    match crate::domain::services::streaming_service::get_streaming_data().await {
        Ok(data) => api_success(data),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Force refresh streaming data
pub async fn streaming_refresh() -> impl IntoResponse {
    match crate::domain::services::streaming_service::force_refresh().await {
        Ok(data) => api_success(json!({
            "message": "Streaming data refreshed",
            "items": data["items"],
            "cached_at": data["cached_at"]
        })),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}
