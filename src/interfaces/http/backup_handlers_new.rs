use actix_web::{get, post, web, HttpResponse, Responder};
use crate::application::use_cases::backup_use_cases_new::*;
use crate::infrastructure::repositories::backup_repository_new::LegacyBackupRepository;
use std::sync::Arc;

#[get("/api/backups/status")]
async fn get_backup_status() -> impl Responder {
    let backup_repo = Arc::new(LegacyBackupRepository);
    let use_case = GetBackupStatusUseCase::new(backup_repo);
    
    match use_case.execute().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/backups/{backup_name}/trigger")]
async fn trigger_backup(path: web::Path<String>) -> impl Responder {
    let backup_name = path.into_inner();
    let backup_repo = Arc::new(LegacyBackupRepository);
    let use_case = TriggerBackupUseCase::new(backup_repo);
    
    match use_case.execute(&backup_name).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "triggered"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_backup_status)
        .service(trigger_backup);
}
