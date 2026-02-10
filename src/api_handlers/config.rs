//! Configuration handler

use axum::{extract::State, response::IntoResponse, Json};
use tracing::error;

use kusanagi::state::AppState;

/// Get public configuration (safe values only)
pub async fn get_config(State(_state): State<AppState>) -> impl IntoResponse {
    // Read config from file
    let config = std::fs::read_to_string("kusanagi.toml")
        .or_else(|_| std::fs::read_to_string("/app/kusanagi.toml"))
        .or_else(|_| std::fs::read_to_string("kusanagi.example.toml"));

    match config {
        Ok(content) => {
            // Parse and filter to only return safe values
            match content.parse::<toml::Value>() {
                Ok(value) => {
                    let safe_config = serde_json::json!({
                        "app_name": value.get("app").and_then(|a| a.get("name")).and_then(|n| n.as_str()),
                        "version": value.get("app").and_then(|a| a.get("version")).and_then(|v| v.as_str()),
                        "features": {
                            "cache_enabled": true,
                            "metrics_enabled": true,
                            "websocket_enabled": true
                        }
                    });
                    Json(safe_config).into_response()
                }
                Err(_) => Json(serde_json::json!({
                    "app_name": "Kusanagi",
                    "version": "1.0.0"
                }))
                .into_response(),
            }
        }
        Err(e) => {
            error!("Failed to read config: {}", e);
            Json(serde_json::json!({
                "app_name": "Kusanagi",
                "error": "Configuration not found"
            }))
            .into_response()
        }
    }
}
