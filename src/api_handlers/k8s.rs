use actix_web::{HttpResponse, Responder};

pub async fn cluster_overview() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

pub async fn nodes_status() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"nodes": []}))
}

pub async fn pods_status() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"pods": []}))
}
