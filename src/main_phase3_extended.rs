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
        pub mod prometheus_repository;
    }
}

use crate::error::Result;
use crate::config::Config;
use crate::infrastructure::repositories::k8s_repository_real::{K8sRepository, KubernetesRepository};
use crate::infrastructure::repositories::prometheus_repository::{PrometheusRepo, PrometheusRepository};

// Application state
#[derive(Clone)]
struct AppState {
    k8s_repo: Arc<dyn KubernetesRepository + Send + Sync>,
    prometheus_repo: Arc<dyn PrometheusRepository + Send + Sync>,
}

async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "0.2.0",
        "status": "running",
        "architecture": "hexagonal",
        "phase": "3 - Extended K8s + Prometheus",
        "mode": if env::var("KUBERNETES_SERVICE_HOST").is_ok() { "kubernetes" } else { "local" },
        "features": [
            "cluster_overview",
            "nodes_listing", 
            "pods_listing",
            "events_listing",
            "prometheus_metrics"
        ]
    }))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "architecture": "hexagonal",
        "phase": "3-extended"
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

async fn get_nodes(data: web::Data<AppState>) -> impl Responder {
    match data.k8s_repo.get_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(json!({
            "nodes": nodes,
            "count": nodes.len()
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string(),
            "type": "nodes_error"
        }))
    }
}

async fn get_pods(
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>
) -> impl Responder {
    let namespace = query.get("namespace").cloned();
    
    match data.k8s_repo.get_pods(namespace.clone()).await {
        Ok(pods) => HttpResponse::Ok().json(json!({
            "pods": pods,
            "count": pods.len(),
            "namespace": namespace
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string(),
            "type": "pods_error"
        }))
    }
}

async fn get_events(
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>
) -> impl Responder {
    let namespace = query.get("namespace").cloned();
    
    match data.k8s_repo.get_events(namespace.clone()).await {
        Ok(events) => HttpResponse::Ok().json(json!({
            "events": events,
            "count": events.len(),
            "namespace": namespace
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string(),
            "type": "events_error"
        }))
    }
}

async fn get_metrics(data: web::Data<AppState>) -> impl Responder {
    match data.prometheus_repo.get_cluster_metrics().await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string(),
            "type": "metrics_error"
        }))
    }
}

async fn get_combined_overview(data: web::Data<AppState>) -> impl Responder {
    let cluster_future = data.k8s_repo.get_cluster_overview();
    let metrics_future = data.prometheus_repo.get_cluster_metrics();
    
    let (cluster_result, metrics_result) = tokio::join!(cluster_future, metrics_future);
    
    match (cluster_result, metrics_result) {
        (Ok(cluster), Ok(metrics)) => HttpResponse::Ok().json(json!({
            "cluster": cluster,
            "metrics": metrics,
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
        (cluster_result, metrics_result) => {
            let mut response = json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "partial_data": true
            });
            
            if let Ok(cluster) = cluster_result {
                response["cluster"] = json!(cluster);
            } else {
                response["cluster_error"] = json!(cluster_result.unwrap_err().to_string());
            }
            
            if let Ok(metrics) = metrics_result {
                response["metrics"] = json!(metrics);
            } else {
                response["metrics_error"] = json!(metrics_result.unwrap_err().to_string());
            }
            
            HttpResponse::Ok().json(response)
        }
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    println!("🚀 Kusanagi Phase 3 Extended - K8s + Prometheus Integration");
    
    let config = Config::load()?;
    
    let is_k8s = env::var("KUBERNETES_SERVICE_HOST").is_ok();
    if is_k8s {
        println!("☸️  Running in Kubernetes mode - attempting real API connections");
    } else {
        println!("🏠 Running in local mode - using mock data");
    }
    
    // Initialize repositories
    let k8s_repo = Arc::new(K8sRepository::new().await?);
    let prometheus_repo = Arc::new(PrometheusRepo::new());
    
    let app_state = AppState { 
        k8s_repo, 
        prometheus_repo 
    };
    
    println!("🌐 Server starting on {}:{}", config.server.host, config.server.port);
    println!("📋 Available endpoints:");
    println!("   - GET /                    : Service info");
    println!("   - GET /health              : Health check");
    println!("   - GET /api/cluster         : Cluster overview");
    println!("   - GET /api/nodes           : Node listing");
    println!("   - GET /api/pods            : Pod listing (optional ?namespace=xxx)");
    println!("   - GET /api/events          : Event listing (optional ?namespace=xxx)");
    println!("   - GET /api/metrics         : Prometheus metrics");
    println!("   - GET /api/overview        : Combined cluster + metrics");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(Logger::default())
            .route("/", web::get().to(index))
            .route("/health", web::get().to(health_check))
            .service(
                web::scope("/api")
                    .route("/cluster", web::get().to(cluster_overview))
                    .route("/nodes", web::get().to(get_nodes))
                    .route("/pods", web::get().to(get_pods))
                    .route("/events", web::get().to(get_events))
                    .route("/metrics", web::get().to(get_metrics))
                    .route("/overview", web::get().to(get_combined_overview))
                    .route("/health", web::get().to(health_check))
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await?;
    
    Ok(())
}
