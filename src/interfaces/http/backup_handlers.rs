//! Backup HTTP Handlers
//!
//! HTTP handlers for backup operations.

use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::backup_use_cases::*;

use crate::interfaces::http::AppState;

#[derive(Debug, Deserialize)]
pub struct ListCronJobsQuery {
    pub namespace: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TriggerBackupPath {
    pub namespace: String,
    pub name: String,
}

/// Get backup status
#[get("/api/backups")]
pub async fn get_backup_status(
    data: web::Data<AppState>,
) -> impl Responder {
    let repo = match data.get_backup_repo() {
        Some(repo) => repo,
        None => return HttpResponse::Ok().json(serde_json::json!({
            "total_cronjobs": 0,
            "cronjobs": [],
            "active_jobs": 0,
            "succeeded_jobs": 0,
            "failed_jobs": 0,
            "_warning": "Backup repository not available - hexagonal architecture migration in progress"
        })),
    };

    let use_case = GetBackupStatusUseCase::new(repo);

    match use_case.execute().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => HttpResponse::Ok().json(serde_json::json!({
            "total_cronjobs": 0,
            "cronjobs": [],
            "active_jobs": 0,
            "succeeded_jobs": 0,
            "failed_jobs": 0,
            "_warning": format!("Backup error: {}", e)
        })),
    }
}

/// Get backup statistics
#[get("/api/backups/stats")]
pub async fn get_backup_stats(
    data: web::Data<AppState>,
) -> impl Responder {
    let repo = match data.get_backup_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Backup repository not available"
            })),
    };

    let use_case = GetBackupStatsUseCase::new(repo);

    match use_case.execute().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => e.error_response(),
    }
}

/// List CronJobs
#[get("/api/backups/cronjobs")]
pub async fn list_cronjobs(
    data: web::Data<AppState>,
    query: web::Query<ListCronJobsQuery>,
) -> impl Responder {
    let repo = match data.get_backup_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Backup repository not available"
            })),
    };

    let use_case = ListCronJobsUseCase::new(repo);

    match use_case.execute(query.namespace.as_deref()).await {
        Ok(cronjobs) => HttpResponse::Ok().json(cronjobs),
        Err(e) => e.error_response(),
    }
}

/// Trigger a backup
#[post("/api/backups/{namespace}/{name}/trigger")]
pub async fn trigger_backup(
    data: web::Data<AppState>,
    path: web::Path<TriggerBackupPath>,
) -> impl Responder {
    let repo = match data.get_backup_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Backup repository not available"
            })),
    };

    let use_case = TriggerBackupUseCase::new(repo);

    match use_case.execute(&path.namespace, &path.name).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Backup triggered for {}/{}", path.namespace, path.name)
        })),
        Err(e) => e.error_response(),
    }
}
