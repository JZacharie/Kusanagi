use actix_web::{get, web, HttpResponse, Responder};
use std::sync::Arc;

#[get("/api/system/status")]
async fn get_system_status() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

#[get("/api/mqtt/stats")]
async fn get_mqtt_stats() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

#[get("/api/homeassistant/sensors")]
async fn get_ha_sensors() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"sensors": []}))
}

#[get("/api/weather")]
async fn get_weather() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"weather": "sunny"}))
}

#[get("/api/health")]
async fn get_health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({"status": "healthy"}))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_system_status)
        .service(get_mqtt_stats)
        .service(get_ha_sensors)
        .service(get_weather)
        .service(get_health);
}
