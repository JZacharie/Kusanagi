//! Alert HTTP Handlers
//!
//! Interface layer for alert endpoints.
//! Uses the GetAlertsUseCase from the application layer.

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use tracing::{debug, error, info};

use crate::{application::use_cases::GetAlertsInput, state::AppState};

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
    State(state): State<AppState>,
    Query(query): Query<AlertsQuery>,
) -> impl IntoResponse {
    debug!("Alerts request received, refresh={}", query.refresh);

    // Check local mode
    if state.alerts_use_case.is_local_mode() {
        debug!("Running in local mode, returning mock alerts");
    }

    let input = GetAlertsInput {
        force_refresh: query.refresh,
    };

    match state.alerts_use_case.execute(input).await {
        Ok(alerts) => {
            debug!("Alerts retrieved successfully: {} total", alerts.total);
            Json(alerts).into_response()
        }
        Err(e) => {
            error!("Failed to get alerts: {}", e);
            Json(serde_json::json!({
                "error": format!("Failed to retrieve alerts: {}", e)
            }))
            .into_response()
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
pub async fn get_active_alerts_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Active alerts request received");

    match state.alerts_use_case.get_active_alerts().await {
        Ok(alerts) => {
            debug!(
                "Active alerts retrieved successfully: {} total",
                alerts.total
            );
            Json(alerts).into_response()
        }
        Err(e) => {
            error!("Failed to get active alerts: {}", e);
            Json(serde_json::json!({
                "error": format!("Failed to retrieve active alerts: {}", e)
            }))
            .into_response()
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
pub async fn refresh_alerts_handler(State(state): State<AppState>) -> impl IntoResponse {
    info!("Manual alerts refresh requested");

    match state.alerts_use_case.refresh_alerts().await {
        Ok(alerts) => {
            info!("Alerts refreshed successfully: {} total", alerts.total);
            Json(serde_json::json!({
                "message": "Alerts refreshed successfully",
                "alerts": alerts
            }))
            .into_response()
        }
        Err(e) => {
            error!("Failed to refresh alerts: {}", e);
            Json(serde_json::json!({
                "error": format!("Failed to refresh alerts: {}", e)
            }))
            .into_response()
        }
    }
}
