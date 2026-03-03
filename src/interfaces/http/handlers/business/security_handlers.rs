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
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct ReportPath {
    pub category: String,
    pub name: String,
}

/// Get security summary
#[utoipa::path(
    get,
    path = "/api/security/summary",
    responses(
        (status = 200, description = "Security summary retrieved successfully"),
        (status = 500, description = "Failed to retrieve security summary")
    ),
    tag = "security"
)]
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
#[utoipa::path(
    get,
    path = "/api/security/reports",
    responses(
        (status = 200, description = "Security reports retrieved successfully"),
        (status = 500, description = "Failed to retrieve security reports")
    ),
    tag = "security"
)]
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
#[utoipa::path(
    get,
    path = "/api/security/vulnerabilities",
    responses(
        (status = 200, description = "Vulnerabilities retrieved successfully"),
        (status = 500, description = "Failed to retrieve vulnerabilities")
    ),
    tag = "security"
)]
pub async fn get_vulnerabilities_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Security vulnerabilities request received");

    match state.security_use_case.get_summary().await {
        Ok(summary) => {
            debug!(
                "Security vulnerabilities retrieved: {} total across {} reports",
                summary.total_vulnerabilities, summary.total_reports
            );

            // Build per-image entries by fetching each individual report
            let mut images: Vec<serde_json::Value> = Vec::new();

            for report_key in summary.reports.iter().take(200) {
                let parts: Vec<&str> = report_key.splitn(2, '/').collect();
                if parts.len() < 2 {
                    continue;
                }
                let category = parts[0];
                let name = parts[1];

                match state.security_use_case.get_report(category, name).await {
                    Ok(report) => {
                        let raw = &report.original_data;

                        // Extract counts from summary block (fast path)
                        let summary_block = &raw["report"]["summary"];
                        let (critical, high, medium, low) = if !summary_block.is_null() {
                            (
                                summary_block["criticalCount"].as_u64().unwrap_or(0),
                                summary_block["highCount"].as_u64().unwrap_or(0),
                                summary_block["mediumCount"].as_u64().unwrap_or(0),
                                summary_block["lowCount"].as_u64().unwrap_or(0),
                            )
                        } else {
                            // Count manually from vulnerabilities list
                            let vulns = raw["report"]["vulnerabilities"]
                                .as_array()
                                .or_else(|| raw["Report"]["Vulnerabilities"].as_array());
                            let mut c = 0u64; let mut h = 0u64; let mut m = 0u64; let mut l = 0u64;
                            if let Some(list) = vulns {
                                for v in list {
                                    match v["severity"].as_str().unwrap_or("").to_lowercase().as_str() {
                                        "critical" => c += 1,
                                        "high" => h += 1,
                                        "medium" => m += 1,
                                        "low" => l += 1,
                                        _ => {}
                                    }
                                }
                            }
                            (c, h, m, l)
                        };

                        // Extract image metadata
                        let artifact = &raw["report"]["artifact"];
                        let image_name = if !artifact.is_null() {
                            format!(
                                "{}:{}",
                                artifact["repository"].as_str().unwrap_or(name),
                                artifact["tag"].as_str().unwrap_or("latest")
                            )
                        } else {
                            name.replace(".json", "")
                        };

                        let namespace = raw["metadata"]["namespace"]
                            .as_str()
                            .unwrap_or(category)
                            .to_string();

                        let last_scan = raw["report"]["updateTimestamp"]
                            .as_str()
                            .unwrap_or(&summary.last_updated)
                            .to_string();

                        images.push(json!({
                            "image": image_name,
                            "namespace": namespace,
                            "critical_count": critical,
                            "high_count": high,
                            "medium_count": medium,
                            "low_count": low,
                            "last_scan": last_scan,
                            "report_id": report_key
                        }));
                    }
                    Err(e) => {
                        debug!("Skipping report {}: {}", report_key, e);
                    }
                }
            }

            api_success(json!({
                "critical": summary.critical_count,
                "high": summary.high_count,
                "medium": summary.medium_count,
                "low": summary.low_count,
                "total": summary.total_vulnerabilities,
                "images": images
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
#[utoipa::path(
    get,
    path = "/api/security/reports/{category}/{name}",
    params(
        ("category" = String, Path, description = "Report category"),
        ("name" = String, Path, description = "Report name")
    ),
    responses(
        (status = 200, description = "Security report retrieved successfully"),
        (status = 404, description = "Report not found")
    ),
    tag = "security"
)]
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
