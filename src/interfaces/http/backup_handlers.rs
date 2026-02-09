//! Backup HTTP Handlers
//!
//! Interface layer for backup endpoints.

use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::application::use_cases::BackupUseCase;
use crate::domain::ports::BackupRepository;
use crate::infrastructure::repositories::BackupRepositoryImpl;
use kube::Client;

/// Get backups status
///
/// # Endpoint
/// GET /api/backups
pub async fn get_backups_handler(use_case: web::Data<BackupUseCase>) -> Result<HttpResponse> {
    debug!("Backups status request received");

    match use_case.get_backups_status().await {
        Ok(backups) => {
            debug!(
                "Backups status retrieved: {} CronJobs",
                backups.total_cronjobs
            );
            Ok(HttpResponse::Ok().json(backups))
        }
        Err(e) => {
            error!("Failed to get backups status: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve backups status: {}", e)
            })))
        }
    }
}

/// Trigger a backup
///
/// # Endpoint
/// POST /api/backups/{namespace}/{name}/trigger
pub async fn trigger_backup_handler(
    use_case: web::Data<BackupUseCase>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse> {
    let (namespace, name) = path.into_inner();
    info!("Backup trigger requested for {}/{}", namespace, name);

    match use_case.trigger_backup(&namespace, &name).await {
        Ok(message) => {
            info!("Backup triggered successfully: {}", message);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": message
            })))
        }
        Err(e) => {
            error!("Failed to trigger backup: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to trigger backup: {}", e)
            })))
        }
    }
}

/// Configure backup routes
pub fn configure_backup_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/backups")
            .route("", web::get().to(get_backups_handler))
            .route(
                "/{namespace}/{name}/trigger",
                web::post().to(trigger_backup_handler),
            ),
    );
}

/// Create BackupUseCase with repository
pub fn create_backup_use_case(client: Arc<Client>) -> BackupUseCase {
    let repository: Arc<dyn BackupRepository> = Arc::new(BackupRepositoryImpl::new(client));
    BackupUseCase::new(repository)
}
