//! Alert HTTP Handlers
//!
//! Interface layer for alert endpoints.
//! Migrated from Actix-web to Axum.

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use tracing::{debug, error, info};

use crate::application::use_cases::GetAlertsInput;
use crate::state::AppState;

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
pub async fn get_alerts_handler(
    State(state): State<AppState>,
    Query(query): Query<AlertsQuery>,
) -> impl IntoResponse {
    debug!("Alerts request received, refresh={}", query.refresh);

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
