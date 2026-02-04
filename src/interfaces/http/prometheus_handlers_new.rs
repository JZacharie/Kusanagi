use actix_web::{get, post, web, HttpResponse, Responder};
use crate::application::use_cases::prometheus_use_cases_new::*;
use crate::infrastructure::repositories::prometheus_repository_new::LegacyPrometheusRepository;
use std::sync::Arc;

#[get("/api/prometheus/metrics")]
async fn get_cluster_metrics() -> impl Responder {
    let prometheus_repo = Arc::new(LegacyPrometheusRepository);
    let use_case = GetPrometheusMetricsUseCase::new(prometheus_repo);
    
    match use_case.execute().await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/prometheus/query")]
async fn query_prometheus(body: web::Json<serde_json::Value>) -> impl Responder {
    let query = body.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let prometheus_repo = Arc::new(LegacyPrometheusRepository);
    let use_case = QueryPrometheusUseCase::new(prometheus_repo);
    
    match use_case.execute_raw(query).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/prometheus/query_range")]
async fn query_range(body: web::Json<serde_json::Value>) -> impl Responder {
    let query = body.get("query").and_then(|q| q.as_str()).unwrap_or("");
    let start = body.get("start").and_then(|s| s.as_i64()).unwrap_or(0);
    let end = body.get("end").and_then(|e| e.as_i64()).unwrap_or(0);
    let step = body.get("step").and_then(|s| s.as_str()).unwrap_or("1m");
    
    let prometheus_repo = Arc::new(LegacyPrometheusRepository);
    let use_case = QueryPrometheusUseCase::new(prometheus_repo);
    
    match use_case.execute_range(query, start, end, step).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_cluster_metrics)
        .service(query_prometheus)
        .service(query_range);
}
