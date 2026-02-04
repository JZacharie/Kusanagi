use actix_web::{get, web, HttpResponse, Responder};
use crate::application::use_cases::cilium_use_cases::*;
use crate::infrastructure::repositories::cilium_repository::LegacyCiliumRepository;
use std::sync::Arc;

#[get("/api/cilium/flows")]
async fn get_network_flows(query: web::Query<serde_json::Value>) -> impl Responder {
    let namespace = query.get("namespace").and_then(|v| v.as_str());
    let cilium_repo = Arc::new(LegacyCiliumRepository);
    let use_case = GetNetworkFlowsUseCase::new(cilium_repo);
    
    match use_case.execute(namespace).await {
        Ok(flows) => HttpResponse::Ok().json(flows),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/cilium/policies")]
async fn get_network_policies() -> impl Responder {
    let cilium_repo = Arc::new(LegacyCiliumRepository);
    let use_case = GetNetworkPoliciesUseCase::new(cilium_repo);
    
    match use_case.execute().await {
        Ok(policies) => HttpResponse::Ok().json(policies),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/cilium/bandwidth")]
async fn get_bandwidth_metrics() -> impl Responder {
    let cilium_repo = Arc::new(LegacyCiliumRepository);
    let use_case = GetBandwidthMetricsUseCase::new(cilium_repo);
    
    match use_case.execute().await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_network_flows)
        .service(get_network_policies)
        .service(get_bandwidth_metrics);
}
