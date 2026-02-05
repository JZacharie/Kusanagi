// Kusanagi - Hexagonal Architecture Entry Point
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use serde_json::json;
use std::sync::Arc;
use kusanagi::{Config, Cache, InMemoryCache};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Kusanagi Hexagonal Architecture");
    
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
            .service(web::scope("/api/v1").route("/status", web::get().to(api_status)))
    })
    .bind(&bind_addr)?
    .run()
    .await
}

async fn service_info(config: web::Data<Config>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "0.2.0",
        "architecture": "hexagonal",
        "config": {
            "host": config.server.host,
            "port": config.server.port
        }
    }))
}

async fn health_check(cache: web::Data<Arc<InMemoryCache>>) -> impl Responder {
    let stats = cache.stats().await;
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "architecture": "hexagonal",
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
        "architecture": "hexagonal"
    }))
}
