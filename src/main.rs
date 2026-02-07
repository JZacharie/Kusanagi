// Kusanagi - Hexagonal Architecture Entry Point
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger, HttpRequest};
use actix_files;
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, ActorContext};
use serde_json::json;
use std::sync::Arc;
use kusanagi::{Config, Cache, InMemoryCache, legacy};
use kusanagi::domain::services::{kubernetes_service, monitoring_service, argocd_service, proxmox_service, news_service, homeassistant_service, mqtt_service, slack_service};
use sysinfo::{System, Networks, CpuRefreshKind, MemoryRefreshKind, Disks};
use std::sync::Mutex;

// Memory logging helper
fn log_memory_usage(label: &str) {
    let mut sys = System::new();
    sys.refresh_memory();
    let used_mb = sys.used_memory() as f64 / 1024.0 / 1024.0;
    let total_mb = sys.total_memory() as f64 / 1024.0 / 1024.0;
    let percent = (used_mb / total_mb) * 100.0;
    tracing::warn!("🔍 RAM [{}]: {:.2} MB / {:.2} MB ({:.1}%)", label, used_mb, total_mb, percent);
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
            Ok(ws::Message::Text(_)) => {},
            Ok(ws::Message::Close(reason)) => ctx.close(reason),
            _ => {}
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Configure logger with timestamps
    env_logger::Builder::from_default_env()
        .format_timestamp_millis()
        .init();
    
    println!("🚀 Kusanagi Hexagonal Architecture + Legacy");
    
    let config = Config::default();
    let cache = Arc::new(InMemoryCache::new());
    log_memory_usage("After Config + Cache Init");
    
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    println!("🌐 Server: {}", bind_addr);
    
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    // Cache warming removed - cache disabled to prevent memory leaks
    // Data is fetched fresh on each request now

    let sys = web::Data::new(Mutex::new(System::new()));

    // MQTT Init
    let mqtt_state = mqtt_service::MqttState::new();
    let mqtt_host = std::env::var("MQTT_HOST").unwrap_or_else(|_| config.mqtt.host.clone());
    let mqtt_port = std::env::var("MQTT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(config.mqtt.port);
    
    let mqtt_user = std::env::var("MQTT_USER").ok();
    let mqtt_password = std::env::var("MQTT_PASSWORD").ok();
    
    mqtt_service::start_mqtt_client(mqtt_state.clone(), mqtt_host, mqtt_port, mqtt_user, mqtt_password);

    // Slack Monitoring Init
    let slack = slack_service::SlackService::new();
    tokio::spawn(start_slack_monitoring(slack));

    HttpServer::new(move || {
        App::new()
            .app_data(sys.clone())
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(mqtt_state.clone()))
            .route("/", web::get().to(web_index))
            .route("/api", web::get().to(service_info))
            .route("/health", web::get().to(health_check))
            .route("/docs", web::get().to(web_docs))
            // API endpoints for frontend
            .route("/api/system/status", web::get().to(system_status))
            .route("/api/alerts", web::get().to(alerts))
            .route("/api/metrics", web::get().to(metrics))
            .route("/api/news", web::get().to(news))
            .route("/api/quotas", web::get().to(quotas))
            .route("/api/pods/status", web::get().to(pods_status))
            .route("/api/cluster/overview", web::get().to(cluster_overview))
            .route("/api/backups", web::get().to(backups))
            .route("/api/services", web::get().to(services))
            .route("/api/ingress", web::get().to(ingress))
            .route("/api/nodes/status", web::get().to(nodes_status))
            .route("/api/storage", web::get().to(storage))
            .route("/api/events", web::get().to(events))
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
                    .route("/legacy/health", web::get().to(legacy_health))
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
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "architecture": "hexagonal + legacy",
        "legacy_modules": [
            "cluster", "nodes", "pods", "argocd", "prometheus", "events", "services", "storage", "ingress", "health"
        ]
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
        }))
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
        }))
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
        }))
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
        }))
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
        }))
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
        }))
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
        }))
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
        }))
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
        }))
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
        }))
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
        Ok(content) => HttpResponse::Ok()
            .content_type("text/html")
            .body(content),
        Err(_) => match std::fs::read_to_string("/app/static/index.html") {
            Ok(content) => HttpResponse::Ok()
                .content_type("text/html")
                .body(content),
            Err(_) => HttpResponse::NotFound().json(json!({
                "error": "Index page not found"
            }))
        }
    }
}

