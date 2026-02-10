//! Health check handler

use axum::response::IntoResponse;

/// Simple health check endpoint
pub async fn health_check() -> impl IntoResponse {
    "OK"
}
