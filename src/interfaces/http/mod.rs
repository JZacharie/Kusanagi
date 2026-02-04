//! HTTP handlers (REST API)
//!
//! These handlers use application services and use cases
//! to fulfill HTTP requests.

use actix_web::{get, web, HttpResponse, Responder};

use crate::error::KusanagiError;

mod event_handlers;
mod node_handlers;
mod argocd_handlers;
mod storage_handlers;
mod service_handlers;
mod ingress_handlers;
mod cluster_handlers;
mod prometheus_handlers;
mod backup_handlers;
mod security_handlers;
mod alert_handlers;
mod chat_handlers;
mod node_metrics_handlers;
mod integration_handlers;
mod nodes_pods_handlers;
mod chat_handlers_new;
mod mcp_handlers;
mod cilium_handlers;
mod proxmox_handlers;
mod newsfeed_handlers;
mod prometheus_handlers_new;
mod security_handlers_new;
mod backup_handlers_new;
mod medium_priority_handlers;
pub mod pod_handlers;

pub use event_handlers::*;
pub use node_handlers::*;
pub use argocd_handlers::*;
pub use storage_handlers::*;
pub use service_handlers::*;
pub use ingress_handlers::*;
pub use cluster_handlers::*;
pub use prometheus_handlers::*;
pub use backup_handlers::*;
pub use security_handlers::*;
pub use alert_handlers::*;
pub use chat_handlers::*;
pub use node_metrics_handlers::*;
pub use integration_handlers::*;
pub use nodes_pods_handlers::*;
pub use chat_handlers_new::*;
pub use mcp_handlers::*;
pub use cilium_handlers::*;
pub use proxmox_handlers::*;
pub use newsfeed_handlers::*;
pub use prometheus_handlers_new::*;
pub use security_handlers_new::*;
pub use backup_handlers_new::*;
pub use medium_priority_handlers::*;

// Re-export the main AppState from crate root
pub use crate::AppState;

/// Configure HTTP routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check)
        // Cluster
        .service(get_cluster_overview)
        .service(get_empty_namespaces)
        .service(get_cluster_stats)
        // Nodes
        .service(list_nodes)
        .service(get_nodes_status)
        .service(get_node_details)
        .service(is_node_ready)
        // Node Metrics (with disk usage)
        .service(list_nodes_with_disk_metrics)
        .service(get_node_disk_metrics)
        .service(get_cluster_disk_summary)
        // Note: Pod handlers are configured separately in main.rs because they require PodService
        // .configure(pod_handlers::configure_routes) // This needs PodService as data
        // Events
        .service(list_events)
        .service(list_warning_events)
        .service(get_event_stats)
        // Services
        .service(list_services)
        .service(get_service_stats)
        .service(get_service_details)
        // Ingresses
        .service(list_ingresses)
        .service(get_ingress_stats)
        .service(get_ingress_details)
        // Storage
        .service(get_storage_info)
        .service(get_storage_stats)
        // Prometheus
        .service(get_cluster_metrics)
        .service(query_metric)
        .service(query_raw)
        .service(query_range)
        // Backups
        .service(get_backup_status)
        .service(get_backup_stats)
        .service(list_cronjobs)
        .service(trigger_backup)
        // Security
        .service(list_security_reports)
        .service(get_security_summary)
        .service(get_enriched_report)
        .service(enrich_security_report)
        .service(run_security_enrichment)
        // Alerts
        .service(get_active_alerts)
        .service(get_cached_alerts)
        .service(get_alert_stats)
        .service(get_alert)
        .service(silence_alert)
        // Chat
        .service(process_chat_message)
        .service(handle_chat_command)
        .service(query_ai)
        .service(get_chat_history)
        .service(clear_chat_history)
        // Integration routes
        .configure(integration_handlers::configure_routes)
        // High-priority migrated modules
        .configure(nodes_pods_handlers::configure_routes)
        .configure(chat_handlers_new::configure_routes)
        .configure(mcp_handlers::configure_routes)
        .configure(cilium_handlers::configure_routes)
        .configure(proxmox_handlers::configure_routes)
        .configure(newsfeed_handlers::configure_routes)
        // Medium-priority migrated modules
        .configure(prometheus_handlers_new::configure_routes)
        .configure(security_handlers_new::configure_routes)
        .configure(backup_handlers_new::configure_routes)
        .configure(medium_priority_handlers::configure_routes);
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// Note: API route handlers are defined in their respective submodule files:
// - cluster_handlers.rs: /api/cluster/overview, /api/cluster/empty-namespaces, /api/cluster/stats
// - node_handlers.rs: /api/nodes, /api/nodes/status, /api/nodes/{name}
// - pod_handlers.rs: /api/pods, /api/pods/{namespace}/{name}
// - event_handlers.rs: /api/events
// - namespace_handlers.rs: /api/namespaces
// - service_handlers.rs: /api/services
// - storage_handlers.rs: /api/storage
// - prometheus_handlers.rs: /api/metrics, /api/prometheus/*
// - alert_handlers.rs: /api/alerts, /api/alerts/*

/// Error response structure (for future error handling)
#[derive(serde::Serialize)]
#[allow(dead_code)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl From<KusanagiError> for ErrorResponse {
    fn from(err: KusanagiError) -> Self {
        Self {
            error: err.to_string(),
            message: err.user_message(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            actix_web::App::new().configure(configure_routes)
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}
