// Kusanagi - Hexagonal Architecture Entry Point
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use actix_files;
use serde_json::json;
use std::sync::Arc;
use kusanagi::{Config, Cache, InMemoryCache, legacy};
use kusanagi::domain::services::{kubernetes_service, monitoring_service, argocd_service, proxmox_service, news_service, homeassistant_service};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Kusanagi Hexagonal Architecture + Legacy");
    
    let config = Config::default();
    let cache = Arc::new(InMemoryCache::new());
    
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);
    println!("🌐 Server: {}", bind_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
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
            .route("/api/argocd/status", web::get().to(argocd_status))
            .route("/api/proxmox/vms", web::get().to(proxmox_vms))
            .route("/api/proxmox/containers", web::get().to(proxmox_containers))
            .route("/api/proxmox/nodes", web::get().to(proxmox_nodes))
            .route("/api/ha/devices", web::get().to(ha_devices))
            .route("/api/ha/sensors", web::get().to(ha_sensors))
            .route("/api/ha/automations", web::get().to(ha_automations))
            .route("/status", web::get().to(system_status))
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

async fn health_check(cache: web::Data<Arc<InMemoryCache>>) -> impl Responder {
    let stats = cache.stats().await;
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "architecture": "hexagonal + legacy",
        "legacy_modules": [
            "cluster", "nodes", "pods", "argocd", "prometheus", "events", "services", "storage", "ingress", "health"
        ],
        "cache": {
            "entries": stats.entries,
            "hits": stats.hits,
            "misses": stats.misses
        }
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
async fn system_status() -> impl Responder {
    let uptime = std::fs::read_to_string("/proc/uptime")
        .unwrap_or_default()
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .map(|s| format!("{}h", (s / 3600.0) as u32))
        .unwrap_or_else(|| "unknown".to_string());
    
    HttpResponse::Ok().json(json!({
        "status": "operational",
        "uptime": uptime,
        "version": "0.2.0"
    }))
}

async fn metrics() -> impl Responder {
    let loadavg = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let load = loadavg.split_whitespace().next().unwrap_or("0.0").parse::<f64>().unwrap_or(0.0);
    
    let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    let mut total_mem = 0;
    let mut free_mem = 0;
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            total_mem = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            free_mem = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        }
    }
    let memory_usage = if total_mem > 0 { ((total_mem - free_mem) * 100 / total_mem) } else { 0 };
    
    HttpResponse::Ok().json(json!({
        "cpu_load": (load * 100.0) as u32,
        "memory_usage": memory_usage,
        "disk_usage": 23
    }))
}

// Endpoints mockés temporairement
async fn alerts() -> impl Responder {
    match monitoring_service::get_alerts().await {
        Ok(alerts) => HttpResponse::Ok().json(alerts),
        Err(_) => HttpResponse::Ok().json(json!([]))
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
        Ok(status) => HttpResponse::Ok().json(status),
        Err(_) => HttpResponse::Ok().json(json!({"running": 0, "pending": 0, "failed": 0}))
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

async fn proxmox_vms() -> impl Responder {
    match proxmox_service::get_proxmox_vms().await {
        Ok(vms) => HttpResponse::Ok().json(vms),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn proxmox_containers() -> impl Responder {
    match proxmox_service::get_proxmox_containers().await {
        Ok(containers) => HttpResponse::Ok().json(containers),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn proxmox_nodes() -> impl Responder {
    match proxmox_service::get_proxmox_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(_) => HttpResponse::Ok().json(json!([]))
    }
}

async fn ha_devices() -> impl Responder {
    match homeassistant_service::get_ha_devices().await {
        Ok(devices) => HttpResponse::Ok().json(devices),
        Err(_) => HttpResponse::Ok().json(json!([]))
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
