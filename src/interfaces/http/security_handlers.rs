//! Security HTTP Handlers
//!
//! HTTP handlers for security operations.

use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::security_use_cases::*;

use crate::interfaces::http::AppState;

#[derive(Debug, Deserialize)]
pub struct GetReportPath {
    pub category: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct EnrichQuery {
    pub lang: Option<String>,
}

/// List security reports
#[get("/api/security/reports")]
pub async fn list_security_reports(
    data: web::Data<AppState>,
) -> impl Responder {
    let repo = match data.get_security_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Security repository not available"
            })),
    };

    let use_case = ListSecurityReportsUseCase::new(repo);

    match use_case.execute().await {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(e) => e.error_response(),
    }
}

/// Get security summary
#[get("/api/security/summary")]
pub async fn get_security_summary(
    data: web::Data<AppState>,
) -> impl Responder {
    let repo = match data.get_security_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Security repository not available"
            })),
    };

    let use_case = GetSecuritySummaryUseCase::new(repo);

    match use_case.execute().await {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(e) => e.error_response(),
    }
}

/// Get enriched security report
#[get("/api/security/enriched/{category}/{name}")]
pub async fn get_enriched_report(
    data: web::Data<AppState>,
    path: web::Path<GetReportPath>,
) -> impl Responder {
    let repo = match data.get_security_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Security repository not available"
            })),
    };

    let use_case = GetSecurityReportUseCase::new(repo);

    match use_case.execute(&path.category, &path.name).await {
        Ok(report) => HttpResponse::Ok().json(report),
        Err(e) => e.error_response(),
    }
}

/// Enrich a security report
#[post("/api/security/enrich/{category}/{name}")]
pub async fn enrich_security_report(
    data: web::Data<AppState>,
    path: web::Path<GetReportPath>,
    query: web::Query<EnrichQuery>,
) -> impl Responder {
    let repo = match data.get_security_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Security repository not available"
            })),
    };

    let scanner = match data.get_vulnerability_scanner() {
        Some(scanner) => scanner,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Vulnerability scanner not available"
            })),
    };

    let enrichment = match data.get_ai_enrichment_service() {
        Some(service) => service,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "AI enrichment service not available"
            })),
    };

    let use_case = EnrichSecurityReportUseCase::new(scanner, enrichment, repo);
    let lang = query.lang.as_deref().unwrap_or("en");

    match use_case.execute(&path.category, &path.name, lang).await {
        Ok(report) => HttpResponse::Ok().json(report),
        Err(e) => e.error_response(),
    }
}

/// Run security enrichment worker
#[post("/api/security/enrich-all")]
pub async fn run_security_enrichment(
    data: web::Data<AppState>,
    query: web::Query<EnrichQuery>,
) -> impl Responder {
    let repo = match data.get_security_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Security repository not available"
            })),
    };

    let scanner = match data.get_vulnerability_scanner() {
        Some(scanner) => scanner,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Vulnerability scanner not available"
            })),
    };

    let enrichment = match data.get_ai_enrichment_service() {
        Some(service) => service,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "AI enrichment service not available"
            })),
    };

    let use_case = RunSecurityEnrichmentUseCase::new(scanner, enrichment, repo);
    let lang = query.lang.as_deref().unwrap_or("en");

    match use_case.execute(lang).await {
        Ok(count) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Enriched {} reports", count),
            "processed": count
        })),
        Err(e) => e.error_response(),
    }
}
