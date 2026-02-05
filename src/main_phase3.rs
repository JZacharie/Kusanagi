use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use serde_json::json;
use std::{env, sync::Arc};

// Modules
mod error_simple as error;
mod config_simple as config;

// Domain
mod domain {
    pub mod entities_simple;
}

// Infrastructure
mod infrastructure {
    pub mod repositories {
        pub mod k8s_repository_real;
    }
}

use crate::error::Result;
use crate::config::Config;
use crate::infrastructure::repositories::k8s_repository_real::{K8sRepository, KubernetesRepository};

// Application state
struct AppState {
    k8s_repo: Arc<dyn KubernetesRepository + Send + Sync>,
}

async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "0.2.0",
        "status": "running",
        "architecture": "hexagonal",
        "phase": "3 - Real K8s Integration",
        "mode": if env::var("KUBERNETES_SERVICE_HOST").is_ok() { "kubernetes" } else { "local" }
    }))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "architecture": "hexagonal",
        "phase": 3
    }))
}

async fn cluster_overview(data: web::Data<AppState>) -> impl Responder {
    match data.k8s_repo.get_cluster_overview().await {
        Ok(overview) => HttpResponse::Ok().json(overview),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string(),
            "type": "cluster_overview_error"
        }))
    }
}

async fn cluster_nodes(data: web::Data<AppState>) -> impl Responder {
    // TODO: Implement node listing
    HttpResponse::Ok().json(json!({
        "message": "Node listing not yet implemented",
        "status": "todo"
    }))
}

async fn cluster_pods(data: web::Data<AppState>) -> impl Responder {
    // TODO: Implement pod listing
    HttpResponse::Ok().json(json!({
        "message": "Pod listing not yet implemented", 
        "status": "todo"
    }))
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    println!("🚀 Kusanagi Phase 3 - Real K8s Integration");
    
    let config = Config::load()?;
    
    let is_k8s = env::var("KUBERNETES_SERVICE_HOST").is_ok();
    if is_k8s {
        println!("☸️  Running in Kubernetes mode - attempting real API connection");
    } else {
        println!("🏠 Running in local mode - using mock data");
    }
    
    // Initialize K8s repository
    let k8s_repo = Arc::new(K8sRepository::new().await?);
    let app_state = AppState { k8s_repo };
    
    println!("🌐 Server starting on {}:{}", config.server.host, config.server.port);
    println!("📋 Available endpoints:");
    println!("   - GET /              : Service info");
    println!("   - GET /health        : Health check");
    println!("   - GET /api/cluster   : Cluster overview (REAL K8s API)");
    println!("   - GET /api/nodes     : Node listing (TODO)");
    println!("   - GET /api/pods      : Pod listing (TODO)");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api")
                    .route("/cluster", web::get().to(cluster_overview))
                    .route("/nodes", web::get().to(cluster_nodes))
                    .route("/pods", web::get().to(cluster_pods))
                    .route("/health", web::get().to(health_check))
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await?;
    
    Ok(())
}
