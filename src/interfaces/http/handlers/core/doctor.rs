//! Doctor HTTP Handlers
//! Axum handlers for system diagnostic endpoints

use crate::domain::entities::diagnostic::QuickHealthResponse;
use crate::domain::services::diagnostic_service::DiagnosticService;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::time::Instant;
use tracing::error;

/// Full diagnostic check endpoint
/// GET /api/doctor
pub async fn doctor_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Check if kube client is available
    let kube_client = match &state.kube_client {
        Some(client) => client.as_ref().clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({
                    "error": "Kubernetes client not available",
                    "message": "Cannot run diagnostics without Kubernetes connection"
                })),
            );
        }
    };

    let service = DiagnosticService::new(kube_client);

    match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        service.run_full_diagnostics(),
    )
    .await
    {
        Ok(report) => {
            let status_code = match report.overall_status {
                crate::domain::entities::diagnostic::CheckStatus::Ok => StatusCode::OK,
                crate::domain::entities::diagnostic::CheckStatus::Warning => StatusCode::OK,
                crate::domain::entities::diagnostic::CheckStatus::Error => {
                    StatusCode::SERVICE_UNAVAILABLE
                }
                crate::domain::entities::diagnostic::CheckStatus::Skipped => StatusCode::OK,
            };
            (
                status_code,
                Json(serde_json::to_value(report).unwrap_or_default()),
            )
        }
        Err(_) => {
            error!("Diagnostic check timed out after 30 seconds");
            (
                StatusCode::REQUEST_TIMEOUT,
                Json(serde_json::json!({
                    "error": "Diagnostic check timed out",
                    "message": "System diagnostics took too long to complete"
                })),
            )
        }
    }
}

/// Quick diagnostic check endpoint (essential checks only)
/// GET /api/doctor/quick
pub async fn doctor_quick_handler(State(state): State<AppState>) -> impl IntoResponse {
    let start = Instant::now();

    // Check if kube client is available
    let kube_client = match &state.kube_client {
        Some(client) => client.as_ref().clone(),
        None => {
            let response = QuickHealthResponse {
                healthy: false,
                kubernetes: false,
                permissions: false,
                duration_ms: start.elapsed().as_millis() as u64,
            };
            return (StatusCode::SERVICE_UNAVAILABLE, Json(response));
        }
    };

    let service = DiagnosticService::new(kube_client);

    match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        service.run_quick_diagnostics(),
    )
    .await
    {
        Ok(report) => {
            // Extract key health indicators
            let kubernetes_ok = report
                .checks
                .iter()
                .find(|c| c.name == "Kubernetes Connection")
                .map(|c| c.status == crate::domain::entities::diagnostic::CheckStatus::Ok)
                .unwrap_or(false);

            let permissions_ok = report
                .checks
                .iter()
                .find(|c| c.name == "Kubernetes Permissions")
                .map(|c| c.status != crate::domain::entities::diagnostic::CheckStatus::Error)
                .unwrap_or(false);

            let healthy = kubernetes_ok && permissions_ok;

            let response = QuickHealthResponse {
                healthy,
                kubernetes: kubernetes_ok,
                permissions: permissions_ok,
                duration_ms: start.elapsed().as_millis() as u64,
            };

            (StatusCode::OK, Json(response))
        }
        Err(_) => {
            error!("Quick diagnostic check timed out");
            let response = QuickHealthResponse {
                healthy: false,
                kubernetes: false,
                permissions: false,
                duration_ms: start.elapsed().as_millis() as u64,
            };
            (StatusCode::REQUEST_TIMEOUT, Json(response))
        }
    }
}
