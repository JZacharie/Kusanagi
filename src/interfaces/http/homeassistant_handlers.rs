//! HomeAssistant HTTP Handlers
//!
//! Interface layer for HomeAssistant endpoints.
//! Migrated from Actix-web to Axum.

use axum::{extract::State, response::IntoResponse, Json};
use tracing::{debug, error};

use crate::state::AppState;

/// Get sensors from Home Assistant
///
/// # Endpoint
/// GET /api/ha/sensors
pub async fn get_sensors_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("HomeAssistant sensors request received");

    match state.ha_use_case.get_sensors().await {
        Ok(response) => {
            debug!(
                "HomeAssistant sensors retrieved successfully: {} sensors",
                response.count
            );
            Json(response).into_response()
        }
        Err(e) => {
            error!("Failed to get HomeAssistant sensors: {}", e);
            Json(serde_json::json!({
                "sensors": [],
                "count": 0,
                "error": format!("Failed to fetch sensors: {}", e)
            }))
            .into_response()
        }
    }
}

/// Get devices from Home Assistant
///
/// # Endpoint
/// GET /api/ha/devices
pub async fn get_devices_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("HomeAssistant devices request received");

    match state.ha_use_case.get_devices().await {
        Ok(response) => {
            debug!(
                "HomeAssistant devices retrieved successfully: {} devices",
                response.count
            );
            Json(response).into_response()
        }
        Err(e) => {
            error!("Failed to get HomeAssistant devices: {}", e);
            Json(serde_json::json!({
                "devices": [],
                "count": 0,
                "error": format!("Failed to fetch devices: {}", e)
            }))
            .into_response()
        }
    }
}
