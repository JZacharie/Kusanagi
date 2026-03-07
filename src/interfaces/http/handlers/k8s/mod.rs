//! Kubernetes handlers

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::domain::services::kubernetes_service;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

/// Cluster overview endpoint
#[utoipa::path(
    get,
    path = "/api/k8s/cluster",
    responses(
        (status = 200, description = "Cluster overview retrieved successfully"),
        (status = 500, description = "Failed to retrieve cluster overview")
    ),
    tag = "kubernetes"
)]
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
#[utoipa::path(
    get,
    path = "/api/k8s/nodes",
    responses(
        (status = 200, description = "Nodes status retrieved successfully"),
        (status = 500, description = "Failed to retrieve nodes status")
    ),
    tag = "kubernetes"
)]
pub async fn nodes_status(State(state): State<AppState>) -> Response {
    match kubernetes_service::get_nodes_status(&state.http_client, &state.k8s_cache).await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Nodes debug/diagnostic endpoint
#[utoipa::path(
    get,
    path = "/api/debug/nodes",
    responses(
        (status = 200, description = "Node debug info retrieved successfully"),
        (status = 500, description = "Failed to retrieve node debug info")
    ),
    tag = "kubernetes"
)]
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
    let metrics_debug = fetch_node_metrics(&state.http_client)
        .await
        .unwrap_or_default();
    let metrics_info: serde_json::Map<String, serde_json::Value> = metrics_debug
        .into_iter()
        .map(|(node, (cpu, mem))| {
            (
                node,
                json!({
                    "cpu_cores": cpu,
                    "memory_bytes": mem
                }),
            )
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
#[utoipa::path(
    get,
    path = "/api/k8s/pods",
    responses(
        (status = 200, description = "Pods status retrieved successfully"),
        (status = 500, description = "Failed to retrieve pods status")
    ),
    tag = "kubernetes"
)]
pub async fn pods_status(State(state): State<AppState>) -> Response {
    match kubernetes_service::get_pods_status(&state.k8s_cache).await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Storage endpoint
#[utoipa::path(
    get,
    path = "/api/storage",
    responses(
        (status = 200, description = "Storage info retrieved successfully"),
        (status = 500, description = "Failed to retrieve storage info")
    ),
    tag = "kubernetes"
)]
pub async fn storage(State(state): State<AppState>) -> Response {
    match kubernetes_service::get_storage(&state.http_client).await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Storage analysis endpoint
#[utoipa::path(
    get,
    path = "/api/storage/analysis",
    responses(
        (status = 200, description = "Storage analysis info retrieved successfully"),
        (status = 500, description = "Failed to retrieve storage analysis info")
    ),
    tag = "kubernetes"
)]
pub async fn storage_analysis(State(state): State<AppState>) -> Response {
    match kubernetes_service::get_storage_analysis(&state.http_client).await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Ingress endpoint
#[utoipa::path(
    get,
    path = "/api/ingress",
    responses(
        (status = 200, description = "Ingresses retrieved successfully"),
        (status = 500, description = "Failed to fetch ingress")
    ),
    tag = "kubernetes"
)]
pub async fn ingress(State(state): State<AppState>) -> Response {
    use crate::domain::services::kubernetes_service::get_ingress;

    match get_ingress(&state.kube_client, &state.k8s_cache).await {
        Ok(ingresses) => api_success(json!(ingresses)),
        Err(_e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch ingress"),
    }
}

/// Services endpoint
#[utoipa::path(
    get,
    path = "/api/services",
    responses(
        (status = 200, description = "Services retrieved successfully"),
        (status = 500, description = "Failed to fetch services")
    ),
    tag = "kubernetes"
)]
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
        (status = 200, description = "ArgoCD status retrieved successfully"),
        (status = 500, description = "Failed to retrieve ArgoCD status")
    ),
    tag = "kubernetes"
)]
pub async fn argocd_status(State(state): State<AppState>) -> Response {
    use crate::domain::services::argocd_service::get_argocd_status;

    match get_argocd_status(&state.k8s_cache).await {
        Ok(status) => api_success(json!(status)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Pod logs endpoint
#[utoipa::path(
    get,
    path = "/api/k8s/pods/{namespace}/{name}/logs",
    params(
        ("namespace" = String, Path, description = "Pod namespace"),
        ("name" = String, Path, description = "Pod name")
    ),
    responses(
        (status = 200, description = "Pod logs retrieved successfully"),
        (status = 404, description = "Failed to fetch logs")
    ),
    tag = "kubernetes"
)]
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
#[utoipa::path(
    post,
    path = "/api/pods/delete-error-pods",
    responses(
        (status = 200, description = "Error pods deleted successfully"),
        (status = 500, description = "Failed to delete error pods")
    ),
    tag = "kubernetes"
)]
pub async fn delete_error_pods_handler(State(state): State<AppState>) -> Response {
    use crate::domain::services::kubernetes_service::delete_error_pods;

    match delete_error_pods(&state.kube_client).await {
        Ok(result) => api_success(json!(result)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct SyncAppRequest {
    pub app_name: String,
}

/// Trigger ArgoCD app sync
#[utoipa::path(
    post,
    path = "/api/argocd/sync",
    request_body = SyncAppRequest,
    responses(
        (status = 200, description = "ArgoCD sync triggered successfully"),
        (status = 500, description = "Failed to trigger sync")
    ),
    tag = "kubernetes"
)]
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
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<DeletePodRequest>,
) -> Response {
    use crate::domain::services::kubernetes_service::force_delete_pod;

    match force_delete_pod(&state.kube_client, &payload.namespace, &payload.pod_name).await {
        Ok(result) => api_success(result),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

#[derive(serde::Deserialize)]
pub struct NamespaceMetricsQuery {
    pub window: Option<String>,
}

/// Get resource usage metrics per namespace
#[utoipa::path(
    get,
    path = "/api/k8s/namespaces/metrics",
    responses(
        (status = 200, description = "Namespace metrics retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "kubernetes"
)]
pub async fn get_namespace_metrics_handler(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<NamespaceMetricsQuery>,
) -> Response {
    use crate::domain::services::kubernetes_service::get_namespace_metrics;

    match get_namespace_metrics(&state.http_client, query.window).await {
        Ok(result) => api_success(result),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}
