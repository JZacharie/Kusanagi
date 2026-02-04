use actix_web::{get, web, HttpResponse, Responder};
use std::sync::Arc;

#[get("/api/system/status")]
async fn get_system_status() -> impl Responder {
    // Delegate to integration handlers
    crate::interfaces::http::integration_handlers::get_system_status(
        web::Data::new(crate::interfaces::http::integration_handlers::IntegrationHandlers::new())
    ).await
}

#[get("/api/mqtt/stats")]
async fn get_mqtt_stats() -> impl Responder {
    // Delegate to integration handlers
    crate::interfaces::http::integration_handlers::get_mqtt_stats(
        web::Data::new(crate::interfaces::http::integration_handlers::IntegrationHandlers::new())
    ).await
}

#[get("/api/homeassistant/sensors")]
async fn get_ha_sensors() -> impl Responder {
    // Delegate to integration handlers
    crate::interfaces::http::integration_handlers::get_ha_sensors(
        web::Data::new(crate::interfaces::http::integration_handlers::IntegrationHandlers::new())
    ).await
}

#[get("/api/weather")]
async fn get_weather() -> impl Responder {
    // Delegate to integration handlers
    crate::interfaces::http::integration_handlers::get_weather(
        web::Data::new(crate::interfaces::http::integration_handlers::IntegrationHandlers::new())
    ).await
}

#[get("/api/health")]
async fn get_health() -> impl Responder {
    // Delegate to integration handlers
    crate::interfaces::http::integration_handlers::get_system_health(
        web::Data::new(crate::interfaces::http::integration_handlers::IntegrationHandlers::new())
    ).await
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_system_status)
        .service(get_mqtt_stats)
        .service(get_ha_sensors)
        .service(get_weather)
        .service(get_health);
}
