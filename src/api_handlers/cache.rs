//! Cache stats handler

use axum::{extract::State, response::IntoResponse, Json};

use kusanagi::state::AppState;

/// Get cache statistics
pub async fn cache_stats(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "k8s": {
            "entries": 0,
            "expired": 0,
            "memory_bytes": 0,
            "ttl_seconds": 60
        },
        "argocd": {
            "entries": 0,
            "expired": 0,
            "memory_bytes": 0,
            "ttl_seconds": 600
        },
        "general": {
            "entries": 0,
            "expired": 0,
            "memory_bytes": 0,
            "ttl_seconds": 120
        }
    }))
}
