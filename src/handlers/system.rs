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
