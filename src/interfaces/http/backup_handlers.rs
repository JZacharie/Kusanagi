//! Backup HTTP Handlers
//!
//! Interface layer for backup endpoints.

use axum::{extract::State, response::IntoResponse, Json};
use tracing::{debug, info};

use crate::state::AppState;

/// Get backups status
///
/// # Endpoint
/// GET /api/backups
pub async fn get_backups_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Backups status request received");

    // Note: Backup use case needs to be added to AppState
    // For now, return empty response
    Json(serde_json::json!({
        "cronjobs": [],
        "total_cronjobs": 0,
        "error": "Backup use case not yet migrated"
    }))
}

/// Trigger a backup
///
/// # Endpoint
/// POST /api/backups/trigger
pub async fn trigger_backup_handler(State(state): State<AppState>) -> impl IntoResponse {
    info!("Backup trigger requested");

    // Note: Backup use case needs to be added to AppState
    Json(serde_json::json!({
        "error": "Backup use case not yet migrated"
    }))
}
