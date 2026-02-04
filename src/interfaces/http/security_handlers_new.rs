use actix_web::{get, post, web, HttpResponse, Responder};
use crate::application::use_cases::security_use_cases_new::*;
use crate::infrastructure::repositories::security_repository_new::LegacySecurityRepository;
use std::sync::Arc;

#[get("/api/security/reports")]
async fn list_security_reports() -> impl Responder {
    let security_repo = Arc::new(LegacySecurityRepository);
    let use_case = GetSecurityReportsUseCase::new(security_repo);
    
    match use_case.execute().await {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/security/reports/{report_id}/enrich")]
async fn enrich_security_report(path: web::Path<String>) -> impl Responder {
    let report_id = path.into_inner();
    let security_repo = Arc::new(LegacySecurityRepository);
    let use_case = EnrichSecurityReportUseCase::new(security_repo);
    
    match use_case.execute(&report_id).await {
        Ok(enrichment) => HttpResponse::Ok().json(enrichment),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_security_reports)
        .service(enrich_security_report);
}
