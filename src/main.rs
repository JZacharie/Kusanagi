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
    cache::cache_stats, config::get_config, database::database_health_handler, docs::docs_handler,
    health::health_check, prometheus::prometheus_range_handler, slack::send_slack_notification,
    websocket::ws_notifications_handler,
};
use kusanagi::domain::services::fusion_service::fusion_handler;
use kusanagi::handlers::{
    k8s::{
        argocd_status, cluster_overview, delete_error_pods_handler, ingress, nodes_status,
        pod_logs, pods_status, services, storage,
    },
    monitoring::{alerts, metrics_handler, quotas},
    system::{news, system_logs, system_status},
};

use kusanagi::interfaces::http::chat_handlers::post_chat_handler;

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
    // Initialize tracing with file appender - Minutely rotation for 15m retention
    // Use /tmp as it's usually writable in containers
    let log_dir = "/tmp/kusanagi-logs";

    // Check if we can write to the log directory
    let file_appender = match std::fs::create_dir_all(log_dir) {
        Ok(_) => {
            // Create a placeholder file to ensure the directory is not empty
            // This prevents system_logs from failing before the first log rotation/flush
            let init_file = std::path::Path::new(log_dir).join("kusanagi.log.0000-init");
            if let Err(e) = std::fs::write(&init_file, "Initializing Kusanagi logs...\n") {
                eprintln!("Failed to create init log file: {}", e);
            }

            let appender = tracing_appender::rolling::minutely(log_dir, "kusanagi.log");
            Some(tracing_appender::non_blocking(appender))
        }
        Err(e) => {
            eprintln!(
                "⚠️ Failed to create log directory '{}': {}. File logging disabled.",
                log_dir, e
            );
            None
        }
    };

    let (non_blocking, _guard) = match file_appender {
        Some((nb, guard)) => (Some(nb), Some(guard)),
        None => (None, None),
    };

    // Spawn background task to clean up old logs ONLY if file logging is enabled
    if _guard.is_some() {
        tokio::spawn(async move {
            // Wait a bit before first cleanup
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

            loop {
                if let Ok(entries) = std::fs::read_dir(log_dir) {
                    let now = std::time::SystemTime::now();
                    let retention_period = std::time::Duration::from_secs(15 * 60); // 15 minutes

                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() {
                            // Check if file name starts with kusanagi.log
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                if name.starts_with("kusanagi.log") {
                                    // Check modification time
                                    if let Ok(metadata) = std::fs::metadata(&path) {
                                        if let Ok(modified) = metadata.modified() {
                                            if let Ok(age) = now.duration_since(modified) {
                                                if age > retention_period {
                                                    if let Err(e) = std::fs::remove_file(&path) {
                                                        eprintln!(
                                                            "Failed to delete old log {}: {}",
                                                            name, e
                                                        );
                                                    } else {
                                                        // Use println instead of tracing to avoid recursive logging issues if we are purging
                                                        println!("Purged old log file: {}", name);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Check every minute
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }

    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "kusanagi=debug,tower_http=debug,axum=debug".into());

    let stdout_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    // We can't conditionally add a layer type nicely without boxing or Option,
    // but tracing-subscriber allows an Option<Layer>.
    // Let's create the file layer as an Option.
    let file_layer = if let Some(nb) = non_blocking {
        Some(
            tracing_subscriber::fmt::layer()
                .with_writer(nb)
                .with_ansi(false),
        )
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
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
        .route("/docs", get(docs_handler))
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
        .route("/api/chat", post(post_chat_handler))
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
    let routes = api_handlers::docs::get_routes();
    let mut endpoints: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();

    for route in routes {
        endpoints
            .entry(route.category.to_lowercase())
            .or_default()
            .push(format!(
                "{} {} - {}",
                route.method, route.path, route.description
            ));
    }

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
        "endpoints": endpoints
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
