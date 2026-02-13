//! Kubernetes handlers

use axum::extract::{Path, State};
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
            "services": 0,
            "pods_running": 0,
            "nodes_ready": 0
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
            "ready_nodes": 0,
            "not_ready_nodes": 0,
            "total_nodes": 0
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
            "running_pods": 0,
            "pending_pods": 0,
            "error_pods": 0,
            "total_pods": 0
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
        Ok(ingresses) => Json(ingresses).into_response(),
        Err(_e) => Json(serde_json::json!([])).into_response(),
    }
}

/// Services endpoint
pub async fn services() -> impl IntoResponse {
    use crate::domain::services::kubernetes_service::get_services;

    match get_services().await {
        Ok(services) => Json(services).into_response(),
        Err(_e) => Json(serde_json::json!([])).into_response(),
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

/// Pod logs endpoint
pub async fn pod_logs(Path((namespace, name)): Path<(String, String)>) -> impl IntoResponse {
    use crate::domain::services::kubernetes_service::get_pod_logs;

    match get_pod_logs(&namespace, &name).await {
        Ok(logs) => logs.into_response(),
        Err(e) => (
            axum::http::StatusCode::NOT_FOUND,
            format!("Failed to fetch logs: {}", e),
        )
            .into_response(),
    }
}

/// Delete pods in error state
pub async fn delete_error_pods_handler() -> impl IntoResponse {
    use crate::domain::services::kubernetes_service::delete_error_pods;

    match delete_error_pods().await {
        Ok(result) => Json(result).into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "success": false
        }))
        .into_response(),
    }
}
