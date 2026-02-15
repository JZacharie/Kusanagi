//! Cilium HTTP Handlers
//! Axum handlers for Cilium network visualization

use crate::domain::services::cilium_service::CiliumService;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::error;

#[derive(Deserialize)]
pub struct CiliumParams {
    pub namespace: Option<String>,
    pub limit: Option<usize>,
}

/// Get Hubble flows
/// GET /api/cilium/flows
#[utoipa::path(
    get,
    path = "/api/cilium/flows",
    responses(
        (status = 200, description = "List of network flows"),
        (status = 503, description = "Kubernetes client not available")
    ),
    params(
        ("namespace" = Option<String>, Query, description = "Namespace to filter by"),
        ("limit" = Option<usize>, Query, description = "Limit number of flows")
    ),
    tag = "cilium"
)]
pub async fn get_flows_handler(
    State(state): State<AppState>,
    Query(params): Query<CiliumParams>,
) -> impl IntoResponse {
    let client = match &state.kube_client {
        Some(c) => c.as_ref().clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Kubernetes client not available" })),
            );
        }
    };

    let service = CiliumService::new(client, state.cilium_cache.clone());
    let limit = params.limit.unwrap_or(100);

    match service
        .get_hubble_flows(params.namespace.as_deref(), limit)
        .await
    {
        Ok(response) => (
            StatusCode::OK,
            Json(serde_json::to_value(response).unwrap_or_default()),
        ),
        Err(e) => {
            error!("Failed to get flows: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
        }
    }
}

/// Get Flow Matrix (Aggregated)
/// GET /api/cilium/matrix
#[utoipa::path(
    get,
    path = "/api/cilium/matrix",
    responses(
        (status = 200, description = "Network flow matrix"),
        (status = 503, description = "Kubernetes client not available")
    ),
    params(
        ("namespace" = Option<String>, Query, description = "Namespace to filter by")
    ),
    tag = "cilium"
)]
pub async fn get_matrix_handler(
    State(state): State<AppState>,
    Query(params): Query<CiliumParams>,
) -> impl IntoResponse {
    let client = match &state.kube_client {
        Some(c) => c.as_ref().clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Kubernetes client not available" })),
            );
        }
    };

    let service = CiliumService::new(client, state.cilium_cache.clone());

    match service.get_flow_matrix(params.namespace.as_deref()).await {
        Ok(matrix) => (
            StatusCode::OK,
            Json(serde_json::to_value(matrix).unwrap_or_default()),
        ),
        Err(e) => {
            error!("Failed to get flow matrix: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
        }
    }
}

/// Get Bandwidth Metrics
/// GET /api/cilium/metrics
#[utoipa::path(
    get,
    path = "/api/cilium/metrics",
    responses(
        (status = 200, description = "Cilium bandwidth metrics"),
        (status = 503, description = "Kubernetes client not available")
    ),
    params(
        ("namespace" = Option<String>, Query, description = "Namespace to filter by")
    ),
    tag = "cilium"
)]
pub async fn get_metrics_handler(
    State(state): State<AppState>,
    Query(params): Query<CiliumParams>,
) -> impl IntoResponse {
    let client = match &state.kube_client {
        Some(c) => c.as_ref().clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Kubernetes client not available" })),
            );
        }
    };

    let service = CiliumService::new(client, state.cilium_cache.clone());

    match service
        .get_bandwidth_metrics(params.namespace.as_deref())
        .await
    {
        Ok(metrics) => (
            StatusCode::OK,
            Json(serde_json::to_value(metrics).unwrap_or_default()),
        ),
        Err(e) => {
            error!("Failed to get bandwidth metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
        }
    }
}

/// Get Network Anomalies
/// GET /api/cilium/anomalies
#[utoipa::path(
    get,
    path = "/api/cilium/anomalies",
    responses(
        (status = 200, description = "Detected network anomalies"),
        (status = 503, description = "Kubernetes client not available")
    ),
    params(
        ("namespace" = Option<String>, Query, description = "Namespace to filter by")
    ),
    tag = "cilium"
)]
pub async fn get_anomalies_handler(
    State(state): State<AppState>,
    Query(params): Query<CiliumParams>,
) -> impl IntoResponse {
    let client = match &state.kube_client {
        Some(c) => c.as_ref().clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Kubernetes client not available" })),
            );
        }
    };

    let service = CiliumService::new(client, state.cilium_cache.clone());

    match service.detect_anomalies(params.namespace.as_deref()).await {
        Ok(anomalies) => (
            StatusCode::OK,
            Json(serde_json::to_value(anomalies).unwrap_or_default()),
        ),
        Err(e) => {
            error!("Failed to detect anomalies: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
        }
    }
}

/// Get Namespaces
/// GET /api/cilium/namespaces
#[utoipa::path(
    get,
    path = "/api/cilium/namespaces",
    responses(
        (status = 200, description = "List of namespaces with Cilium enabled"),
        (status = 503, description = "Kubernetes client not available")
    ),
    tag = "cilium"
)]
pub async fn get_namespaces_handler(State(state): State<AppState>) -> impl IntoResponse {
    let client = match &state.kube_client {
        Some(c) => c.as_ref().clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Kubernetes client not available" })),
            );
        }
    };

    let service = CiliumService::new(client, state.cilium_cache.clone());

    match service.get_namespaces().await {
        Ok(namespaces) => (
            StatusCode::OK,
            Json(serde_json::to_value(namespaces).unwrap_or_default()),
        ),
        Err(e) => {
            error!("Failed to get namespaces: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
        }
    }
}

/// Get Cilium Status
/// GET /api/cilium/status
pub async fn get_cilium_status_handler(State(state): State<AppState>) -> impl IntoResponse {
    let client = match &state.kube_client {
        Some(c) => c.as_ref().clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Kubernetes client not available",
                    "status": "unavailable"
                })),
            );
        }
    };

    let service = CiliumService::new(client, state.cilium_cache.clone());

    match service.get_cilium_status().await {
        Ok(status) => (StatusCode::OK, Json(status)),
        Err(e) => {
            error!("Failed to get Cilium status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": e,
                    "status": "error"
                })),
            )
        }
    }
}
