use actix_web::{get, post, web, HttpResponse, Responder};
use crate::application::use_cases::proxmox_use_cases::*;
use crate::infrastructure::repositories::proxmox_repository::LegacyProxmoxRepository;
use std::sync::Arc;

#[get("/api/proxmox/cluster")]
async fn get_cluster_status() -> impl Responder {
    let proxmox_repo = Arc::new(LegacyProxmoxRepository);
    let use_case = GetProxmoxClusterUseCase::new(proxmox_repo);
    
    match use_case.execute().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/proxmox/vms")]
async fn get_vms() -> impl Responder {
    let proxmox_repo = Arc::new(LegacyProxmoxRepository);
    let use_case = GetProxmoxVMsUseCase::new(proxmox_repo);
    
    match use_case.execute().await {
        Ok(vms) => HttpResponse::Ok().json(vms),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/proxmox/vm/{vmid}/control")]
async fn vm_control(path: web::Path<u32>, body: web::Json<serde_json::Value>) -> impl Responder {
    let vmid = path.into_inner();
    let action = body.get("action").and_then(|a| a.as_str()).unwrap_or("status");
    
    let proxmox_repo = Arc::new(LegacyProxmoxRepository);
    let use_case = ControlProxmoxVMUseCase::new(proxmox_repo);
    
    match use_case.execute(vmid, action).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_cluster_status)
        .service(get_vms)
        .service(vm_control);
}
