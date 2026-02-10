//! Security HTTP Handlers
//!
//! Interface layer for security endpoints.
//! Migrated from Actix-web to Axum.

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use tracing::{debug, error};

use crate::state::AppState;

/// Path parameters for getting a specific report
#[derive(Debug, serde::Deserialize)]
pub struct ReportPath {
    pub category: String,
    pub name: String,
}

/// Get security summary
///
/// # Endpoint
/// GET /api/security/summary
pub async fn get_security_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Security summary request received");

    if state.security_use_case.is_local_mode() {
        debug!("Running in local mode, returning mock security summary");
    }

    match state.security_use_case.get_summary().await {
        Ok(summary) => {
            debug!(
                "Security summary retrieved: {} reports, {} vulnerabilities",
                summary.total_reports, summary.total_vulnerabilities
            );
            Json(summary).into_response()
        }
        Err(e) => {
            error!("Failed to get security summary: {}", e);
            Json(serde_json::json!({
                "total_reports": 0,
                "total_vulnerabilities": 0,
                "critical_count": 0,
                "high_count": 0,
                "medium_count": 0,
                "low_count": 0,
                "reports": [],
                "error": format!("Failed to retrieve security summary: {}", e)
            }))
            .into_response()
        }
    }
}

/// Get list of all security reports
///
/// # Endpoint
/// GET /api/security/reports
pub async fn get_security_reports_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Security reports list request received");

    match state.security_use_case.get_reports().await {
        Ok(reports) => {
            debug!("Security reports retrieved: {} reports", reports.len());
            Json(reports).into_response()
        }
        Err(e) => {
            error!("Failed to get security reports: {}", e);
            Json(serde_json::json!([])).into_response()
        }
    }
}

/// Get vulnerabilities summary
///
/// # Endpoint
/// GET /api/security/vulnerabilities
pub async fn get_vulnerabilities_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Security vulnerabilities request received");

    match state.security_use_case.get_summary().await {
        Ok(summary) => {
            debug!(
                "Security vulnerabilities retrieved: {} total",
                summary.total_vulnerabilities
            );
            Json(serde_json::json!({
                "critical": summary.critical_count,
                "high": summary.high_count,
                "medium": summary.medium_count,
                "low": summary.low_count,
                "total": summary.total_vulnerabilities,
                "images": []
            }))
            .into_response()
        }
        Err(e) => {
            error!("Failed to get security vulnerabilities: {}", e);
            Json(serde_json::json!({
                "critical": 0,
                "high": 0,
                "medium": 0,
                "low": 0,
                "total": 0,
                "images": [],
                "error": format!("Failed to retrieve vulnerabilities: {}", e)
            }))
            .into_response()
        }
    }
}

/// Get a specific security report
///
/// # Endpoint
/// GET /api/security/reports/{category}/{name}
pub async fn get_security_report_handler(
    State(state): State<AppState>,
    Path(path): Path<ReportPath>,
) -> impl IntoResponse {
    debug!(
        "Security report request received: {}/{}",
        path.category, path.name
    );

    match state
        .security_use_case
        .get_report(&path.category, &path.name)
        .await
    {
        Ok(report) => {
            debug!("Security report retrieved: {}/{}", path.category, path.name);
            Json(report).into_response()
        }
        Err(e) => {
            error!(
                "Failed to get security report {}/{}: {}",
                path.category, path.name, e
            );
            Json(serde_json::json!({
                "error": format!("Report not found: {}/{}", path.category, path.name)
            }))
            .into_response()
        }
    }
}
