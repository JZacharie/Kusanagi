// Kusanagi - Hexagonal Architecture Entry Point
use actix::{Actor, StreamHandler};
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use kusanagi::domain::services::{
    argocd_service, fusion_service, homeassistant_service, irc_service, kubernetes_service,
    monitoring_service, mqtt_service, news_service, proxmox_service, slack_service,
    trivy_service,
};
use kusanagi::{legacy, Config};
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use sysinfo::System;

// Track process startup time
static START_TIME: OnceLock<Instant> = OnceLock::new();

// Memory logging helper
fn log_memory_usage(label: &str) {
    let mut sys = System::new();
    sys.refresh_memory();
    let used_mb = sys.used_memory() as f64 / 1024.0 / 1024.0;
    let total_mb = sys.total_memory() as f64 / 1024.0 / 1024.0;
    let percent = (used_mb / total_mb) * 100.0;
    tracing::warn!(
        "🔍 RAM [{}]: {:.2} MB / {:.2} MB ({:.1}%)",
        label,
        used_mb,
        total_mb,
        percent
    );
}

// WebSocket Actor
struct WsNotifications;

impl Actor for WsNotifications {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.text(r#"{"type":"connected","message":"WebSocket connected"}"#);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsNotifications {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(_)) => {}
            Ok(ws::Message::Close(reason)) => ctx.close(reason),
            _ => {}
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize rustls crypto provider
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();

    // Configure logger with timestamps
    env_logger::Builder::from_default_env()
        .filter_module(
            "kusanagi::domain::services::proxmox_service",
            log::LevelFilter::Error,
        )
        .format_timestamp_millis()
        .init();

    println!("🚀 Kusanagi Hexagonal Architecture + Legacy");

    let version = env!("CARGO_PKG_VERSION");
    let build_time = env!("BUILD_TIMESTAMP");
    println!("📅 Version: {}", version);
    println!("⏰ Build Time: {}", build_time);

    // Initialize startup time
    START_TIME.set(Instant::now()).ok();

    let config = Config::default();

    // Advanced caches with TTL - Augmenté pour réduire les requêtes
    let k8s_cache = Arc::new(kusanagi::AdvancedCache::<String>::new(
        std::time::Duration::from_secs(60), // Augmenté de 30s à 60s
    ));
    let argocd_cache = Arc::new(kusanagi::AdvancedCache::<String>::new(
        std::time::Duration::from_secs(600), // Augmenté de 300s à 600s
    ));
    let general_cache = Arc::new(kusanagi::AdvancedCache::<String>::new(
        std::time::Duration::from_secs(120), // Augmenté de 60s à 120s
    ));

    log_memory_usage("After Config + Cache Init");

    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    println!("🌐 Server: {}", bind_addr);

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    // Initialize Kubernetes client
    let kube_client = match kube::Client::try_default().await {
        Ok(client) => {
            println!("✅ Kubernetes client initialized");
            Some(client)
        }
        Err(e) => {
            eprintln!("⚠️  Failed to initialize Kubernetes client: {}", e);
            eprintln!("   Logs endpoint will be unavailable");
            None
        }
    };

    // MQTT Init
    let mqtt_state = mqtt_service::MqttState::new();
    let mqtt_host = std::env::var("MQTT_HOST").unwrap_or_else(|_| config.mqtt.host.clone());
    let mqtt_port = std::env::var("MQTT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(config.mqtt.port);

    let mqtt_user = std::env::var("MQTT_USER").ok();
    let mqtt_password = std::env::var("MQTT_PASSWORD").ok();

    mqtt_service::start_mqtt_client(
        mqtt_state.clone(),
        mqtt_host,
        mqtt_port,
        mqtt_user,
        mqtt_password,
    );

    // Slack Monitoring Init
    let slack = slack_service::SlackService::new();
    tokio::spawn(start_slack_monitoring(slack.clone()));

    // IRC Monitoring Init
    let mut irc = irc_service::IrcService::new();
    if let Err(e) = irc.connect().await {
        eprintln!("⚠️  Failed to connect to IRC: {}", e);
    }
    tokio::spawn(start_irc_monitoring(irc, slack));

    // Start Alertmanager background cache refresh
    tokio::spawn(async {
        kusanagi::legacy::alertmanager::start_background_refresh().await;
    });

    // Start News background refresh
    tokio::spawn(async {
        println!("📰 Starting background news refresh...");
        if let Err(e) = news_service::force_refresh().await {
            eprintln!("❌ Failed to refresh news at startup: {}", e);
        } else {
            println!("✅ News refreshed and cached successfully");
        }
    });

    // Check Proxmox connectivity
    proxmox_service::check_proxmox_health(&client).await;

    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(k8s_cache.clone()))
            .app_data(web::Data::new(argocd_cache.clone()))
            .app_data(web::Data::new(general_cache.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(mqtt_state.clone()));

        // Inject Kubernetes client if available
        if let Some(ref kube_client) = kube_client {
            app = app.app_data(web::Data::new(kube_client.clone()));
        }

        app.route("/", web::get().to(web_index))
            .route("/api", web::get().to(service_info))
            .route("/health", web::get().to(health_check))
            .route("/metrics", web::get().to(prometheus_metrics))
            .route("/docs", web::get().to(web_docs))
            // API endpoints for frontend
            .route("/api/system/status", web::get().to(system_status))
            .route("/api/system/logs", web::get().to(system_logs))
            .route("/api/cache/stats", web::get().to(cache_stats))
            .route("/api/alerts", web::get().to(alerts))
            .route("/api/metrics", web::get().to(metrics))
            .route("/api/news", web::get().to(news))
            .route("/api/news/refresh", web::post().to(refresh_news))
            .route("/api/quotas", web::get().to(quotas))
            .route("/api/pods/status", web::get().to(pods_status))
            .route("/api/pods/force-delete", web::post().to(force_delete_pod))
            .route(
                "/api/pods/delete-error-pods",
                web::post().to(delete_error_pods_handler),
            )
            .route("/api/cluster/overview", web::get().to(cluster_overview))
            .route("/api/backups", web::get().to(backups))
            .route("/api/services", web::get().to(services))
            .route("/api/ingress", web::get().to(ingress))
            .route("/api/nodes/status", web::get().to(nodes_status))
            .route("/api/storage", web::get().to(storage))
            .route("/api/events", web::get().to(events))
            .route("/api/fusion", web::get().to(fusion_service::fusion_handler))
            .route("/api/mqtt/devices", web::get().to(mqtt_devices))
            .route("/api/mqtt/messages", web::get().to(mqtt_messages))
            .route("/api/argocd/status", web::get().to(argocd_status))
            .route("/api/proxmox/vms", web::get().to(proxmox_vms))
            .route("/api/proxmox/containers", web::get().to(proxmox_containers))
            .route("/api/proxmox/nodes", web::get().to(proxmox_nodes))
            .route("/api/ha/devices", web::get().to(ha_devices))
            .route("/api/ha/sensors", web::get().to(ha_sensors))
            .route("/api/ha/automations", web::get().to(ha_automations))
            .route("/status", web::get().to(system_status))
            .route("/api/logs", web::get().to(logs_endpoint))
            // Security endpoints (Trivy)
            .route(
                "/api/security/vulnerabilities",
                web::get().to(security_vulnerabilities),
            )
            .route("/api/security/reports", web::get().to(security_reports))
            .route(
                "/api/security/reports/{report_id}",
                web::get().to(security_report_by_id),
            )
            // WebSocket endpoint
            .route("/api/ws/notifications", web::get().to(websocket_handler))
            // Static files (including manifest.json) - no auth required
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
            .service(
                web::scope("/api/v1")
                    .route("/status", web::get().to(api_status))
                    // Legacy endpoints
                    .route("/legacy/cluster", web::get().to(legacy_cluster))
                    .route("/legacy/nodes", web::get().to(legacy_nodes))
                    .route("/legacy/pods", web::get().to(legacy_pods))
                    .route("/legacy/argocd", web::get().to(legacy_argocd))
                    .route("/legacy/metrics", web::get().to(legacy_metrics))
                    .route("/legacy/events", web::get().to(legacy_events))
                    .route("/legacy/services", web::get().to(legacy_services))
                    .route("/legacy/storage", web::get().to(legacy_storage))
                    .route("/legacy/ingress", web::get().to(legacy_ingress))
                    .route("/legacy/health", web::get().to(legacy_health)),
            )
    })
    .bind(&bind_addr)?
    .run()
    .await
}

