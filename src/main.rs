// Kusanagi - Hexagonal Architecture Entry Point
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
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
            .route("/", web::get().to(service_info))
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api/v1")
                    .route("/status", web::get().to(api_status))
                    // Legacy endpoints
                    .route("/legacy/cluster", web::get().to(legacy_cluster))
                    .route("/legacy/nodes", web::get().to(legacy_nodes))
                    .route("/legacy/pods", web::get().to(legacy_pods))
                    .route("/legacy/argocd", web::get().to(legacy_argocd))
                    .route("/legacy/metrics", web::get().to(legacy_metrics))
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
                "GET /",
                "GET /health",
                "GET /api/v1/status"
            ],
            "legacy": [
                "GET /api/v1/legacy/cluster",
                "GET /api/v1/legacy/nodes", 
                "GET /api/v1/legacy/pods",
                "GET /api/v1/legacy/argocd",
                "GET /api/v1/legacy/metrics"
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
            "cluster", "nodes", "pods", "argocd", "prometheus"
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
