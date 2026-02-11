use axum::{response::IntoResponse, Json};
use serde_json::json;

pub async fn database_health_handler() -> impl IntoResponse {
    // Mock response for now, assuming SQLite/embedded is healthy if app is running
    Json(json!({
        "status": "Healthy",
        "latency_ms": 0,
        "version": env!("CARGO_PKG_VERSION")
    }))
}
