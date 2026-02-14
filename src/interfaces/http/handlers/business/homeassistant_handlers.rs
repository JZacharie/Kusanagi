//! HomeAssistant HTTP Handlers
//!
//! Interface layer for HomeAssistant endpoints.
//! Migrated from Actix-web to Axum.

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use serde_json::json;
use tracing::{debug, error};

use crate::interfaces::http::response::{api_error, api_success};
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
            api_success(json!({
                "sensors": response.sensors,
                "count": response.count
            }))
        }
        Err(e) => {
            error!("Failed to get HomeAssistant sensors: {}", e);
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch sensors: {}", e),
            )
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
            api_success(json!({
                "devices": response.devices,
                "count": response.count
            }))
        }
        Err(e) => {
            error!("Failed to get HomeAssistant devices: {}", e);
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch devices: {}", e),
            )
        }
    }
}

/// Get automations from Home Assistant
///
/// # Endpoint
/// GET /api/ha/automations
pub async fn get_automations_handler() -> impl IntoResponse {
    debug!("HomeAssistant automations request received");

    use crate::domain::services::homeassistant_service::get_ha_automations;

    match get_ha_automations().await {
        Ok(automations) => {
            debug!("HomeAssistant automations retrieved successfully");
            api_success(json!({
                "automations": automations,
                "count": automations.as_array().map(|a| a.len()).unwrap_or(0)
            }))
        }
        Err(e) => {
            error!("Failed to get HomeAssistant automations: {}", e);
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch automations: {}", e),
            )
        }
    }
}
