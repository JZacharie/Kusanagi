use crate::domain::ports::{IntegrationRepository, SystemRepository, DatabaseRepository};
use crate::application::use_cases::*;
use crate::infrastructure::repositories::{LegacyIntegrationRepository, LegacySystemRepository, LegacyDatabaseRepository};
use actix_web::{get, web, HttpResponse, Responder};
use std::sync::Arc;

/// Integration handlers using clean architecture
pub struct IntegrationHandlers {
    integration_repo: Arc<dyn IntegrationRepository>,
    system_repo: Arc<dyn SystemRepository>,
    database_repo: Arc<dyn DatabaseRepository>,
}

impl IntegrationHandlers {
    pub fn new() -> Self {
        Self {
            integration_repo: Arc::new(LegacyIntegrationRepository),
            system_repo: Arc::new(LegacySystemRepository),
            database_repo: Arc::new(LegacyDatabaseRepository),
        }
    }
}

// MQTT endpoints
#[get("/api/mqtt/stats")]
async fn get_mqtt_stats(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.integration_repo.get_mqtt_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

// Home Assistant endpoints
#[get("/api/homeassistant/sensors")]
async fn get_ha_sensors(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.integration_repo.get_ha_sensors().await {
        Ok(sensors) => HttpResponse::Ok().json(sensors),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/homeassistant/devices")]
async fn get_ha_devices(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.integration_repo.get_ha_devices().await {
        Ok(devices) => HttpResponse::Ok().json(devices),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

// Proxmox endpoints
#[get("/api/proxmox/vms")]
async fn get_proxmox_vms(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.integration_repo.get_proxmox_vms().await {
        Ok(vms) => HttpResponse::Ok().json(vms),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/proxmox/containers")]
async fn get_proxmox_containers(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.integration_repo.get_proxmox_containers().await {
        Ok(containers) => HttpResponse::Ok().json(containers),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

// Weather endpoints
#[get("/api/weather")]
async fn get_weather(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.integration_repo.get_weather_data().await {
        Ok(weather) => HttpResponse::Ok().json(weather),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

// Calendar endpoints
#[get("/api/calendar/events")]
async fn get_calendar_events(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.integration_repo.get_calendar_events().await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

// System endpoints
#[get("/api/system/status")]
async fn get_system_status(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.system_repo.get_system_status().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/system/health")]
async fn get_system_health(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.system_repo.check_health().await {
        Ok(health) => HttpResponse::Ok().json(health),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

// Database endpoints
#[get("/api/database/health")]
async fn get_database_health(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.database_repo.check_health().await {
        Ok(health) => HttpResponse::Ok().json(health),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/database/stats")]
async fn get_database_stats(handlers: web::Data<IntegrationHandlers>) -> impl Responder {
    match handlers.database_repo.get_stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

/// Configure integration routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    let handlers = IntegrationHandlers::new();
    
    cfg.app_data(web::Data::new(handlers))
        // MQTT
        .service(get_mqtt_stats)
        // Home Assistant
        .service(get_ha_sensors)
        .service(get_ha_devices)
        // Proxmox
        .service(get_proxmox_vms)
        .service(get_proxmox_containers)
        // Weather
        .service(get_weather)
        // Calendar
        .service(get_calendar_events)
        // System
        .service(get_system_status)
        .service(get_system_health)
        // Database
        .service(get_database_health)
        .service(get_database_stats);
}
