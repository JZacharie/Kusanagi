use actix_web::{HttpResponse, Responder};
use serde_json::json;

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "kusanagi",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn service_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "name": "Kusanagi",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Kubernetes & Infrastructure Monitoring Platform",
        "build_timestamp": env!("BUILD_TIMESTAMP")
    }))
}
