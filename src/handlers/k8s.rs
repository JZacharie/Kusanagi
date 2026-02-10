//! Kubernetes handlers

use axum::response::IntoResponse;
use axum::Json;

/// Cluster overview endpoint
pub async fn cluster_overview() -> impl IntoResponse {
    Json(serde_json::json!({
        "nodes": 0,
        "pods": 0,
        "services": 0
    }))
}

/// Nodes status endpoint
pub async fn nodes_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "ready": 0,
        "not_ready": 0
    }))
}

/// Pods status endpoint
pub async fn pods_status() -> impl IntoResponse {
    Json(serde_json::json!({
        "running": 0,
        "pending": 0,
        "failed": 0
    }))
}
