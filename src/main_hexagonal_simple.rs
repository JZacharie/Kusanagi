use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde_json::json;
use std::env;

async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "0.2.0",
        "status": "running",
        "architecture": "hexagonal",
        "mode": if env::var("KUBERNETES_SERVICE_HOST").is_ok() { "kubernetes" } else { "local" }
    }))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "architecture": "hexagonal"
    }))
}

async fn cluster_overview() -> impl Responder {
    let is_k8s = env::var("KUBERNETES_SERVICE_HOST").is_ok();
    
    if is_k8s {
        HttpResponse::Ok().json(json!({
            "cluster_name": "kubernetes",
            "node_count": 0,
            "pod_count": 0,
            "namespace_count": 0,
            "healthy_nodes": 0,
            "running_pods": 0,
            "status": "Connected (TODO: Implement K8s API)"
        }))
    } else {
        HttpResponse::Ok().json(json!({
            "cluster_name": "local-mock",
            "node_count": 1,
            "pod_count": 5,
            "namespace_count": 3,
            "healthy_nodes": 1,
            "running_pods": 5,
            "status": "Healthy (Mock Data)"
        }))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🚀 Kusanagi starting (Hexagonal Architecture)...");
    
    let host = env::var("KUSANAGI_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("KUSANAGI_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    
    let is_k8s = env::var("KUBERNETES_SERVICE_HOST").is_ok();
    if is_k8s {
        println!("☸️  Running in Kubernetes mode");
    } else {
        println!("🏠 Running in local mode - services will be mocked");
    }
    
    println!("🌐 Server starting on {}:{}", host, port);
    println!("📋 Available endpoints:");
    println!("   - GET /              : Service info");
    println!("   - GET /health        : Health check");
    println!("   - GET /api/cluster   : Cluster overview");
    
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api")
                    .route("/cluster", web::get().to(cluster_overview))
                    .route("/health", web::get().to(health_check))
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
