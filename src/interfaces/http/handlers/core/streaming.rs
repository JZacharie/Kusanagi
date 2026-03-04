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

/// Serve a cached movie poster from S3
pub async fn get_poster(
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> impl IntoResponse {
    match crate::domain::services::streaming_service::get_poster_data(&hash).await {
        Ok((bytes, content_type)) => {
            axum::response::Response::builder()
                .header(axum::http::header::CONTENT_TYPE, content_type)
                .header(axum::http::header::CACHE_CONTROL, "public, max-age=31536000") // 1 year
                .body(axum::body::Body::from(bytes))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
        Err(e) => api_error(StatusCode::NOT_FOUND, e),
    }
}
