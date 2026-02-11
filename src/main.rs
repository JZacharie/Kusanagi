// Kusanagi - Axum Entry Point
// Migration from Actix-web to Axum

use axum::{
    extract::Request,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tokio::net::TcpListener;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};
use tracing::info;

// State - from library
use kusanagi::state::AppState;

// Handlers
pub mod api_handlers;
// Declare submodules in main.rs if they are not in mod.rs of api_handlers
// But usually they should be in api_handlers/mod.rs
// I will check api_handlers/mod.rs next.

use api_handlers::{
    cache::cache_stats, config::get_config, database::database_health_handler,
    health::health_check, prometheus::prometheus_range_handler, slack::send_slack_notification,
    websocket::ws_notifications_handler,
};
use kusanagi::domain::services::fusion_service::fusion_handler;
use kusanagi::handlers::{
    k8s::{
        argocd_status, cluster_overview, delete_error_pods_handler, ingress, nodes_status,
        pod_logs, pods_status, services, storage,
    },
    monitoring::{alerts, quotas},
    system::{news, system_logs, system_status},
};

// Hexagonal handlers
use kusanagi::interfaces::http::{
    alert_handlers::get_alerts_handler,
    backup_handlers::{get_backups_handler, trigger_backup_handler},
    homeassistant_handlers::{get_automations_handler, get_devices_handler, get_sensors_handler},
    proxmox_handlers::{get_containers_handler, get_nodes_handler, get_vms_handler},
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
    // Install rustls crypto provider (required for rustls 0.23+)
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

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
            "/api/backups/{namespace}/{name}/trigger",
            post(trigger_backup_handler),
        )
        .route("/api/ha/devices", get(get_devices_handler))
        .route("/api/ha/sensors", get(get_sensors_handler))
        .route("/api/ha/automations", get(get_automations_handler))
        .route("/api/security/summary", get(get_security_handler))
        .route("/api/security/reports", get(get_security_reports_handler))
        .route(
            "/api/security/reports/{category}/{name}",
            get(get_security_report_handler),
        )
        .route(
            "/api/security/vulnerabilities",
            get(get_vulnerabilities_handler),
        )
        .route("/api/weather/current", get(get_weather_handler))
        // System routes
        .route("/api/system/status", get(system_status))
        .route("/api/system/logs", get(system_logs))
        // Kubernetes routes
        .route("/api/k8s/cluster", get(cluster_overview))
        .route("/api/k8s/nodes", get(nodes_status))
        .route("/api/k8s/pods", get(pods_status))
        .route("/api/k8s/pods/:namespace/:name/logs", get(pod_logs))
        .route(
            "/api/pods/delete-error-pods",
            post(delete_error_pods_handler),
        )
        .route("/api/storage", get(storage))
        .route("/api/ingress", get(ingress))
        .route("/api/services", get(services))
        .route("/api/argocd/status", get(argocd_status))
        .route("/api/news", get(news))
        // Legacy/Expected Kubernetes routes
        .route("/api/cluster/overview", get(cluster_overview))
        .route("/api/nodes/status", get(nodes_status))
        .route("/api/pods/status", get(pods_status))
        // Monitoring routes
        .route("/api/monitoring/alerts", get(alerts))
        .route("/api/monitoring/quotas", get(quotas))
        .route("/api/quotas", get(quotas)) // Alias for frontend
        .route("/api/metrics", get(metrics_handler))
        .route("/metrics", get(metrics_handler))
        .route("/api/prometheus/range", get(prometheus_range_handler))
        .route("/api/database/health", get(database_health_handler))
        .route("/api/fusion", get(fusion_handler))
        // Proxmox routes
        .route("/api/proxmox/vms", get(get_vms_handler))
        .route("/api/proxmox/containers", get(get_containers_handler))
        .route("/api/proxmox/nodes", get(get_nodes_handler))
        // Static files (doit être après les routes API)
        .nest_service("/static", ServeDir::new("./static"))
        // Layers (appliqués dans l'ordre inverse - le dernier est exécuté en premier)
        .layer(middleware::from_fn(log_request))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        // State (doit être en dernier)
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
            "system": [
                "GET /api/system/status",
                "GET /api/system/logs"
            ],
            "kubernetes": [
                "GET /api/k8s/cluster",
                "GET /api/k8s/nodes",
                "GET /api/k8s/pods",
                "GET /api/storage",
                "GET /api/ingress"
            ],
            "monitoring": [
                "GET /api/monitoring/alerts",
                "GET /api/monitoring/quotas",
                "GET /api/metrics"
            ],
            "fusion": [
                "GET /api/fusion"
            ],
            "hexagonal": [
                "GET /api/alerts",
                "GET /api/backups",
                "GET /api/security/*",
                "GET /api/weather/current",
                "GET /api/ha/*"
            ],
            "proxmox": [
                "GET /api/proxmox/vms",
                "GET /api/proxmox/containers",
                "GET /api/proxmox/nodes"
            ],
            "homeassistant": [
                "GET /api/ha/devices",
                "GET /api/ha/sensors",
                "GET /api/ha/automations"
            ]
        }
    }))
}

/// Metrics endpoint
async fn metrics_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_seconds": 0,
        "system": {
            "cpu_usage": 0.0,
            "memory_usage": 0,
            "memory_total": 0
        },
        "kubernetes": {
            "pods_total": 0,
            "pods_running": 0,
            "nodes_total": 0
        }
    }))
}

/// Middleware pour logger les requêtes reçues
async fn log_request(request: Request, next: Next) -> impl IntoResponse {
    let method = request.method().clone();
    let uri = request.uri().clone();

    info!("📥 {} {}", method, uri);

    let response = next.run(request).await;

    info!("📤 {} - Status: {}", uri, response.status());

    response
}
