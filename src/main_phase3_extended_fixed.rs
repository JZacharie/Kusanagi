use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use serde_json::json;
use std::{env, sync::Arc};

// Modules
mod error_simple;
mod config_simple;

// Domain
mod domain {
    pub mod entities_simple;
}

// Infrastructure
mod infrastructure {
    pub mod repositories {
        pub mod k8s_repository_real;
        pub mod prometheus_repository;
    }
}

use crate::error_simple::Result;
use crate::config_simple::Config;
use crate::infrastructure::repositories::k8s_repository_real::{K8sRepository, KubernetesRepository};
use crate::infrastructure::repositories::prometheus_repository::{PrometheusRepo, PrometheusRepository};

// Application state
#[derive(Clone)]
struct AppState {
    k8s_repo: Arc<dyn KubernetesRepository + Send + Sync>,
    prometheus_repo: Arc<dyn PrometheusRepository + Send + Sync>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Starting Kusanagi Phase 3 Extended Server...");
    
    // Initialize repositories
    let k8s_repo: Arc<dyn KubernetesRepository + Send + Sync> = Arc::new(K8sRepository::new().await);
    let prometheus_repo: Arc<dyn PrometheusRepository + Send + Sync> = Arc::new(PrometheusRepo::new());
    
    let app_state = AppState {
        k8s_repo,
        prometheus_repo,
    };
    
    let config = Config::from_env();
    let bind_addr = format!("{}:{}", config.host, config.port);
    
    println!("🌐 Server starting on {}", bind_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .route("/", web::get().to(service_info))
            .route("/health", web::get().to(health_check))
            .route("/api/cluster", web::get().to(get_cluster_overview))
            .route("/api/nodes", web::get().to(get_nodes))
            .route("/api/pods", web::get().to(get_pods))
            .route("/api/events", web::get().to(get_events))
            .route("/api/metrics", web::get().to(get_metrics))
            .route("/api/overview", web::get().to(get_combined_overview))
    })
    .bind(&bind_addr)?
    .run()
    .await
}

// Handlers
async fn service_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "0.2.0-phase3-extended",
        "description": "Kubernetes monitoring platform with Prometheus integration",
        "endpoints": [
            "GET / - Service information",
            "GET /health - Health check",
            "GET /api/cluster - Cluster overview",
            "GET /api/nodes - Node listing",
            "GET /api/pods?namespace=xxx - Pod listing (optional namespace filter)",
            "GET /api/events?namespace=xxx - Event listing (optional namespace filter)",
            "GET /api/metrics - Prometheus metrics",
            "GET /api/overview - Combined K8s + Prometheus overview"
        ]
    }))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.2.0-phase3-extended"
    }))
}

async fn get_cluster_overview(data: web::Data<AppState>) -> Result<impl Responder> {
    let overview = data.k8s_repo.get_cluster_overview().await?;
    Ok(HttpResponse::Ok().json(overview))
}

async fn get_nodes(data: web::Data<AppState>) -> Result<impl Responder> {
    let nodes = data.k8s_repo.get_nodes().await?;
    Ok(HttpResponse::Ok().json(nodes))
}

async fn get_pods(
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>
) -> Result<impl Responder> {
    let namespace = query.get("namespace").map(|s| s.as_str());
    let pods = data.k8s_repo.get_pods(namespace).await?;
    Ok(HttpResponse::Ok().json(pods))
}

async fn get_events(
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>
) -> Result<impl Responder> {
    let namespace = query.get("namespace").map(|s| s.as_str());
    let events = data.k8s_repo.get_events(namespace).await?;
    Ok(HttpResponse::Ok().json(events))
}

async fn get_metrics(data: web::Data<AppState>) -> Result<impl Responder> {
    let metrics = data.prometheus_repo.get_cluster_metrics().await?;
    Ok(HttpResponse::Ok().json(metrics))
}

async fn get_combined_overview(data: web::Data<AppState>) -> Result<impl Responder> {
    // Fetch K8s and Prometheus data in parallel
    let (cluster_result, metrics_result) = tokio::join!(
        data.k8s_repo.get_cluster_overview(),
        data.prometheus_repo.get_cluster_metrics()
    );
    
    let cluster_overview = cluster_result?;
    let prometheus_metrics = metrics_result?;
    
    let combined = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "kubernetes": cluster_overview,
        "prometheus": prometheus_metrics,
        "summary": {
            "total_nodes": cluster_overview.nodes.len(),
            "total_pods": cluster_overview.pods.len(),
            "metrics_available": !prometheus_metrics.node_metrics.is_empty() || !prometheus_metrics.pod_metrics.is_empty()
        }
    });
    
    Ok(HttpResponse::Ok().json(combined))
}
