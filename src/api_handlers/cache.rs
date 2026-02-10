//! Cache stats handler

use axum::{extract::State, response::IntoResponse, Json};

use kusanagi::state::AppState;

/// Get cache statistics
pub async fn cache_stats(State(state): State<AppState>) -> impl IntoResponse {
    let k8s = state.k8s_cache.stats().await;
    let argocd = state.argocd_cache.stats().await;
    let general = state.general_cache.stats().await;

    Json(serde_json::json!({
        "k8s": {
            "entries": k8s.entries,
            "expired": k8s.expired,
            "memory_bytes": k8s.memory_bytes,
            "ttl_seconds": 60
        },
        "argocd": {
            "entries": argocd.entries,
            "expired": argocd.expired,
            "memory_bytes": argocd.memory_bytes,
            "ttl_seconds": 600
        },
        "general": {
            "entries": general.entries,
            "expired": general.expired,
            "memory_bytes": general.memory_bytes,
            "ttl_seconds": 120
        },
        "total": {
            "entries": k8s.entries + argocd.entries + general.entries,
            "expired": k8s.expired + argocd.expired + general.expired,
            "memory_bytes": k8s.memory_bytes + argocd.memory_bytes + general.memory_bytes
        }
    }))
}
