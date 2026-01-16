use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use actix_files::Files;
use serde::Deserialize;
use tracing::info;

mod argocd;
mod cluster;
mod events;
mod nodes;

#[derive(Deserialize)]
struct SyncRequest {
    app_name: String,
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Kusanagi Agent Controller is healthy")
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

#[get("/api/argocd/status")]
async fn argocd_status() -> impl Responder {
    match argocd::get_argocd_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::error!("Failed to get ArgoCD status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[post("/api/argocd/sync")]
async fn argocd_sync(body: web::Json<SyncRequest>) -> impl Responder {
    info!("Sync requested for application: {}", body.app_name);
    
    match argocd::sync_application(&body.app_name).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("Failed to sync application {}: {}", body.app_name, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": e
            }))
        }
    }
}

#[get("/api/nodes/status")]
async fn nodes_status() -> impl Responder {
    match nodes::get_nodes_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::error!("Failed to get nodes status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cluster/overview")]
async fn cluster_overview() -> impl Responder {
    match cluster::get_cluster_overview().await {
        Ok(overview) => HttpResponse::Ok().json(overview),
        Err(e) => {
            tracing::error!("Failed to get cluster overview: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/events")]
async fn k8s_events() -> impl Responder {
    match events::get_events().await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => {
            tracing::error!("Failed to get events: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting Kusanagi server on port 8080");
    info!("Access the cyberpunk interface at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .service(health_check)
            .service(index)
            .service(argocd_status)
            .service(argocd_sync)
            .service(nodes_status)
            .service(cluster_overview)
            .service(k8s_events)
            .service(Files::new("/static", "./static").show_files_listing())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

