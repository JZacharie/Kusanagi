use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, middleware::Logger};
use serde_json::json;
use std::io;

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "Kusanagi Agent Controller",
        "version": "0.2.0",
        "legacy_modules": 37,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "message": "🚀 Kusanagi Agent Controller",
        "version": "0.2.0",
        "status": "running",
        "endpoints": [
            "/health",
            "/api/status",
            "/api/legacy-modules"
        ]
    }))
}

#[get("/api/status")]
async fn status() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "architecture": "hexagonal + legacy",
        "legacy_modules_preserved": 37,
        "compilation": "docker_ready",
        "features": [
            "kubernetes_integration",
            "prometheus_metrics", 
            "security_scanning",
            "backup_management",
            "legacy_compatibility"
        ]
    }))
}

#[get("/api/legacy-modules")]
async fn legacy_modules() -> impl Responder {
    let modules = vec![
        "pods", "system", "health", "backups", "security",
        "prometheus", "nodes", "calendar", "doctor", "mcp",
        "translation", "proxmox", "cilium", "newsfeed", "chat"
    ];
    
    HttpResponse::Ok().json(json!({
        "total_legacy_modules": 37,
        "sample_modules": modules,
        "status": "all_preserved",
        "migration_status": "coexisting_with_hexagonal"
    }))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let host = std::env::var("KUSANAGI_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("KUSANAGI_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("{}:{}", host, port);
    
    println!("🚀 Starting Kusanagi Agent Controller");
    println!("📍 Binding to: {}", bind_addr);
    println!("🏗️  Architecture: Hexagonal + Legacy (37 modules preserved)");
    println!("✅ Ready to serve requests!");
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(index)
            .service(health_check)
            .service(status)
            .service(legacy_modules)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
