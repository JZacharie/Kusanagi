// HTTP Controllers - Interface Layer
use crate::application::use_cases::ClusterUseCase;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

pub mod alert_handlers;
pub mod backup_handlers;
pub mod homeassistant_handlers;
pub mod security_handlers;
pub mod weather_handlers;

pub async fn get_cluster_info(cluster_use_case: web::Data<Arc<ClusterUseCase>>) -> impl Responder {
    match cluster_use_case.get_cluster_status().await {
        Ok(cluster) => HttpResponse::Ok().json(cluster),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_nodes_info(cluster_use_case: web::Data<Arc<ClusterUseCase>>) -> impl Responder {
    match cluster_use_case.list_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

// Re-export handlers
pub use alert_handlers::{configure_alert_routes, create_alerts_use_case};
pub use backup_handlers::{configure_backup_routes, create_backup_use_case};
pub use homeassistant_handlers::{configure_ha_routes, create_homeassistant_use_case};
pub use security_handlers::{configure_security_routes, create_security_use_case};
pub use weather_handlers::{configure_weather_routes, create_weather_use_case};
