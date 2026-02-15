use axum::response::IntoResponse;
use serde_json::json;

use crate::interfaces::http::response::api_success;

pub async fn database_health_handler() -> impl IntoResponse {
    // Mock response for now, assuming SQLite/embedded is healthy if app is running
    api_success(json!({
        "status": "Healthy",
        "latency_ms": 0,
        "version": env!("CARGO_PKG_VERSION")
    }))
}
