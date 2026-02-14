//! Health check handler

use axum::response::IntoResponse;
use axum::Json;

/// Simple health check endpoint
pub async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
