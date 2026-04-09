use crate::domain::services::mcp_service::McpService;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use tracing::error;

/// Get Trivy vulnerabilities via MCP
/// GET /api/security/vulnerabilities
pub async fn mcp_vulnerabilities_handler(State(state): State<AppState>) -> impl IntoResponse {
    let service = McpService::new(
        state.kube_client.as_ref().map(|c| c.as_ref().clone()),
        state.k8s_cache.clone(),
    );

    match service.get_trivy_vulnerabilities().await {
        Ok(summary) => {
            let total = summary.critical + summary.high + summary.medium + summary.low;
            api_success(json!({
                "critical": summary.critical,
                "high": summary.high,
                "medium": summary.medium,
                "low": summary.low,
                "total": total,
                "images": summary.images
            }))
        }
        Err(e) => {
            tracing::warn!(
                "MCP vulnerabilities failed, falling back to SecurityRepository: {}",
                e
            );
            match state.security_use_case.get_summary().await {
                Ok(summary) => api_success(json!({
                    "critical": summary.critical_count,
                    "high": summary.high_count,
                    "medium": summary.medium_count,
                    "low": summary.low_count,
                    "total": summary.total_vulnerabilities,
                    "images": []
                })),
                Err(err) => {
                    error!("Security fallback also failed: {}", err);
                    api_error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                }
            }
        }
    }
}

/// Get Cilium policies via MCP
/// GET /api/security/policies
pub async fn mcp_policies_handler(State(state): State<AppState>) -> impl IntoResponse {
    let service = McpService::new(
        state.kube_client.as_ref().map(|c| c.as_ref().clone()),
        state.k8s_cache.clone(),
    );

    match service.get_cilium_policies(None).await {
        Ok(summary) => api_success(json!(summary)),
        Err(e) => {
            error!("Failed to get policies: {}", e);
            api_error(StatusCode::INTERNAL_SERVER_ERROR, e)
        }
    }
}

/// Get Kyverno policy violations
/// GET /api/security/policies/violations
pub async fn mcp_policy_violations_handler(State(state): State<AppState>) -> impl IntoResponse {
    let service = McpService::new(
        state.kube_client.as_ref().map(|c| c.as_ref().clone()),
        state.k8s_cache.clone(),
    );

    match service.get_policy_violations().await {
        Ok(overview) => api_success(json!(overview)),
        Err(e) => {
            error!("Failed to get policy violations: {}", e);
            api_error(StatusCode::INTERNAL_SERVER_ERROR, e)
        }
    }
}

/// Get Fence status
/// GET /api/security/fence
pub async fn mcp_fence_handler(State(state): State<AppState>) -> impl IntoResponse {
    let service = McpService::new(
        state.kube_client.as_ref().map(|c| c.as_ref().clone()),
        state.k8s_cache.clone(),
    );

    match service.get_fence_status().await {
        Ok(status) => api_success(status),
        Err(e) => {
            error!("Failed to get fence status: {}", e);
            api_error(StatusCode::INTERNAL_SERVER_ERROR, e)
        }
    }
}

/// Query OpenObserve logs via MCP
/// GET /api/monitoring/logs?query=...&limit=...
pub async fn mcp_openobserve_logs_handler(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let service = McpService::new(
        state.kube_client.as_ref().map(|c| c.as_ref().clone()),
        state.k8s_cache.clone(),
    );

    let query = params.get("query").map(|s| s.as_str()).unwrap_or("*");
    let limit = params
        .get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(100);

    match service.query_logs(query, Some(limit)).await {
        Ok(result) => api_success(json!(result)),
        Err(e) => {
            error!("Failed to query logs: {}", e);
            api_error(StatusCode::INTERNAL_SERVER_ERROR, e)
        }
    }
}

/// Get Netbox inventory via MCP
/// GET /api/monitoring/netbox
pub async fn mcp_netbox_handler(State(state): State<AppState>) -> impl IntoResponse {
    let service = McpService::new(
        state.kube_client.as_ref().map(|c| c.as_ref().clone()),
        state.k8s_cache.clone(),
    );

    match service.get_netbox_inventory().await {
        Ok(inventory) => api_success(inventory),
        Err(e) => {
            error!("Failed to get Netbox inventory: {}", e);
            api_error(StatusCode::INTERNAL_SERVER_ERROR, e)
        }
    }
}
