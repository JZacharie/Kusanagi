//! Kubernetes handlers

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::domain::services::kubernetes_service;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

/// Cluster overview endpoint
pub async fn cluster_overview(State(state): State<AppState>) -> impl IntoResponse {
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
pub async fn nodes_status(State(state): State<AppState>) -> impl IntoResponse {
    match kubernetes_service::get_nodes_status(&state.http_client).await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Pods status endpoint
pub async fn pods_status() -> impl IntoResponse {
    match kubernetes_service::get_pods_status().await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Storage endpoint
pub async fn storage(State(state): State<AppState>) -> impl IntoResponse {
    match kubernetes_service::get_storage(&state.http_client).await {
        Ok(data) => api_success(json!(data)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Ingress endpoint
pub async fn ingress(State(state): State<AppState>) -> impl IntoResponse {
    use crate::domain::services::kubernetes_service::get_ingress;

    match get_ingress(&state.kube_client, &state.k8s_cache).await {
        Ok(ingresses) => api_success(json!(ingresses)),
        Err(_e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch ingress"),
    }
}

/// Services endpoint
pub async fn services(State(state): State<AppState>) -> impl IntoResponse {
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
pub async fn argocd_status() -> impl IntoResponse {
    use crate::domain::services::argocd_service::get_argocd_status;

    match get_argocd_status().await {
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
pub async fn delete_error_pods_handler() -> impl IntoResponse {
    use crate::domain::services::kubernetes_service::delete_error_pods;

    match delete_error_pods().await {
        Ok(result) => api_success(json!(result)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}
