//! MCP HTTP Handlers
//! Axum handlers for Model Context Protocol endpoints

use crate::domain::services::mcp_service::McpService;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
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
            Json(serde_json::json!({
                "critical": summary.critical,
                "high": summary.high,
                "medium": summary.medium,
                "low": summary.low,
                "total": total,
                "images": summary.images
            }))
            .into_response()
        }
        Err(e) => {
            tracing::warn!(
                "MCP vulnerabilities failed, falling back to SecurityRepository: {}",
                e
            );
            match state.security_use_case.get_summary().await {
                Ok(summary) => Json(serde_json::json!({
                    "critical": summary.critical_count,
                    "high": summary.high_count,
                    "medium": summary.medium_count,
                    "low": summary.low_count,
                    "total": summary.total_vulnerabilities,
                    "images": []
                }))
                .into_response(),
                Err(err) => {
                    error!("Security fallback also failed: {}", err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({ "error": err.to_string() })),
                    )
                        .into_response()
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
        Ok(summary) => (
            StatusCode::OK,
            Json(serde_json::to_value(summary).unwrap_or_default()),
        ),
        Err(e) => {
            error!("Failed to get policies: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
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
        Ok(overview) => (
            StatusCode::OK,
            Json(serde_json::to_value(overview).unwrap_or_default()),
        ),
        Err(e) => {
            error!("Failed to get policy violations: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
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
        Ok(status) => (StatusCode::OK, Json(status)),
        Err(e) => {
            error!("Failed to get fence status: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": e })),
            )
        }
    }
}
