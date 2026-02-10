// Kusanagi - Axum Entry Point
// Migration from Actix-web to Axum

use axum::{
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};
use tracing::info;

// State - from library
use kusanagi::state::AppState;

// Handlers
pub mod api_handlers;
use api_handlers::{
    cache::cache_stats, config::get_config, health::health_check, slack::send_slack_notification,
    websocket::ws_notifications_handler,
};

// Hexagonal handlers
use kusanagi::interfaces::http::{
    alert_handlers::get_alerts_handler,
    backup_handlers::{get_backups_handler, trigger_backup_handler},
    homeassistant_handlers::{get_devices_handler, get_sensors_handler},
    security_handlers::{
        get_security_handler, get_security_report_handler, get_security_reports_handler,
        get_vulnerabilities_handler,
    },
    weather_handlers::get_weather_handler,
};

// Static files (will be served separately via tower_http)
static INDEX_HTML: &str = include_str!("../static/index.html");

// Build timestamp
const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let version = env!("CARGO_PKG_VERSION");
    info!("🚀 Kusanagi Axum Migration");
    info!("📅 Version: {}", version);
    info!("⏰ Build Time: {}", BUILD_TIMESTAMP);

    // Get bind address
    let host = std::env::var("KUSANAGI_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("KUSANAGI_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let bind_addr = format!("{}:{}", host, port);

    info!("🌐 Server binding to: {}", bind_addr);

    // Create application state
    let state = AppState::new().await?;

    // Build router
    let app = Router::new()
        // Core routes
        .route("/", get(index_handler))
        .route("/health", get(health_check))
        .route("/api", get(api_info))
        .route("/api/config", get(get_config))
        .route("/api/cache/stats", get(cache_stats))
        .route("/api/slack/notify", post(send_slack_notification))
        // WebSocket
        .route("/api/ws/notifications", get(ws_notifications_handler))
        // Hexagonal routes
        .route("/api/alerts", get(get_alerts_handler))
        .route("/api/backups", get(get_backups_handler))
        .route(
            "/api/backups/:namespace/:name/trigger",
            post(trigger_backup_handler),
        )
        .route("/api/ha/devices", get(get_devices_handler))
        .route("/api/ha/sensors", get(get_sensors_handler))
        .route("/api/security/summary", get(get_security_handler))
        .route("/api/security/reports", get(get_security_reports_handler))
        .route(
            "/api/security/reports/:category/:name",
            get(get_security_report_handler),
        )
        .route(
            "/api/security/vulnerabilities",
            get(get_vulnerabilities_handler),
        )
        .route("/api/weather/current", get(get_weather_handler))
        // Static files
        .nest_service("/static", ServeDir::new("./static"))
        // Layers
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        // State
        .with_state(state);

    // Start server
    let listener = TcpListener::bind(&bind_addr).await?;
    info!("✅ Server ready at http://{}", bind_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Serve index.html
async fn index_handler() -> impl IntoResponse {
    axum::response::Html(INDEX_HTML)
}

/// API information endpoint
async fn api_info() -> impl IntoResponse {
    Json(serde_json::json!({
        "service": "Kusanagi",
        "version": env!("CARGO_PKG_VERSION"),
        "architecture": "axum-migration",
        "features": [
            "Axum Framework",
            "Hexagonal Architecture",
            "Kubernetes Integration",
            "Cache System",
            "WebSocket Notifications"
        ],
        "endpoints": {
            "core": [
                "GET / - Web interface",
                "GET /api - API information",
                "GET /health - Health check",
                "GET /api/config - Configuration",
                "GET /api/cache/stats - Cache statistics",
                "POST /api/slack/notify - Send Slack notification",
                "GET /api/ws/notifications - WebSocket"
            ],
            "hexagonal": [
                "GET /api/alerts",
                "GET /api/backups",
                "GET /api/security/*",
                "GET /api/weather/current",
                "GET /api/ha/*"
            ]
        }
    }))
}
