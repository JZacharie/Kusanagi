//! Backup HTTP Handlers
//!
//! Interface layer for backup endpoints.
//! Migrated from Actix-web to Axum.

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::state::AppState;

/// Get backups status
///
/// # Endpoint
/// GET /api/backups
pub async fn get_backups_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Backups status request received");

    match state.backup_use_case.get_backups_status().await {
        Ok(backups) => {
            debug!(
                "Backups status retrieved: {} CronJobs",
                backups.total_cronjobs
            );
            Json(backups).into_response()
        }
        Err(e) => {
            error!("Failed to get backups status: {}", e);
            Json(serde_json::json!({
                "error": format!("Failed to retrieve backups status: {}", e)
            }))
            .into_response()
        }
    }
}

/// Trigger a backup
///
/// # Endpoint
/// POST /api/backups/{namespace}/{name}/trigger
pub async fn trigger_backup_handler(
    State(state): State<AppState>,
    Path((namespace, name)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Backup trigger requested for {}/{}", namespace, name);

    match state
        .backup_use_case
        .trigger_backup(&namespace, &name)
        .await
    {
        Ok(message) => {
            info!("Backup triggered successfully: {}", message);
            Json(serde_json::json!({
                "message": message
            }))
            .into_response()
        }
        Err(e) => {
            error!("Failed to trigger backup: {}", e);
            Json(serde_json::json!({
                "error": format!("Failed to trigger backup: {}", e)
            }))
            .into_response()
        }
    }
}
