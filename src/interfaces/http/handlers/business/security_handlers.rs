//! Security HTTP Handlers
//!
//! Interface layer for security endpoints.
//! Migrated from Actix-web to Axum.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use tracing::{debug, error};

use crate::interfaces::http::response::{api_error, api_success};
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
            api_success(serde_json::to_value(summary).unwrap_or_default())
        }
        Err(e) => {
            error!("Failed to get security summary: {}", e);
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve security summary: {}", e),
            )
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
            api_success(json!(reports))
        }
        Err(e) => {
            error!("Failed to get security reports: {}", e);
            api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e))
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
            api_success(json!({
                "critical": summary.critical_count,
                "high": summary.high_count,
                "medium": summary.medium_count,
                "low": summary.low_count,
                "total": summary.total_vulnerabilities,
                "images": []
            }))
        }
        Err(e) => {
            error!("Failed to get security vulnerabilities: {}", e);
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve vulnerabilities: {}", e),
            )
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
            api_success(json!(report))
        }
        Err(e) => {
            error!(
                "Failed to get security report {}/{}: {}",
                path.category, path.name, e
            );
            api_error(
                StatusCode::NOT_FOUND,
                format!("Report not found: {}/{}", path.category, path.name),
            )
        }
    }
}
