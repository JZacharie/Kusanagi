use axum::{response::IntoResponse, Json};
use serde_json::json;

/// Streaming data endpoint
pub async fn streaming() -> impl IntoResponse {
    match crate::domain::services::streaming_service::get_streaming_data().await {
        Ok(data) => Json(data).into_response(),
        Err(e) => Json(json!({
            "status": "error",
            "message": e,
            "items": []
        }))
        .into_response(),
    }
}

/// Force refresh streaming data
pub async fn streaming_refresh() -> impl IntoResponse {
    match crate::domain::services::streaming_service::force_refresh().await {
        Ok(data) => Json(json!({
            "status": "success",
            "message": "Streaming data refreshed",
            "items": data["items"],
            "cached_at": data["cached_at"]
        })).into_response(),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        })).into_response(),
    }
}
