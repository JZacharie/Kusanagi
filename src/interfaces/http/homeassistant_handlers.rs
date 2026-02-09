//! HomeAssistant HTTP Handlers
//!
//! Interface layer for HomeAssistant endpoints.
//! Uses the GetHomeAssistantUseCase from the application layer.

use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::application::use_cases::GetHomeAssistantUseCase;
use crate::domain::ports::HomeAssistantRepository;
use crate::infrastructure::repositories::HomeAssistantRepositoryImpl;

/// Get sensors from Home Assistant
///
/// # Endpoint
/// GET /api/ha/sensors
///
/// # Response
/// Returns a JSON object with sensors array and count
pub async fn get_sensors_handler(
    use_case: web::Data<GetHomeAssistantUseCase>,
) -> Result<HttpResponse> {
    debug!("HomeAssistant sensors request received");

    match use_case.get_sensors().await {
        Ok(response) => {
            debug!(
                "HomeAssistant sensors retrieved successfully: {} sensors",
                response.count
            );
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!("Failed to get HomeAssistant sensors: {}", e);
            // Return 503 Service Unavailable for configuration errors
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Failed to fetch sensors: {}", e)
            })))
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
pub async fn get_devices_handler(
    use_case: web::Data<GetHomeAssistantUseCase>,
) -> Result<HttpResponse> {
    debug!("HomeAssistant devices request received");

    match use_case.get_devices().await {
        Ok(response) => {
            debug!(
                "HomeAssistant devices retrieved successfully: {} devices",
                response.count
            );
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            error!("Failed to get HomeAssistant devices: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Failed to fetch devices: {}", e)
            })))
        }
    }
}

/// Check Home Assistant configuration status
///
/// # Endpoint
/// GET /api/ha/status
///
/// # Response
/// Returns a JSON object with configuration status
pub async fn get_ha_status_handler() -> Result<HttpResponse> {
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

            Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": status,
                "configured": configured
            })))
        }
        Err(e) => {
            error!("Failed to check HomeAssistant status: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "status": "error",
                "configured": false,
                "error": format!("{}", e)
            })))
        }
    }
}

/// Configure HomeAssistant routes
///
/// Adds HomeAssistant endpoints to the Actix-Web service configuration
pub fn configure_ha_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/ha")
            .route("/sensors", web::get().to(get_sensors_handler))
            .route("/devices", web::get().to(get_devices_handler))
            .route("/status", web::get().to(get_ha_status_handler)),
    );
}

/// Create GetHomeAssistantUseCase with repository
///
/// Helper function to create the use case with the HomeAssistant repository
pub fn create_homeassistant_use_case() -> Option<GetHomeAssistantUseCase> {
    match HomeAssistantRepositoryImpl::new() {
        Ok(repo) => {
            let repository: Arc<dyn HomeAssistantRepository> = Arc::new(repo);
            Some(GetHomeAssistantUseCase::new(repository))
        }
        Err(e) => {
            error!("Failed to create HomeAssistant repository: {}", e);
            None
        }
    }
}
