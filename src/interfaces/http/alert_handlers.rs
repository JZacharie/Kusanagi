//! Alert HTTP Handlers
//! 
//! Interface layer for alert endpoints.
//! Uses the GetAlertsUseCase from the application layer.

use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::application::use_cases::{GetAlertsUseCase, GetAlertsInput};
use crate::domain::ports::AlertRepository;
use crate::infrastructure::repositories::AlertRepositoryImpl;

/// Query parameters for alerts endpoint
#[derive(Debug, serde::Deserialize)]
pub struct AlertsQuery {
    /// Force refresh of alerts data
    #[serde(default)]
    pub refresh: bool,
}

/// Get active alerts
/// 
/// # Endpoint
/// GET /api/alerts
/// 
/// # Query Parameters
/// - `refresh`: Force cache refresh (optional, default: false)
/// 
/// # Response
/// Returns a JSON object with grouped alerts (critical, warning, info)
pub async fn get_alerts_handler(
    use_case: web::Data<GetAlertsUseCase>,
    query: web::Query<AlertsQuery>,
) -> Result<HttpResponse> {
    debug!("Alerts request received, refresh={}", query.refresh);

    // Check local mode
    if use_case.is_local_mode() {
        debug!("Running in local mode, returning mock alerts");
    }

    let input = GetAlertsInput {
        force_refresh: query.refresh,
    };

    match use_case.execute(input).await {
        Ok(alerts) => {
            debug!("Alerts retrieved successfully: {} total", alerts.total);
            Ok(HttpResponse::Ok().json(alerts))
        }
        Err(e) => {
            error!("Failed to get alerts: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve alerts: {}", e)
            })))
        }
    }
}

/// Get active alerts (bypass cache)
/// 
/// # Endpoint
/// GET /api/alerts/active
/// 
/// # Response
/// Returns a JSON object with current active alerts from Alertmanager
pub async fn get_active_alerts_handler(
    use_case: web::Data<GetAlertsUseCase>,
) -> Result<HttpResponse> {
    debug!("Active alerts request received");

    match use_case.get_active_alerts().await {
        Ok(alerts) => {
            debug!("Active alerts retrieved successfully: {} total", alerts.total);
            Ok(HttpResponse::Ok().json(alerts))
        }
        Err(e) => {
            error!("Failed to get active alerts: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve active alerts: {}", e)
            })))
        }
    }
}

/// Refresh alerts cache
/// 
/// # Endpoint
/// POST /api/alerts/refresh
/// 
/// # Response
/// Returns 200 OK with fresh alerts data
pub async fn refresh_alerts_handler(
    use_case: web::Data<GetAlertsUseCase>,
) -> Result<HttpResponse> {
    info!("Manual alerts refresh requested");

    match use_case.refresh_alerts().await {
        Ok(alerts) => {
            info!("Alerts refreshed successfully: {} total", alerts.total);
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Alerts refreshed successfully",
                "alerts": alerts
            })))
        }
        Err(e) => {
            error!("Failed to refresh alerts: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to refresh alerts: {}", e)
            })))
        }
    }
}

/// Configure alert routes
/// 
/// Adds alert endpoints to the Actix-Web service configuration
pub fn configure_alert_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/alerts")
            .route("", web::get().to(get_alerts_handler))
            .route("/active", web::get().to(get_active_alerts_handler))
            .route("/refresh", web::post().to(refresh_alerts_handler)),
    );
}

/// Create GetAlertsUseCase with repository
/// 
/// Helper function to create the use case with the alert repository
pub fn create_alerts_use_case() -> GetAlertsUseCase {
    let repository: Arc<dyn AlertRepository> = Arc::new(AlertRepositoryImpl::new());
    GetAlertsUseCase::new(repository)
}
