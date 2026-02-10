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
    handlers::{cache::cache_stats, health::health_check, k8s::*, monitoring::*, system::*},
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
        // Kubernetes
        .route("/api/services", get(services))
        .route("/api/ingress", get(ingress))
        .route("/api/nodes/status", get(nodes_status))
        .route("/api/pods/status", get(pods_status))
        .route("/api/storage", get(storage))
        .route("/api/events", get(events))
        .route("/api/cluster/overview", get(cluster_overview))
        // Cache
        .route("/api/cache/stats", get(cache_stats))
        // System
        .route("/api/system/status", get(system_status))
        .route("/api/system/logs", get(system_logs))
        .route("/api/metrics", get(metrics))
        // Monitoring
        .route("/api/alerts", get(alerts))
        .route("/api/quotas", get(quotas))
        // Other
        .route("/api/news", get(news))
        .route("/api/news/refresh", post(refresh_news))
        .route("/api/fusion", get(fusion))
        .route("/api/mqtt/devices", get(mqtt_devices))
        .route("/api/mqtt/messages", get(mqtt_messages))
        .route("/api/argocd/status", get(argocd_status))
        .route("/api/argocd/sync", post(argocd_sync))
        .route("/api/proxmox/vms", get(proxmox_vms))
        .route("/api/proxmox/containers", get(proxmox_containers))
        .route("/api/proxmox/nodes", get(proxmox_nodes))
        .route("/api/ha/devices", get(ha_devices))
        .route("/api/ha/sensors", get(ha_sensors))
}

/// Create health check routes
fn create_health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health_check))
        .route("/", get(index_handler))
}

/// Index handler - serves the dashboard
async fn index_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/index.html"))
}

// Legacy handlers that need to be migrated
async fn services(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!([]))
}

async fn ingress(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!([]))
}

async fn nodes_status(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"ready": 0, "not_ready": 0}))
}

async fn pods_status(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"pods": [], "total": 0}))
}

async fn storage(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"total": "0GB", "used": "0GB"}))
}

async fn events(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!([]))
}

async fn cluster_overview(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({}))
}

async fn alerts(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"alerts": []}))
}

async fn quotas(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({}))
}

async fn news(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"items": []}))
}

async fn refresh_news(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"status": "success"}))
}

async fn fusion(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({}))
}

async fn mqtt_devices(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"devices": []}))
}

async fn mqtt_messages(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"messages": []}))
}

async fn argocd_status(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"status": "unknown"}))
}

async fn argocd_sync(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"status": "success"}))
}

async fn proxmox_vms(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"vms": []}))
}

async fn proxmox_containers(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"containers": []}))
}

async fn proxmox_nodes(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"nodes": []}))
}

async fn ha_devices(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"devices": []}))
}

async fn ha_sensors(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"sensors": []}))
}

async fn system_status(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

async fn system_logs(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({"logs": ""}))
}

async fn metrics(State(_state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({}))
}

use axum::response::IntoResponse;
