//! Backup HTTP Handlers

use axum::{
    extract::{Path, State},
    response::Response,
};
use tracing::{debug, error, info};

use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

/// Get backups status
///
/// # Endpoint
/// GET /api/backups
pub async fn get_backups_handler(State(state): State<AppState>) -> Response {
    debug!("Backups status request received");

    match state.backup_use_case.get_backups_status().await {
        Ok(backups) => {
            debug!(
                "Backups status retrieved: {} CronJobs",
                backups.total_cronjobs
            );
            api_success(serde_json::to_value(backups).unwrap_or_default())
        }
        Err(e) => {
            error!("Failed to get backups status: {}", e);
            api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve backups status: {}", e),
            )
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
) -> Response {
    info!("Backup trigger requested for {}/{}", namespace, name);

    match state
        .backup_use_case
        .trigger_backup(&namespace, &name)
        .await
    {
        Ok(message) => {
            info!("Backup triggered successfully: {}", message);
            api_success(serde_json::json!({
                "message": message
            }))
        }
        Err(e) => {
            error!("Failed to trigger backup: {}", e);
            api_error(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to trigger backup: {}", e),
            )
        }
    }
}
