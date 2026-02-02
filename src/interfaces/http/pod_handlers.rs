//! Pod HTTP Handlers
//!
//! HTTP interface adapters for pod operations.

use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;

use crate::application::use_cases::pod_use_cases::PodService;
use crate::error::KusanagiError;

/// Request for logs
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    pub container: Option<String>,
    pub tail: Option<i64>,
}

/// Request for scaling
#[derive(Debug, Deserialize)]
pub struct ScaleRequest {
    pub replicas: i32,
}

/// Request for force delete
#[derive(Debug, Deserialize)]
pub struct ForceDeleteRequest {
    pub namespace: String,
    pub pod_name: String,
}

/// Get pods status handler
#[get("/api/pods/status")]
pub async fn get_pods_status(service: web::Data<PodService>) -> impl Responder {
    match service.get_status.execute().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::error!("Failed to get pods status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

/// Get pod logs handler
#[get("/api/pods/{namespace}/{name}/logs")]
pub async fn get_pod_logs(
    service: web::Data<PodService>,
    path: web::Path<(String, String)>,
    query: web::Query<LogsQuery>,
) -> impl Responder {
    let (namespace, name) = path.into_inner();
    let tail = query.tail.unwrap_or(200);

    match service.get_logs.execute(&namespace, &name, query.container.clone(), tail).await {
        Ok(logs) => HttpResponse::Ok().body(logs),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// Force delete pod handler
#[post("/api/pods/force-delete")]
pub async fn force_delete_pod(
    service: web::Data<PodService>,
    body: web::Json<ForceDeleteRequest>,
) -> impl Responder {
    match service.force_delete.execute(&body.namespace, &body.pod_name).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("Pod {}/{} deleted", body.namespace, body.pod_name)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Delete error pods handler
#[post("/api/pods/delete-error-pods")]
pub async fn delete_error_pods(service: web::Data<PodService>) -> impl Responder {
    match service.delete_error_pods.execute().await {
        Ok((deleted, failed)) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "deleted": deleted,
            "failed": failed
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Scale deployment handler
#[post("/api/scale/deployment/{namespace}/{name}")]
pub async fn scale_deployment(
    service: web::Data<PodService>,
    path: web::Path<(String, String)>,
    body: web::Json<ScaleRequest>,
) -> impl Responder {
    let (namespace, name) = path.into_inner();

    match service.scale_deployment.execute(&namespace, &name, body.replicas).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("Deployment {}/{} scaled to {}", namespace, name, body.replicas)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Scale statefulset handler
#[post("/api/scale/statefulset/{namespace}/{name}")]
pub async fn scale_statefulset(
    service: web::Data<PodService>,
    path: web::Path<(String, String)>,
    body: web::Json<ScaleRequest>,
) -> impl Responder {
    let (namespace, name) = path.into_inner();

    match service.scale_statefulset.execute(&namespace, &name, body.replicas).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": format!("StatefulSet {}/{} scaled to {}", namespace, name, body.replicas)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

/// Configure routes for pod handlers
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_pods_status)
        .service(get_pod_logs)
        .service(force_delete_pod)
        .service(delete_error_pods)
        .service(scale_deployment)
        .service(scale_statefulset);
}
