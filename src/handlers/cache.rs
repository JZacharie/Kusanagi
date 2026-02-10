//! Cache handlers

use axum::response::IntoResponse;
use axum::Json;

/// Cache stats endpoint
pub async fn cache_stats() -> impl IntoResponse {
    Json(serde_json::json!({
        "k8s": {
            "entries": 0,
            "expired": 0,
            "memory_bytes": 0
        },
        "argocd": {
            "entries": 0,
            "expired": 0,
            "memory_bytes": 0
        },
        "general": {
            "entries": 0,
            "expired": 0,
            "memory_bytes": 0
        }
    }))
}
