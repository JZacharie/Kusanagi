use actix_web::{get, post, web, HttpResponse, Responder};
use crate::application::use_cases::mcp_use_cases::*;
use crate::infrastructure::repositories::mcp_repository::LegacyMcpRepository;
use std::sync::Arc;

#[get("/api/mcp/k8s-resources")]
async fn get_k8s_resources() -> impl Responder {
    let mcp_repo = Arc::new(LegacyMcpRepository);
    let use_case = GetK8sResourcesUseCase::new(mcp_repo);
    
    match use_case.execute().await {
        Ok(resources) => HttpResponse::Ok().json(resources),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/mcp/cilium-policies")]
async fn get_cilium_policies() -> impl Responder {
    let mcp_repo = Arc::new(LegacyMcpRepository);
    let use_case = GetCiliumPoliciesUseCase::new(mcp_repo);
    
    match use_case.execute().await {
        Ok(policies) => HttpResponse::Ok().json(policies),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/mcp/trivy-vulnerabilities")]
async fn get_trivy_vulnerabilities() -> impl Responder {
    let mcp_repo = Arc::new(LegacyMcpRepository);
    let use_case = GetTrivyVulnerabilitiesUseCase::new(mcp_repo);
    
    match use_case.execute().await {
        Ok(vulns) => HttpResponse::Ok().json(vulns),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/mcp/steampipe/query")]
async fn query_steampipe(body: web::Json<serde_json::Value>) -> impl Responder {
    let query = body.get("query").and_then(|q| q.as_str()).unwrap_or("");
    
    let mcp_repo = Arc::new(LegacyMcpRepository);
    let use_case = QuerySteampipeUseCase::new(mcp_repo);
    
    match use_case.execute(query).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_k8s_resources)
        .service(get_cilium_policies)
        .service(get_trivy_vulnerabilities)
        .service(query_steampipe);
}