async fn service_info(config: web::Data<Config>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "0.2.0",
        "architecture": "hexagonal + legacy",
        "features": [
            "Hexagonal Architecture",
            "Legacy Modules Restored",
            "Kubernetes Integration",
            "ArgoCD Support",
            "Prometheus Metrics"
        ],
        "config": {
            "host": config.server.host,
            "port": config.server.port
        },
        "endpoints": {
            "core": [
                "GET / - Kusanagi web interface",
                "GET /api - Service information",
                "GET /health - Health check",
                "GET /docs - API documentation"
            ],
            "legacy": [
                "GET /api/v1/legacy/cluster",
                "GET /api/v1/legacy/nodes",
                "GET /api/v1/legacy/pods",
                "GET /api/v1/legacy/argocd",
                "GET /api/v1/legacy/metrics",
                "GET /api/v1/legacy/events",
                "GET /api/v1/legacy/services",
                "GET /api/v1/legacy/storage",
                "GET /api/v1/legacy/ingress",
                "GET /api/v1/legacy/health"
            ]
        }
    }))
}

async fn health_check() -> impl Responder {
    // Ultra-fast health check - no blocking operations
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn api_status() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "api": "v1",
        "status": "active",
        "architecture": "hexagonal + legacy"
    }))
}

// Legacy endpoints
async fn legacy_cluster() -> impl Responder {
    match legacy::get_cluster_info().await {
        Ok(cluster) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": cluster
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn legacy_nodes() -> impl Responder {
    match legacy::get_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": nodes
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn legacy_pods() -> impl Responder {
    match legacy::get_pods().await {
        Ok(pods) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": pods
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn legacy_argocd() -> impl Responder {
    match legacy::get_applications().await {
        Ok(apps) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": apps
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn legacy_metrics() -> impl Responder {
    match legacy::get_metrics().await {
        Ok(metrics) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": metrics
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn legacy_events() -> impl Responder {
    match legacy::get_events().await {
        Ok(events) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": events
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn legacy_services() -> impl Responder {
    match legacy::get_services().await {
        Ok(services) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": services
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn legacy_storage() -> impl Responder {
    match legacy::get_storage().await {
        Ok(storage) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": storage
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn legacy_ingress() -> impl Responder {
    match legacy::get_ingresses().await {
        Ok(ingresses) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": ingresses
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn legacy_health() -> impl Responder {
    match legacy::get_health_status().await {
        Ok(health) => HttpResponse::Ok().json(json!({
            "source": "legacy",
            "data": health
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

async fn web_docs() -> impl Responder {
    match std::fs::read_to_string("./static/api-docs.html") {
        Ok(content) => HttpResponse::Ok()
            .content_type("text/html")
            .body(content),
        Err(_) => HttpResponse::Ok()
            .content_type("text/html")
            .body(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Kusanagi API Documentation</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; }
        h1 { color: #333; border-bottom: 2px solid #007acc; padding-bottom: 10px; }
        .endpoint { background: #f8f9fa; padding: 15px; margin: 10px 0; border-radius: 5px; }
        .method { color: #28a745; font-weight: bold; }
        .legacy { color: #dc3545; }
        code { background: #e9ecef; padding: 2px 5px; border-radius: 3px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 Kusanagi API Documentation</h1>
        <p><strong>Architecture:</strong> Hexagonal + Legacy</p>
        <p><strong>Version:</strong> 0.2.0</p>
        
        <h2>Core Endpoints</h2>
        <div class="endpoint">
            <span class="method">GET</span> <code>/</code> - Service information
        </div>
        <div class="endpoint">
            <span class="method">GET</span> <code>/health</code> - Health check
        </div>
        <div class="endpoint">
            <span class="method">GET</span> <code>/docs</code> - This documentation
        </div>
        
        <h2>Legacy Endpoints</h2>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/cluster</code> - Cluster information
        </div>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/nodes</code> - Node status
        </div>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/pods</code> - Pod information
        </div>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/argocd</code> - ArgoCD applications
        </div>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/metrics</code> - Prometheus metrics
        </div>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/events</code> - Cluster events
        </div>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/services</code> - Kubernetes services
        </div>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/storage</code> - Storage volumes
        </div>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/ingress</code> - Ingress controllers
        </div>
        <div class="endpoint">
            <span class="method legacy">GET</span> <code>/api/v1/legacy/health</code> - Component health
        </div>
        
        <h2>Static Files</h2>
        <div class="endpoint">
            <span class="method">GET</span> <code>/static/*</code> - Static file serving
        </div>
        
        <p><em>Total: 13 endpoints (3 core + 10 legacy)</em></p>
    </div>
</body>
</html>
            "#)
    }
}

async fn web_index() -> impl Responder {
    match std::fs::read_to_string("./static/index.html") {
        Ok(content) => HttpResponse::Ok().content_type("text/html").body(content),
        Err(_) => match std::fs::read_to_string("/app/static/index.html") {
            Ok(content) => HttpResponse::Ok().content_type("text/html").body(content),
            Err(_) => HttpResponse::NotFound().json(json!({
                "error": "Index page not found"
            })),
        },
    }
}

// Prometheus metrics endpoint
async fn prometheus_metrics(
    k8s_cache: web::Data<Arc<kusanagi::AdvancedCache<String>>>,
    argocd_cache: web::Data<Arc<kusanagi::AdvancedCache<String>>>,
    general_cache: web::Data<Arc<kusanagi::AdvancedCache<String>>>,
) -> impl Responder {
    let uptime_secs = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);

    // Read process stats
    let (memory_mb, cpu_usage) = match tokio::fs::read_to_string("/proc/self/stat").await {
        Ok(stat) => {
            let fields: Vec<&str> = stat.split_whitespace().collect();
            let utime = fields
                .get(13)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let stime = fields
                .get(14)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let total_time = utime + stime;
            let cpu_percent = if uptime_secs > 0 {
                (total_time as f64 / 100.0) / uptime_secs as f64 * 100.0
            } else {
                0.0
            };

            let mem = tokio::fs::read_to_string("/proc/self/status")
                .await
                .ok()
                .and_then(|status| {
                    status
                        .lines()
                        .find(|line| line.starts_with("VmRSS:"))
                        .and_then(|line| line.split_whitespace().nth(1))
                        .and_then(|kb| kb.parse::<f64>().ok())
                        .map(|kb_val| kb_val / 1024.0)
                })
                .unwrap_or(0.0);

            (mem, cpu_percent.min(100.0))
        }
        Err(_) => (0.0, 0.0),
    };

    // Get cache stats
    let k8s_stats = k8s_cache.stats().await;
    let argocd_stats = argocd_cache.stats().await;
    let general_stats = general_cache.stats().await;

    // Prometheus text format
    let metrics = format!(
        "# HELP kusanagi_uptime_seconds Kusanagi uptime in seconds\n\
         # TYPE kusanagi_uptime_seconds gauge\n\
         kusanagi_uptime_seconds {}\n\
         # HELP kusanagi_memory_usage_mb Kusanagi memory usage in MB\n\
         # TYPE kusanagi_memory_usage_mb gauge\n\
         kusanagi_memory_usage_mb {:.2}\n\
         # HELP kusanagi_cpu_usage_percent Kusanagi CPU usage percentage\n\
         # TYPE kusanagi_cpu_usage_percent gauge\n\
         kusanagi_cpu_usage_percent {:.2}\n\
         # HELP kusanagi_cache_entries Cache entries by type\n\
         # TYPE kusanagi_cache_entries gauge\n\
         kusanagi_cache_entries{{type=\"k8s\"}} {}\n\
         kusanagi_cache_entries{{type=\"argocd\"}} {}\n\
         kusanagi_cache_entries{{type=\"general\"}} {}\n\
         # HELP kusanagi_cache_expired Expired cache entries by type\n\
         # TYPE kusanagi_cache_expired gauge\n\
         kusanagi_cache_expired{{type=\"k8s\"}} {}\n\
         kusanagi_cache_expired{{type=\"argocd\"}} {}\n\
         kusanagi_cache_expired{{type=\"general\"}} {}\n\
         # HELP kusanagi_cache_memory_bytes Cache memory usage by type\n\
         # TYPE kusanagi_cache_memory_bytes gauge\n\
         kusanagi_cache_memory_bytes{{type=\"k8s\"}} {}\n\
         kusanagi_cache_memory_bytes{{type=\"argocd\"}} {}\n\
         kusanagi_cache_memory_bytes{{type=\"general\"}} {}\n\
         # HELP kusanagi_build_info Kusanagi build information\n\
         # TYPE kusanagi_build_info gauge\n\
         kusanagi_build_info{{version=\"0.2.0\",build_timestamp=\"{}\"}} 1\n",
        uptime_secs,
        memory_mb,
        cpu_usage,
        k8s_stats.entries,
        argocd_stats.entries,
        general_stats.entries,
        k8s_stats.expired,
        argocd_stats.expired,
        general_stats.expired,
        k8s_stats.memory_bytes,
        argocd_stats.memory_bytes,
        general_stats.memory_bytes,
        env!("BUILD_TIMESTAMP")
    );

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics)
}

// API endpoints for frontend
async fn system_status() -> impl Responder {
    // Lightweight - only Kusanagi process metrics
    let uptime_secs = START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0);

    // Read Kusanagi process memory and CPU from /proc/self/status
    let (memory_mb, cpu_usage) = match tokio::fs::read_to_string("/proc/self/stat").await {
        Ok(stat) => {
            let fields: Vec<&str> = stat.split_whitespace().collect();
            let utime = fields
                .get(13)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let stime = fields
                .get(14)
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let total_time = utime + stime;
            let cpu_percent = if uptime_secs > 0 {
                (total_time as f64 / 100.0) / uptime_secs as f64 * 100.0
            } else {
                0.0
            };

            let mem = tokio::fs::read_to_string("/proc/self/status")
                .await
                .ok()
                .and_then(|status| {
                    status
                        .lines()
                        .find(|line| line.starts_with("VmRSS:"))
                        .and_then(|line| line.split_whitespace().nth(1))
                        .and_then(|kb| kb.parse::<f64>().ok())
                        .map(|kb_val| kb_val / 1024.0)
                })
                .unwrap_or(0.0);

            (mem, cpu_percent.min(100.0))
        }
        Err(_) => (0.0, 0.0),
    };

    HttpResponse::Ok().json(json!({
        "status": "operational",
        "uptime_secs": uptime_secs,
        "uptime": format!("{}h {}m", uptime_secs / 3600, (uptime_secs % 3600) / 60),
        "version": "0.2.0",
        "build_timestamp": env!("BUILD_TIMESTAMP"),
        "cpu_usage": cpu_usage,
        "memory_usage": memory_mb,
        "memory_usage_mb": memory_mb,
        "memory_total_mb": 0.0
    }))
}

async fn system_logs(client: Option<web::Data<kube::Client>>) -> impl Responder {
    use k8s_openapi::api::core::v1::Pod;
    use kube::api::LogParams;
    use kube::Api;

    // Check if Kubernetes client is available
    let Some(kube_client) = client else {
        let error_msg = "=== Kubernetes Client Unavailable ===\n\n\
                        The Kubernetes client could not be initialized.\n\
                        This usually means Kusanagi is running outside a Kubernetes cluster.\n\n\
                        Logs are only available when running inside Kubernetes.";
        return HttpResponse::Ok().body(error_msg);
    };

    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "kusanagi".to_string());
    let namespace = std::env::var("POD_NAMESPACE").unwrap_or_else(|_| "kusanagi".to_string());

    let pods: Api<Pod> = Api::namespaced(kube_client.get_ref().clone(), &namespace);

    match pods
        .logs(
            &hostname,
            &LogParams {
                tail_lines: Some(1000),
                ..Default::default()
            },
        )
        .await
    {
        Ok(logs) => HttpResponse::Ok().body(logs),
        Err(_) => {
            let error_msg = format!(
                "=== Kusanagi Logs Unavailable ===\n\n\
                Could not fetch logs from pod {} in namespace {}\n\n\
                Try: kubectl logs -n {} {}",
                hostname, namespace, namespace, hostname
            );
            HttpResponse::Ok().body(error_msg)
        }
    }
}

async fn metrics() -> impl Responder {
    // Stubbed - System removed to save 7GB RAM
    HttpResponse::Ok().json(json!({
        "cpu_load": 0.0,
        "memory_usage": 0,
        "disk_usage": 0
    }))
}

async fn logs_endpoint() -> impl Responder {
    // Get logs from kubectl (last 100 lines) - async version
    let output = tokio::process::Command::new("kubectl")
        .args([
            "logs",
            "-n",
            "kusanagi",
            "-l",
            "app.kubernetes.io/name=kusanagi",
            "--tail=100",
        ])
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            let logs = String::from_utf8_lossy(&result.stdout).to_string();
            HttpResponse::Ok().json(json!({
                "logs": logs
            }))
        }
        _ => HttpResponse::Ok().json(json!({
            "logs": "Unable to fetch logs"
        })),
    }
}

// Endpoints mockés temporairement
async fn alerts() -> impl Responder {
    match monitoring_service::get_alerts().await {
        Ok(alerts) => {
            let alerts_array = alerts.as_array().unwrap_or(&vec![]).clone();

            // Group alerts by severity
            let mut critical = vec![];
            let mut warning = vec![];
            let mut info = vec![];

            for alert in &alerts_array {
                let severity = alert
                    .get("severity")
                    .and_then(|s| s.as_str())
                    .unwrap_or("info");

                match severity {
                    "critical" => critical.push(alert.clone()),
                    "warning" => warning.push(alert.clone()),
                    _ => info.push(alert.clone()),
                }
            }

            HttpResponse::Ok().json(json!({
                "total": alerts_array.len(),
                "critical": critical,
                "warning": warning,
                "info": info,
                "status": "success"
            }))
        }
        Err(_) => HttpResponse::Ok().json(json!({
            "total": 0,
            "critical": [],
            "warning": [],
            "info": [],
            "status": "error"
        })),
    }
}

async fn cache_stats(
    k8s_cache: web::Data<Arc<kusanagi::AdvancedCache<String>>>,
    argocd_cache: web::Data<Arc<kusanagi::AdvancedCache<String>>>,
    general_cache: web::Data<Arc<kusanagi::AdvancedCache<String>>>,
) -> impl Responder {
    let k8s = k8s_cache.stats().await;
    let argocd = argocd_cache.stats().await;
    let general = general_cache.stats().await;

    HttpResponse::Ok().json(json!({
        "k8s": {
            "entries": k8s.entries,
            "expired": k8s.expired,
            "memory_bytes": k8s.memory_bytes,
            "ttl_seconds": 60
        },
        "argocd": {
            "entries": argocd.entries,
            "expired": argocd.expired,
            "memory_bytes": argocd.memory_bytes,
            "ttl_seconds": 600
        },
        "general": {
            "entries": general.entries,
            "expired": general.expired,
            "memory_bytes": general.memory_bytes,
            "ttl_seconds": 120
        },
        "total": {
            "entries": k8s.entries + argocd.entries + general.entries,
            "expired": k8s.expired + argocd.expired + general.expired,
            "memory_bytes": k8s.memory_bytes + argocd.memory_bytes + general.memory_bytes
        }
    }))
}

async fn news() -> impl Responder {
    match news_service::get_news().await {
        Ok(news) => HttpResponse::Ok().json(news),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn refresh_news() -> impl Responder {
    match news_service::force_refresh().await {
        Ok(news) => HttpResponse::Ok().json(json!({
            "status": "success",
            "message": "News refreshed successfully",
            "items": news["items"]
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "status": "error",
            "message": e
        })),
    }
}

async fn quotas() -> impl Responder {
    match monitoring_service::get_quotas().await {
        Ok(quotas) => HttpResponse::Ok().json(quotas),
        Err(_) => HttpResponse::Ok().json(json!({"used": 50, "total": 100})),
    }
}

async fn pods_status() -> impl Responder {
    match kubernetes_service::get_pods_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(_) => HttpResponse::Ok().json(json!({
            "running": 0, "pending": 0, "failed": 0, "total": 0,
            "total_pods": 0, "running_pods": 0, "error_pods": 0, "pods_in_error": []
        })),
    }
}

#[derive(Deserialize)]
struct DeletePodRequest {
    namespace: String,
    pod_name: String,
}

async fn force_delete_pod(params: web::Json<DeletePodRequest>) -> impl Responder {
    match kubernetes_service::delete_pod(&params.namespace, &params.pod_name).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::InternalServerError().json(json!({"success": false, "message": e})),
    }
}

async fn delete_error_pods_handler() -> impl Responder {
    match kubernetes_service::delete_error_pods().await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::InternalServerError().json(json!({"success": false, "message": e})),
    }
}

async fn cluster_overview() -> impl Responder {
    match kubernetes_service::get_cluster_overview().await {
        Ok(overview) => HttpResponse::Ok().json(overview),
        Err(_) => HttpResponse::Ok().json(json!({"nodes": 0, "pods": 0, "services": 0})),
    }
}

async fn backups() -> impl Responder {
    match monitoring_service::get_backups().await {
        Ok(backups) => HttpResponse::Ok().json(backups),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn services() -> impl Responder {
    match kubernetes_service::get_services().await {
        Ok(services) => HttpResponse::Ok().json(services),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn ingress() -> impl Responder {
    match kubernetes_service::get_ingress().await {
        Ok(ingress) => HttpResponse::Ok().json(ingress),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn nodes_status() -> impl Responder {
    match kubernetes_service::get_nodes_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(_) => HttpResponse::Ok().json(json!({"ready": 0, "not_ready": 0})),
    }
}

async fn storage() -> impl Responder {
    match kubernetes_service::get_storage().await {
        Ok(storage) => HttpResponse::Ok().json(storage),
        Err(_) => HttpResponse::Ok().json(json!({"total": "0GB", "used": "0GB"})),
    }
}

async fn events() -> impl Responder {
    match kubernetes_service::get_events().await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn argocd_status() -> impl Responder {
    match argocd_service::get_argocd_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(_) => HttpResponse::Ok().json(json!({"healthy": false, "apps": 0})),
    }
}

async fn proxmox_vms(client: web::Data<reqwest::Client>) -> impl Responder {
    match proxmox_service::get_proxmox_vms(&client).await {
        Ok(vms) => HttpResponse::Ok().json(vms),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn proxmox_containers(client: web::Data<reqwest::Client>) -> impl Responder {
    match proxmox_service::get_proxmox_containers(&client).await {
        Ok(containers) => HttpResponse::Ok().json(containers),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn proxmox_nodes(client: web::Data<reqwest::Client>) -> impl Responder {
    match proxmox_service::get_proxmox_nodes(&client).await {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn ha_devices() -> impl Responder {
    match homeassistant_service::get_ha_devices().await {
        Ok(devices) => {
            let devices_array = devices.as_array().unwrap_or(&vec![]).clone();
            HttpResponse::Ok().json(json!({
                "devices": devices_array,
                "data": devices_array,
                "count": devices_array.len(),
                "status": "success",
                "total": devices_array.len(),
                "online": 0,
                "offline": 0
            }))
        }
        Err(_) => HttpResponse::Ok().json(json!({
            "devices": [],
            "data": [],
            "count": 0,
            "status": "no_ha",
            "total": 0,
            "online": 0,
            "offline": 0
        })),
    }
}

async fn ha_sensors() -> impl Responder {
    match homeassistant_service::get_ha_sensors().await {
        Ok(sensors) => HttpResponse::Ok().json(sensors),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn ha_automations() -> impl Responder {
    match homeassistant_service::get_ha_automations().await {
        Ok(automations) => HttpResponse::Ok().json(automations),
        Err(_) => HttpResponse::Ok().json(json!([])),
    }
}

async fn websocket_handler(req: HttpRequest, stream: web::Payload) -> impl Responder {
    ws::start(WsNotifications, &req, stream)
}

async fn mqtt_devices(state: web::Data<mqtt_service::MqttState>) -> impl Responder {
    HttpResponse::Ok().json(state.get_devices())
}

async fn mqtt_messages(state: web::Data<mqtt_service::MqttState>) -> impl Responder {
    HttpResponse::Ok().json(state.get_messages())
}

// Slack Monitoring Background Task
async fn start_slack_monitoring(slack: slack_service::SlackService) {
    use std::time::Duration;

    let mut last_error_pods = 0u64;
    let mut last_unhealthy_apps = 0u64;

    // Wait 30s before starting to allow services to initialize
    tokio::time::sleep(Duration::from_secs(30)).await;

    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;

        let prev_error_pods = last_error_pods;
        let prev_unhealthy_apps = last_unhealthy_apps;

        let mut pods_checked = false;
        let mut apps_checked = false;

        // Check Pods
        if let Ok(pods_status) = kubernetes_service::get_pods_status().await {
            pods_checked = true;
            let error_pods = pods_status["error_pods"].as_u64().unwrap_or(0);

            if error_pods > last_error_pods {
                let mut message = format!("Detected {} pods in error state:\n", error_pods);

                if let Some(pods_in_error) = pods_status["pods_in_error"].as_array() {
                    for pod in pods_in_error.iter().take(10) {
                        let name = pod["name"].as_str().unwrap_or("unknown");
                        let namespace = pod["namespace"].as_str().unwrap_or("unknown");
                        let status = pod["status"].as_str().unwrap_or("unknown");
                        let reason = pod["reason"].as_str().unwrap_or("Unknown");
                        message.push_str(&format!(
                            "• *{}/{}*: {} ({})\n",
                            namespace, name, status, reason
                        ));
                    }

                    if error_pods > 10 {
                        message.push_str(&format!("...and {} more.", error_pods - 10));
                    }
                }

                let _ = slack
                    .send_alert("Infrastructure Issue", &message, "error")
                    .await;
            }
            last_error_pods = error_pods;
        }

        // Check ArgoCD
        if let Ok(argocd_status) = argocd_service::get_argocd_status().await {
            apps_checked = true;
            let unhealthy = argocd_status["unhealthy"].as_u64().unwrap_or(0);

            if unhealthy > last_unhealthy_apps {
                let mut message = format!("Detected {} unhealthy applications:\n", unhealthy);

                if let Some(apps) = argocd_status["apps_with_issues"].as_array() {
                    for app in apps.iter().take(10) {
                        let name = app["name"].as_str().unwrap_or("unknown");
                        let health = app["health_status"].as_str().unwrap_or("unknown");
                        let sync = app["sync_status"].as_str().unwrap_or("unknown");
                        message.push_str(&format!("• *{}*: {} ({})\n", name, health, sync));
                    }

                    if unhealthy > 10 {
                        message.push_str(&format!("...and {} more.", unhealthy - 10));
                    }
                }

                let _ = slack
                    .send_alert("ArgoCD Sync Alert", &message, "warning")
                    .await;
            }
            last_unhealthy_apps = unhealthy;
        }

        // Check for recovery
        if pods_checked && apps_checked {
            let was_unhealthy = prev_error_pods > 0 || prev_unhealthy_apps > 0;
            let is_now_healthy = last_error_pods == 0 && last_unhealthy_apps == 0;

            if was_unhealthy && is_now_healthy {
                let _ = slack
                    .send_alert(
                        "System Recovered",
                        "All pods and applications are now healthy! 🎉",
                        "success",
                    )
                    .await;
            }
        }
    }
}

// IRC Monitoring Background Task - mirrors Slack alerts to IRC
async fn start_irc_monitoring(
    irc: irc_service::IrcService,
    slack: slack_service::SlackService,
) {
    use std::time::Duration;

    let mut last_error_pods = 0u64;
    let mut last_unhealthy_apps = 0u64;

    // Wait 30s before starting to allow services to initialize
    tokio::time::sleep(Duration::from_secs(30)).await;

    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;

        let prev_error_pods = last_error_pods;
        let prev_unhealthy_apps = last_unhealthy_apps;

        let mut pods_checked = false;
        let mut apps_checked = false;

        // Check Pods
        if let Ok(pods_status) = kubernetes_service::get_pods_status().await {
            pods_checked = true;
            let error_pods = pods_status["error_pods"].as_u64().unwrap_or(0);

            if error_pods > last_error_pods {
                let mut message = format!("Detected {} pods in error state:\n", error_pods);

                if let Some(pods_in_error) = pods_status["pods_in_error"].as_array() {
                    for pod in pods_in_error.iter().take(10) {
                        let name = pod["name"].as_str().unwrap_or("unknown");
                        let namespace = pod["namespace"].as_str().unwrap_or("unknown");
                        let status = pod["status"].as_str().unwrap_or("unknown");
                        let reason = pod["reason"].as_str().unwrap_or("Unknown");
                        message.push_str(&format!(
                            "• {}/{}: {} ({})\n",
                            namespace, name, status, reason
                        ));
                    }

                    if error_pods > 10 {
                        message.push_str(&format!("...and {} more.", error_pods - 10));
                    }
                }

                // Send to both Slack and IRC
                let _ = slack
                    .send_alert("Infrastructure Issue", &message, "error")
                    .await;
                let _ = irc
                    .send_alert("Infrastructure Issue", &message, "error")
                    .await;
            }
            last_error_pods = error_pods;
        }

        // Check ArgoCD
        if let Ok(argocd_status) = argocd_service::get_argocd_status().await {
            apps_checked = true;
            let unhealthy = argocd_status["unhealthy"].as_u64().unwrap_or(0);

            if unhealthy > last_unhealthy_apps {
                let mut message = format!("Detected {} unhealthy applications:\n", unhealthy);

                if let Some(apps) = argocd_status["apps_with_issues"].as_array() {
                    for app in apps.iter().take(10) {
                        let name = app["name"].as_str().unwrap_or("unknown");
                        let health = app["health_status"].as_str().unwrap_or("unknown");
                        let sync = app["sync_status"].as_str().unwrap_or("unknown");
                        message.push_str(&format!("• {}: {} ({})\n", name, health, sync));
                    }

                    if unhealthy > 10 {
                        message.push_str(&format!("...and {} more.", unhealthy - 10));
                    }
                }

                // Send to both Slack and IRC
                let _ = slack
                    .send_alert("ArgoCD Sync Alert", &message, "warning")
                    .await;
                let _ = irc
                    .send_alert("ArgoCD Sync Alert", &message, "warning")
                    .await;
            }
            last_unhealthy_apps = unhealthy;
        }

        // Check for recovery
        if pods_checked && apps_checked {
            let was_unhealthy = prev_error_pods > 0 || prev_unhealthy_apps > 0;
            let is_now_healthy = last_error_pods == 0 && last_unhealthy_apps == 0;

            if was_unhealthy && is_now_healthy {
                let recovery_msg = "All pods and applications are now healthy! 🎉";
                let _ = slack
                    .send_alert("System Recovered", recovery_msg, "success")
                    .await;
                let _ = irc
                    .send_alert("System Recovered", recovery_msg, "success")
                    .await;
            }
        }
    }
}


// Security endpoints (Trivy)
async fn security_vulnerabilities() -> impl Responder {
    match trivy_service::get_vulnerabilities().await {
        Ok(vulns) => HttpResponse::Ok().json(vulns),
        Err(e) => {
            tracing::debug!("Trivy vulnerabilities unavailable: {}", e);
            HttpResponse::Ok().json(json!({
                "critical": 0,
                "high": 0,
                "medium": 0,
                "low": 0,
                "total": 0,
                "images": [],
                "error": e
            }))
        }
    }
}

async fn security_reports() -> impl Responder {
    match trivy_service::list_reports().await {
        Ok(reports) => HttpResponse::Ok().json(reports),
        Err(e) => {
            tracing::debug!("Trivy reports unavailable: {}", e);
            HttpResponse::Ok().json(json!({
                "reports": [],
                "total": 0,
                "error": e
            }))
        }
    }
}

async fn security_report_by_id(path: web::Path<String>) -> impl Responder {
    let report_id = path.into_inner();
    match trivy_service::get_report_by_id(&report_id).await {
        Ok(report) => HttpResponse::Ok().json(report),
        Err(e) => {
            tracing::warn!("Failed to fetch report {}: {}", report_id, e);
            HttpResponse::NotFound().json(json!({
                "error": format!("Report not found: {}", e)
            }))
        }
    }
}
