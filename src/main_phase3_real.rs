use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use serde_json::json;
use std::{env, sync::Arc};

// Modules
mod error_simple;
mod config_simple;
mod domain {
    pub mod entities_simple;
}
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

#[derive(Clone)]
struct AppState {
    k8s_repo: Arc<dyn KubernetesRepository + Send + Sync>,
    prometheus_repo: Arc<dyn PrometheusRepository + Send + Sync>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Starting Kusanagi Phase 3 Real Integration...");
    
    // Initialize repositories with real implementations
    let k8s_repo: Arc<dyn KubernetesRepository + Send + Sync> = Arc::new(K8sRepository::new().await);
    let prometheus_repo: Arc<dyn PrometheusRepository + Send + Sync> = Arc::new(PrometheusRepo::new());
    
    let app_state = AppState { k8s_repo, prometheus_repo };
    
    let config = Config::new();
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

async fn service_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "0.3.0-real-integration",
        "description": "Kubernetes monitoring with real K8s API and Prometheus integration",
        "endpoints": [
            "GET / - Service information",
            "GET /health - Health check", 
            "GET /api/cluster - Real cluster overview",
            "GET /api/nodes - Real node listing",
            "GET /api/pods?namespace=xxx - Real pod listing",
            "GET /api/events?namespace=xxx - Real event listing",
            "GET /api/metrics - Real Prometheus metrics",
            "GET /api/overview - Combined real data"
        ]
    }))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.3.0-real-integration"
    }))
}

async fn get_cluster_overview(data: web::Data<AppState>) -> impl Responder {
    match data.k8s_repo.get_cluster_overview().await {
        Ok(overview) => HttpResponse::Ok().json(overview),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get cluster overview",
            "details": e.to_string()
        }))
    }
}

async fn get_nodes(data: web::Data<AppState>) -> impl Responder {
    match data.k8s_repo.get_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get nodes",
            "details": e.to_string()
        }))
    }
}

async fn get_pods(
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>
) -> impl Responder {
    let namespace = query.get("namespace").map(|s| s.clone());
    match data.k8s_repo.get_pods(namespace).await {
        Ok(pods) => HttpResponse::Ok().json(pods),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get pods",
            "details": e.to_string()
        }))
    }
}

async fn get_events(
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>
) -> impl Responder {
    let namespace = query.get("namespace").map(|s| s.clone());
    match data.k8s_repo.get_events(namespace).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get events",
            "details": e.to_string()
        }))
    }
}

async fn get_metrics(data: web::Data<AppState>) -> impl Responder {
    match data.prometheus_repo.get_cluster_metrics().await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": "Failed to get metrics",
            "details": e.to_string()
        }))
    }
}

async fn get_combined_overview(data: web::Data<AppState>) -> impl Responder {
    let (cluster_result, metrics_result) = tokio::join!(
        data.k8s_repo.get_cluster_overview(),
        data.prometheus_repo.get_cluster_metrics()
    );
    
    match (cluster_result, metrics_result) {
        (Ok(cluster), Ok(metrics)) => {
            HttpResponse::Ok().json(json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "kubernetes": cluster,
                "prometheus": metrics,
                "summary": {
                    "total_nodes": cluster.nodes.len(),
                    "total_pods": cluster.pods.len(),
                    "metrics_available": !metrics.node_metrics.is_empty()
                }
            }))
        },
        (Err(k8s_err), Ok(metrics)) => {
            HttpResponse::Ok().json(json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "kubernetes": {"error": k8s_err.to_string()},
                "prometheus": metrics,
                "summary": {"k8s_connection": "failed", "prometheus_connection": "ok"}
            }))
        },
        (Ok(cluster), Err(prom_err)) => {
            HttpResponse::Ok().json(json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "kubernetes": cluster,
                "prometheus": {"error": prom_err.to_string()},
                "summary": {"k8s_connection": "ok", "prometheus_connection": "failed"}
            }))
        },
        (Err(k8s_err), Err(prom_err)) => {
            HttpResponse::Ok().json(json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "kubernetes": {"error": k8s_err.to_string()},
                "prometheus": {"error": prom_err.to_string()},
                "summary": {"k8s_connection": "failed", "prometheus_connection": "failed"}
            }))
        }
    }
}
