use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse};
use actix_files::Files;
use serde::Deserialize;
use tracing::info;

mod apps;
mod argocd;
mod backups;
mod chat;
mod cluster;
mod events;
mod nodes;
mod storage;
mod chat_storage;
mod mcp;
mod services;
mod ingress;
mod pods;
mod cilium;
mod ws;

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

#[get("/api/apps")]
async fn apps_with_resources() -> impl Responder {
    match apps::get_apps_with_resources().await {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(e) => {
            tracing::error!("Failed to get apps with resources: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[post("/api/chat")]
async fn chat_endpoint(body: web::Json<chat::ChatRequest>) -> impl Responder {
    info!("Chat message: {}", body.message);
    let response = chat::process_message(body.into_inner()).await;
    HttpResponse::Ok().json(response)
}

#[get("/api/backups")]
async fn backups_status() -> impl Responder {
    match backups::get_backups_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::error!("Failed to get backups status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/storage")]
async fn storage_status() -> impl Responder {
    match storage::get_storage_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::error!("Failed to get storage status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/services")]
async fn services_status() -> impl Responder {
    match services::get_services().await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => {
            tracing::error!("Failed to get services info: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/ingress")]
async fn ingress_status() -> impl Responder {
    match ingress::get_ingresses().await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => {
            tracing::error!("Failed to get ingress info: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/pods/status")]
async fn pods_status() -> impl Responder {
    match pods::get_pods_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::error!("Failed to get pods status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[derive(Deserialize)]
struct CiliumQuery {
    namespace: Option<String>,
    limit: Option<usize>,
    format: Option<String>,
}

#[get("/api/cilium/flows")]
async fn cilium_flows(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    let limit = query.limit.unwrap_or(100);
    
    match cilium::get_hubble_flows(namespace, limit).await {
        Ok(flows) => HttpResponse::Ok().json(flows),
        Err(e) => {
            tracing::error!("Failed to get Cilium flows: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cilium/matrix")]
async fn cilium_matrix(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    
    match cilium::get_flow_matrix(namespace).await {
        Ok(matrix) => HttpResponse::Ok().json(matrix),
        Err(e) => {
            tracing::error!("Failed to get flow matrix: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cilium/metrics")]
async fn cilium_metrics(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    
    match cilium::get_bandwidth_metrics(namespace).await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => {
            tracing::error!("Failed to get bandwidth metrics: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cilium/anomalies")]
async fn cilium_anomalies(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    
    match cilium::detect_anomalies(namespace).await {
        Ok(anomalies) => HttpResponse::Ok().json(anomalies),
        Err(e) => {
            tracing::error!("Failed to detect anomalies: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cilium/export")]
async fn cilium_export(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    let limit = query.limit.unwrap_or(1000);
    let format = query.format.as_deref().unwrap_or("json");
    
    match cilium::get_hubble_flows(namespace, limit).await {
        Ok(flows) => {
            match format {
                "csv" => HttpResponse::Ok()
                    .content_type("text/csv")
                    .insert_header(("Content-Disposition", "attachment; filename=flows.csv"))
                    .body(cilium::export_flows_csv(&flows)),
                _ => HttpResponse::Ok()
                    .content_type("application/json")
                    .insert_header(("Content-Disposition", "attachment; filename=flows.json"))
                    .body(cilium::export_flows_json(&flows)),
            }
        }
        Err(e) => {
            tracing::error!("Failed to export flows: {}", e);
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
            .service(apps_with_resources)
            .service(chat_endpoint)
            .service(backups_status)
            .service(storage_status)
            .service(services_status)
            .service(ingress_status)
            .service(pods_status)
            .service(cilium_flows)
            .service(cilium_matrix)
            .service(cilium_metrics)
            .service(cilium_anomalies)
            .service(cilium_export)
            .route("/ws/notifications", web::get().to(ws::ws_notifications))
            .service(Files::new("/static", "./static").show_files_listing())
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
