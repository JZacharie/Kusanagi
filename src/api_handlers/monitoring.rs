use actix_web::{HttpResponse, Responder};

pub async fn alerts() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"alerts": []}))
}

pub async fn quotas() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"quotas": []}))
}
