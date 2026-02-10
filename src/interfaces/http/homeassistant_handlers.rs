//! HomeAssistant HTTP Handlers
//!
//! Interface layer for HomeAssistant endpoints.
//! Uses the GetHomeAssistantUseCase from the application layer.

use axum::{extract::State, response::IntoResponse, Json};
use tracing::{debug, error, info};

use crate::{infrastructure::repositories::HomeAssistantRepositoryImpl, state::AppState};

/// Get sensors from Home Assistant
///
/// # Endpoint
/// GET /api/ha/sensors
///
/// # Response
/// Returns a JSON object with sensors array and count
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
            // Return empty response on error
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
///
/// # Response
/// Returns a JSON object with devices array and count
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

/// Get automations from Home Assistant
///
/// # Endpoint
/// GET /api/ha/automations
///
/// # Response
/// Returns a JSON object with automations array
pub async fn get_automations_handler() -> impl IntoResponse {
    debug!("HomeAssistant automations request received");

    // Return empty automations list for now (Home Assistant not configured)
    Json(serde_json::json!({
        "automations": [],
        "count": 0
    }))
}

/// Check Home Assistant configuration status
///
/// # Endpoint
/// GET /api/ha/status
///
/// # Response
/// Returns a JSON object with configuration status
pub async fn get_ha_status_handler() -> impl IntoResponse {
    debug!("HomeAssistant status request received");

    match HomeAssistantRepositoryImpl::new() {
        Ok(repo) => {
            let configured = repo.is_configured();
            let status = if configured {
                "configured"
            } else {
                "not_configured"
            };

            info!("HomeAssistant status check: {}", status);

            Json(serde_json::json!({
                "status": status,
                "configured": configured
            }))
        }
        Err(e) => {
            error!("Failed to check HomeAssistant status: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "configured": false,
                "error": format!("{}", e)
            }))
        }
    }
}
