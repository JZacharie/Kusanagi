//! Kubernetes handlers

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::domain::services::kubernetes_service;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

/// Cluster overview endpoint
pub async fn cluster_overview(State(state): State<AppState>) -> Response {
    match kubernetes_service::get_cluster_overview(
        &state.http_client,
        &state.kube_client,
        &state.k8s_cache,
    )
    .await
    {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Nodes status endpoint
pub async fn nodes_status(State(state): State<AppState>) -> Response {
    match kubernetes_service::get_nodes_status(&state.http_client, &state.k8s_cache).await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Nodes debug/diagnostic endpoint
pub async fn nodes_debug(State(state): State<AppState>) -> Response {
    use crate::domain::services::kubernetes_service::fetch_node_metrics;
    
    let k8s_nodes_ok = kubernetes_service::get_nodes_status(&state.http_client, &state.k8s_cache)
        .await
        .is_ok();
    let k8s_pods_ok = kubernetes_service::get_pods_status(&state.k8s_cache)
        .await
        .is_ok();

    // Check prometheus connectivity
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });
    let prometheus_ok = state
        .http_client
        .get(format!("{}/-/healthy", prometheus_url))
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    
    // Fetch raw metrics for debugging
    let metrics_debug = fetch_node_metrics(&state.http_client).await.unwrap_or_default();
    let metrics_info: serde_json::Map<String, serde_json::Value> = metrics_debug
        .into_iter()
        .map(|(node, (cpu, mem))| {
            (node, json!({
                "cpu_cores": cpu,
                "memory_bytes": mem
            }))
        })
        .collect();

    api_success(json!({
        "k8s_nodes_ok": k8s_nodes_ok,
        "k8s_pods_ok": k8s_pods_ok,
        "prometheus_ok": prometheus_ok,
        "prometheus_url": prometheus_url,
        "raw_metrics": metrics_info
    }))
}

/// Pods status endpoint
pub async fn pods_status(State(state): State<AppState>) -> Response {
    match kubernetes_service::get_pods_status(&state.k8s_cache).await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Storage endpoint
pub async fn storage(State(state): State<AppState>) -> Response {
    match kubernetes_service::get_storage(&state.http_client).await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Ingress endpoint
pub async fn ingress(State(state): State<AppState>) -> Response {
    use crate::domain::services::kubernetes_service::get_ingress;

    match get_ingress(&state.kube_client, &state.k8s_cache).await {
        Ok(ingresses) => api_success(json!(ingresses)),
        Err(_e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch ingress"),
    }
}

/// Services endpoint
pub async fn services(State(state): State<AppState>) -> Response {
    use crate::domain::services::kubernetes_service::get_services;

    match get_services(&state.kube_client, &state.k8s_cache).await {
        Ok(services) => api_success(json!(services)),
        Err(_e) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to fetch services",
        ),
    }
}

/// ArgoCD status endpoint
#[utoipa::path(
    get,
    path = "/api/argocd/status",
    responses(
        (status = 200, description = "ArgoCD status retrieved successfully", body = Object),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn argocd_status(State(state): State<AppState>) -> Response {
    use crate::domain::services::argocd_service::get_argocd_status;

    match get_argocd_status(&state.k8s_cache).await {
        Ok(status) => api_success(json!(status)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Pod logs endpoint
/// Note: Returns raw text, not JSON envelope, as logs are text-based
pub async fn pod_logs(Path((namespace, name)): Path<(String, String)>) -> impl IntoResponse {
    use crate::domain::services::kubernetes_service::get_pod_logs;

    match get_pod_logs(&namespace, &name).await {
        Ok(logs) => logs.into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            format!("Failed to fetch logs: {}", e),
        )
            .into_response(),
    }
}

/// Delete pods in error state
pub async fn delete_error_pods_handler() -> Response {
    use crate::domain::services::kubernetes_service::delete_error_pods;

    match delete_error_pods().await {
        Ok(result) => api_success(json!(result)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}
#[derive(serde::Deserialize)]
pub struct SyncAppRequest {
    pub app_name: String,
}

/// Trigger ArgoCD app sync
pub async fn argocd_sync(
    State(_state): State<AppState>,
    axum::Json(payload): axum::Json<SyncAppRequest>,
) -> Response {
    use crate::domain::services::argocd_service::sync_app;

    match sync_app(&payload.app_name).await {
        Ok(msg) => api_success(json!({ "success": true, "message": msg })),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct DeletePodRequest {
    pub namespace: String,
    pub pod_name: String,
}

/// Force delete a pod
#[utoipa::path(
    post,
    path = "/api/pods/force-delete",
    request_body = DeletePodRequest,
    responses(
        (status = 200, description = "Pod deleted successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "kubernetes"
)]
pub async fn force_delete_pod_handler(
    State(_state): State<AppState>,
    axum::Json(payload): axum::Json<DeletePodRequest>,
) -> Response {
    use crate::domain::services::kubernetes_service::force_delete_pod;

    match force_delete_pod(&payload.namespace, &payload.pod_name).await {
        Ok(result) => api_success(result),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}