// API endpoints for frontend
async fn system_status(sys: web::Data<Mutex<System>>) -> impl Responder {
    let mut sys = sys.lock().unwrap();
    sys.refresh_all();

    let uptime = System::uptime();
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    
    // Convert to MB
    let memory_usage_mb = used_mem as f64 / 1024.0 / 1024.0;
    
    HttpResponse::Ok().json(json!({
        "status": "operational",
        "uptime_secs": uptime,
        "uptime": format!("{}h", uptime / 3600),
        "version": "0.2.0",
        "cpu_usage": cpu_usage,
        "memory_usage_mb": memory_usage_mb,
        "memory_total_mb": total_mem as f64 / 1024.0 / 1024.0
    }))
}

async fn metrics(sys: web::Data<Mutex<System>>) -> impl Responder {
    let mut sys = sys.lock().unwrap();
    sys.refresh_all();
    
    let load = sys.global_cpu_info().cpu_usage(); // Use cpu usage as load approx or use sys.load_average() if available?
    // sysinfo 0.30 removed load_average() from SystemExt? It's usually in SystemExt.
    // Let's stick to using what we have.
    
    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let memory_usage = if total_mem > 0 { (used_mem * 100) / total_mem } else { 0 };

    // Calculate real disk usage
    let disks = Disks::new_with_refreshed_list();
    let mut total_disk = 0;
    let mut available_disk = 0;
    
    for disk in &disks {
        total_disk += disk.total_space();
        available_disk += disk.available_space();
    }
    
    let used_disk = total_disk - available_disk;
    let disk_usage = if total_disk > 0 { (used_disk * 100) / total_disk } else { 0 };

    HttpResponse::Ok().json(json!({
        "cpu_load": load,
        "memory_usage": memory_usage,
        "disk_usage": disk_usage
    }))
}

// Endpoints mockés temporairement
async fn alerts() -> impl Responder {
    match monitoring_service::get_alerts().await {
        Ok(alerts) => {
            let alerts_array = alerts.as_array().unwrap_or(&vec![]).clone();
            HttpResponse::Ok().json(json!({
                "alerts": alerts_array,
                "data": alerts_array,
                "count": alerts_array.len(),
                "status": "success"
            }))
        },
        Err(_) => HttpResponse::Ok().json(json!({
            "alerts": [],
            "data": [],
            "count": 0,
            "status": "error"
        }))
    }
}

