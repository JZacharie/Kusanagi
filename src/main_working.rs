use actix_web::{get, web, App, HttpServer, Responder, HttpResponse, middleware::Logger};
use serde_json::json;
use std::io;

// Import des modules corrigés
use kusanagi::legacy::health;

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi Agent Controller",
        "version": "0.2.0",
        "status": "running",
        "legacy_modules": 37,
        "architecture": "hexagonal + legacy"
    }))
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "Kusanagi",
        "legacy_modules_preserved": 37
    }))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let host = std::env::var("KUSANAGI_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("KUSANAGI_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("{}:{}", host, port);
    
    println!("🚀 Kusanagi Agent Controller Starting");
    println!("📍 Binding to: {}", bind_addr);
    println!("🏗️  Legacy modules: 37 preserved");
    println!("✅ Corrections applied - Ready to serve!");
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(index)
            .service(health_check)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
