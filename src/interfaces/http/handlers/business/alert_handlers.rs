//! Alert HTTP Handlers
//!
//! Interface layer for alert endpoints.
//! Migrated from Actix-web to Axum.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{debug, error};

use crate::application::use_cases::GetAlertsInput;
use crate::interfaces::http::response::{api_error, api_success};
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
            api_success(serde_json::to_value(alerts).unwrap_or_default())
        }
        Err(e) => {
            error!("Failed to get alerts: {}", e);
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve alerts: {}", e),
            )
        }
    }
}