async fn news() -> impl Responder {
    match news_service::get_news().await {
        Ok(news) => HttpResponse::Ok().json(news),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn quotas() -> impl Responder {
    match monitoring_service::get_quotas().await {
        Ok(quotas) => HttpResponse::Ok().json(quotas),
        Err(_) => HttpResponse::Ok().json(json!({"used": 50, "total": 100}))
    }
}

async fn pods_status() -> impl Responder {
    match kubernetes_service::get_pods_status().await {
        Ok(status) => {
            let running = status["running"].as_u64().unwrap_or(0);
            let pending = status["pending"].as_u64().unwrap_or(0);
            let failed = status["failed"].as_u64().unwrap_or(0);
            let total = status["total"].as_u64().unwrap_or(0);
            
            HttpResponse::Ok().json(json!({
                "running": running,
                "pending": pending,
                "failed": failed,
                "total": total,
                // Champs attendus par le frontend
                "total_pods": total,
                "running_pods": running,
                "error_pods": failed,
                "pods_in_error": failed
            }))
        },
        Err(_) => HttpResponse::Ok().json(json!({
            "running": 0, "pending": 0, "failed": 0, "total": 0,
            "total_pods": 0, "running_pods": 0, "error_pods": 0, "pods_in_error": []
        }))
    }
}

async fn cluster_overview() -> impl Responder {
    match kubernetes_service::get_cluster_overview().await {
        Ok(overview) => HttpResponse::Ok().json(overview),
        Err(_) => HttpResponse::Ok().json(json!({"nodes": 0, "pods": 0, "services": 0}))
    }
}

async fn backups() -> impl Responder {
    match monitoring_service::get_backups().await {
        Ok(backups) => HttpResponse::Ok().json(backups),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn services() -> impl Responder {
    match kubernetes_service::get_services().await {
        Ok(services) => HttpResponse::Ok().json(services),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn ingress() -> impl Responder {
    match kubernetes_service::get_ingress().await {
        Ok(ingress) => HttpResponse::Ok().json(ingress),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn nodes_status() -> impl Responder {
    match kubernetes_service::get_nodes_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(_) => HttpResponse::Ok().json(json!({"ready": 0, "not_ready": 0}))
    }
}

async fn storage() -> impl Responder {
    match kubernetes_service::get_storage().await {
        Ok(storage) => HttpResponse::Ok().json(storage),
        Err(_) => HttpResponse::Ok().json(json!({"total": "0GB", "used": "0GB"}))
    }
}

async fn events() -> impl Responder {
    match kubernetes_service::get_events().await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn argocd_status() -> impl Responder {
    match argocd_service::get_argocd_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(_) => HttpResponse::Ok().json(json!({"healthy": false, "apps": 0}))
    }
}

async fn proxmox_vms(client: web::Data<reqwest::Client>) -> impl Responder {
    match proxmox_service::get_proxmox_vms(&client).await {
        Ok(vms) => HttpResponse::Ok().json(vms),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn proxmox_containers(client: web::Data<reqwest::Client>) -> impl Responder {
    match proxmox_service::get_proxmox_containers(&client).await {
        Ok(containers) => HttpResponse::Ok().json(containers),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn proxmox_nodes(client: web::Data<reqwest::Client>) -> impl Responder {
    match proxmox_service::get_proxmox_nodes(&client).await {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(_) => HttpResponse::Ok().json(json!([]))
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
        },
        Err(_) => HttpResponse::Ok().json(json!({
            "devices": [],
            "data": [],
            "count": 0,
            "status": "no_ha",
            "total": 0,
            "online": 0,
            "offline": 0
        }))
    }
}

async fn ha_sensors() -> impl Responder {
    match homeassistant_service::get_ha_sensors().await {
        Ok(sensors) => HttpResponse::Ok().json(sensors),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn ha_automations() -> impl Responder {
    match homeassistant_service::get_ha_automations().await {
        Ok(automations) => HttpResponse::Ok().json(automations),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn websocket_stub() -> impl Responder {
    HttpResponse::NotImplemented().json(json!({
        "error": "WebSocket not implemented",
        "message": "WebSocket notifications endpoint not available"
    }))
}

async fn websocket_handler(req: HttpRequest, stream: web::Payload) -> impl Responder {
    ws::start(WsNotifications, &req, stream)
}

async fn manifest_handler() -> impl Responder {
    match std::fs::read_to_string("./static/manifest.json") {
        Ok(content) => HttpResponse::Ok()
            .content_type("application/json")
            .body(content),
        Err(_) => HttpResponse::Ok()
            .content_type("application/json")
            .json(json!({
                "name": "Kusanagi",
                "short_name": "Kusanagi",
                "description": "Kubernetes Monitoring Platform",
                "start_url": "/",
                "display": "standalone",
                "background_color": "#0a0f1e",
                "theme_color": "#0a0f1e",
                "icons": []
            }))
    }
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
                        message.push_str(&format!("• *{}/{}*: {} ({})\n", namespace, name, status, reason));
                    }
                    
                    if error_pods > 10 {
                        message.push_str(&format!("...and {} more.", error_pods - 10));
                    }
                }
                
                let _ = slack.send_alert("Infrastructure Issue", &message, "error").await;
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
                
                let _ = slack.send_alert("ArgoCD Sync Alert", &message, "warning").await;
            }
            last_unhealthy_apps = unhealthy;
        }
        
        // Check for recovery
        if pods_checked && apps_checked {
            let was_unhealthy = prev_error_pods > 0 || prev_unhealthy_apps > 0;
            let is_now_healthy = last_error_pods == 0 && last_unhealthy_apps == 0;
            
            if was_unhealthy && is_now_healthy {
                let _ = slack.send_alert(
                    "System Recovered",
                    "All pods and applications are now healthy! 🎉",
                    "success"
                ).await;
            }
        }
    }
}

