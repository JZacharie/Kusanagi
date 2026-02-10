//! Security HTTP Handlers
//!
//! Interface layer for security endpoints.
//! Uses the GetSecurityUseCase from the application layer.

use actix_web::{web, HttpResponse};
use std::sync::Arc;
use tracing::{debug, error};

use crate::application::use_cases::GetSecurityUseCase;
use crate::domain::ports::SecurityRepository;
use crate::infrastructure::repositories::SecurityRepositoryImpl;

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
///
/// # Response
/// Returns a JSON object with security summary including:
/// - total_reports: Total number of security reports
/// - total_vulnerabilities: Total vulnerabilities across all reports
/// - critical_count, high_count, medium_count, low_count: Vulnerability counts by severity
/// - reports: List of report keys
/// - last_updated: Timestamp of last update
pub async fn get_security_handler(use_case: web::Data<GetSecurityUseCase>) -> HttpResponse {
    debug!("Security summary request received");

    // Check local mode
    if use_case.is_local_mode() {
        debug!("Running in local mode, returning mock security summary");
    }

    match use_case.get_summary().await {
        Ok(summary) => {
            debug!(
                "Security summary retrieved: {} reports, {} vulnerabilities",
                summary.total_reports, summary.total_vulnerabilities
            );
            HttpResponse::Ok().json(summary)
        }
        Err(e) => {
            error!("Failed to get security summary: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "total_reports": 0,
                "total_vulnerabilities": 0,
                "critical_count": 0,
                "high_count": 0,
                "medium_count": 0,
                "low_count": 0,
                "reports": [],
                "error": format!("Failed to retrieve security summary: {}", e)
            }))
        }
    }
}

/// Get list of all security reports
///
/// # Endpoint
/// GET /api/security/reports
///
/// # Response
/// Returns a JSON array of report keys (e.g., ["cluster/report1.json", "apps/app-report.json"])
pub async fn get_security_reports_handler(use_case: web::Data<GetSecurityUseCase>) -> HttpResponse {
    debug!("Security reports list request received");

    match use_case.get_reports().await {
        Ok(reports) => {
            debug!("Security reports retrieved: {} reports", reports.len());
            HttpResponse::Ok().json(reports)
        }
        Err(e) => {
            error!("Failed to get security reports: {}", e);
            HttpResponse::Ok().json(serde_json::json!([]))
        }
    }
}

/// Get vulnerabilities summary
///
/// # Endpoint
/// GET /api/security/vulnerabilities
///
/// # Response
/// Returns a JSON object with vulnerability counts by severity
pub async fn get_vulnerabilities_handler(use_case: web::Data<GetSecurityUseCase>) -> HttpResponse {
    debug!("Security vulnerabilities request received");

    match use_case.get_summary().await {
        Ok(summary) => {
            debug!(
                "Security vulnerabilities retrieved: {} total",
                summary.total_vulnerabilities
            );
            HttpResponse::Ok().json(serde_json::json!({
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
            HttpResponse::Ok().json(serde_json::json!({
                "critical": 0,
                "high": 0,
                "medium": 0,
                "low": 0,
                "total": 0,
                "images": [],
                "error": format!("Failed to retrieve vulnerabilities: {}", e)
            }))
        }
    }
}

/// Get a specific security report
///
/// # Endpoint
/// GET /api/security/reports/{category}/{name}
///
/// # Path Parameters
/// - `category`: Report category (e.g., "cluster", "apps")
/// - `name`: Report filename (e.g., "report.json")
///
/// # Response
/// Returns the full security report with original Trivy data and optional AI enrichment
pub async fn get_security_report_handler(
    use_case: web::Data<GetSecurityUseCase>,
    path: web::Path<ReportPath>,
) -> HttpResponse {
    let ReportPath { category, name } = path.into_inner();
    debug!("Security report request received: {}/{}", category, name);

    match use_case.get_report(&category, &name).await {
        Ok(report) => {
            debug!("Security report retrieved: {}/{}", category, name);
            HttpResponse::Ok().json(report)
        }
        Err(e) => {
            error!("Failed to get security report {}/{}: {}", category, name, e);
            HttpResponse::Ok().json(serde_json::json!({
                "error": format!("Report not found: {}/{}", category, name)
            }))
        }
    }
}

/// Configure security routes
///
/// Adds security endpoints to the Actix-Web service configuration
pub fn configure_security_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/security")
            .route("/summary", web::get().to(get_security_handler))
            .route(
                "/vulnerabilities",
                web::get().to(get_vulnerabilities_handler),
            )
            .route("/reports", web::get().to(get_security_reports_handler))
            .route(
                "/reports/{category}/{name}",
                web::get().to(get_security_report_handler),
            ),
    );
}

/// Create GetSecurityUseCase with repository
///
/// Helper function to create the use case with the security repository
pub async fn create_security_use_case() -> GetSecurityUseCase {
    let repository: Arc<dyn SecurityRepository> = Arc::new(SecurityRepositoryImpl::new().await);
    GetSecurityUseCase::new(repository)
}
