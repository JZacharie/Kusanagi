//! Configuration handler

use axum::{extract::State, response::IntoResponse, Json};

use kusanagi::state::AppState;

/// Get public configuration (safe values only)
pub async fn get_config(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "app_name": "Kusanagi",
        "version": env!("CARGO_PKG_VERSION"),
        "features": {
            "cache_enabled": true,
            "metrics_enabled": true,
            "websocket_enabled": true
        }
    }))
}
