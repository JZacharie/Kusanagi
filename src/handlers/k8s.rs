//! Kubernetes handlers

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;

use crate::domain::services::kubernetes_service;
use crate::state::AppState;

/// Cluster overview endpoint
pub async fn cluster_overview() -> impl IntoResponse {
    match kubernetes_service::get_cluster_overview().await {
        Ok(data) => Json(data).into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "nodes": 0,
            "pods": 0,
            "services": 0
        }))
        .into_response(),
    }
}

/// Nodes status endpoint
pub async fn nodes_status() -> impl IntoResponse {
    match kubernetes_service::get_nodes_status().await {
        Ok(data) => Json(data).into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "ready": 0,
            "not_ready": 0
        }))
        .into_response(),
    }
}

/// Pods status endpoint
pub async fn pods_status() -> impl IntoResponse {
    match kubernetes_service::get_pods_status().await {
        Ok(data) => Json(data).into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "running": 0,
            "pending": 0,
            "failed": 0
        }))
        .into_response(),
    }
}

/// Storage endpoint
pub async fn storage(State(state): State<AppState>) -> impl IntoResponse {
    match kubernetes_service::get_storage(&state.http_client).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "pvc_count": 0,
            "pvc_total_capacity": "0 B",
            "pvcs": []
        }))
        .into_response(),
    }
}

/// Ingress endpoint
pub async fn ingress() -> impl IntoResponse {
    use crate::domain::services::kubernetes_service::get_ingress;

    match get_ingress().await {
        Ok(ingresses) => Json(serde_json::json!({
            "status": "success",
            "count": ingresses.as_array().map(|a| a.len()).unwrap_or(0),
            "data": ingresses
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "data": []
        }))
        .into_response(),
    }
}

/// Services endpoint
pub async fn services() -> impl IntoResponse {
    use crate::domain::services::kubernetes_service::get_services;

    match get_services().await {
        Ok(services) => Json(serde_json::json!({
            "status": "success",
            "count": services.as_array().map(|a| a.len()).unwrap_or(0),
            "data": services
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "data": []
        }))
        .into_response(),
    }
}

/// ArgoCD status endpoint
pub async fn argocd_status() -> impl IntoResponse {
    use crate::domain::services::argocd_service::get_argocd_status;

    match get_argocd_status().await {
        Ok(status) => Json(serde_json::json!({
            "status": "success",
            "data": status
        }))
        .into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "data": {
                "applications": [],
                "total": 0,
                "healthy": 0,
                "synced": 0
            }
        }))
        .into_response(),
    }
}
