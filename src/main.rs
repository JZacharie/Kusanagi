// Kusanagi - Hexagonal Architecture Entry Point
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use actix_files;
use serde_json::json;
use std::sync::Arc;
use kusanagi::{Config, Cache, InMemoryCache, legacy};

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
