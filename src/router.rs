//! Router Configuration
//!
//! Defines all API routes for the application.

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};

use crate::{
    interfaces::http::{
        alert_handlers, backup_handlers, homeassistant_handlers, security_handlers,
        weather_handlers,
    },
    state::AppState,
};

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    // API routes
    let api_routes = create_api_routes();

    // Health routes
    let health_routes = create_health_routes();

    Router::new()
        .merge(api_routes)
        .merge(health_routes)
        // Static files
        .nest_service("/static", ServeDir::new("./static"))
        // Middleware
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state)
}

/// Create API routes
fn create_api_routes() -> Router<AppState> {
    Router::new()
        // Weather
        .route("/api/weather/current", get(weather_handlers::get_weather_handler))
        .route("/api/weather/refresh", post(weather_handlers::refresh_weather_handler))
        // HomeAssistant
        .route("/api/ha/devices", get(homeassistant_handlers::get_devices_handler))
        .route("/api/ha/sensors", get(homeassistant_handlers::get_sensors_handler))
        .route("/api/ha/automations", get(homeassistant_handlers::get_automations_handler))
        .route("/api/ha/status", get(homeassistant_handlers::get_ha_status_handler))
        // Security
        .route("/api/security/summary", get(security_handlers::get_security_handler))
        .route("/api/security/vulnerabilities", get(security_handlers::get_vulnerabilities_handler))
        .route("/api/security/reports", get(security_handlers::get_security_reports_handler))
        .route("/api/security/reports/:category/:name", get(security_handlers::get_security_report_handler))
        // Alerts
        .route("/api/alerts", get(alert_handlers::get_alerts_handler))
        // Backups
        .route("/api/backups", get(backup_handlers::get_backups_handler))
        .route("/api/backups/trigger", post(backup_handlers::trigger_backup_handler))
}

/// Create health check routes
fn create_health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/", get(index_handler))
}

/// Health check handler
async fn health_check() -> &'static str {
    "OK"
}

/// Index handler - serves the dashboard
async fn index_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/index.html"))
}
