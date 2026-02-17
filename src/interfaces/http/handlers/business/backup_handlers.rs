//! Backup HTTP Handlers
//!
//! Interface layer for backup endpoints.
//! Migrated from Actix-web to Axum.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use tracing::{debug, error, info};

use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

/// Get backups status
#[utoipa::path(
    get,
    path = "/api/backups",
    responses(
        (status = 200, description = "Backups status retrieved successfully"),
        (status = 500, description = "Failed to retrieve backups status")
    ),
    tag = "backups"
)]
pub async fn get_backups_handler(State(state): State<AppState>) -> impl IntoResponse {
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
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve backups status: {}", e),
            )
        }
    }
}

/// Trigger a backup
#[utoipa::path(
    post,
    path = "/api/backups/{namespace}/{name}/trigger",
    params(
        ("namespace" = String, Path, description = "Namespace of the CronJob"),
        ("name" = String, Path, description = "Name of the CronJob")
    ),
    responses(
        (status = 200, description = "Backup triggered successfully"),
        (status = 500, description = "Failed to trigger backup")
    ),
    tag = "backups"
)]
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
            api_success(json!({"message": message}))
        }
        Err(e) => {
            error!("Failed to trigger backup: {}", e);
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to trigger backup: {}", e),
            )
        }
    }
}
