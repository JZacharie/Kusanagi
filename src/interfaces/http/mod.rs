//! HTTP handlers (REST API)
//!
//! These handlers use application services and use cases
//! to fulfill HTTP requests.

use actix_web::{get, web, HttpResponse, Responder};

use crate::error::KusanagiError;

// Temporarily comment out problematic handlers for compilation
// mod event_handlers;
// mod node_handlers;
// mod argocd_handlers;
// mod storage_handlers;
// mod service_handlers;
// mod ingress_handlers;
// mod cluster_handlers;
// mod prometheus_handlers;
// mod backup_handlers;
// mod security_handlers;
// mod alert_handlers;
// mod chat_handlers;
// mod node_metrics_handlers;
// mod integration_handlers;
// mod nodes_pods_handlers;
// mod chat_handlers_new;
// mod mcp_handlers;
// mod cilium_handlers;
// mod proxmox_handlers;
// mod newsfeed_handlers;
// mod prometheus_handlers_new;
// mod security_handlers_new;
// mod backup_handlers_new;
// mod medium_priority_handlers;
// pub mod pod_handlers;

// pub use event_handlers::*;
// pub use node_handlers::*;
// pub use argocd_handlers::*;
// pub use storage_handlers::*;
// pub use service_handlers::*;
// pub use ingress_handlers::*;
// pub use cluster_handlers::*;
// pub use prometheus_handlers::*;
// pub use backup_handlers::*;
// pub use security_handlers::*;
// pub use alert_handlers::*;
// pub use chat_handlers::*;
// pub use node_metrics_handlers::*;
// pub use integration_handlers::*;
// pub use nodes_pods_handlers::*;
// pub use chat_handlers_new::*;
// pub use mcp_handlers::*;
// pub use cilium_handlers::*;
// pub use proxmox_handlers::*;
// pub use newsfeed_handlers::*;
// pub use prometheus_handlers_new::*;
// pub use security_handlers_new::*;
// pub use backup_handlers_new::*;
// pub use medium_priority_handlers::*;

// Re-export the main AppState from crate root
// pub use crate::AppState; // Commented out for compilation

/// Configure HTTP routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check);
        // Temporarily disabled all other routes for compilation
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

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

// Low Priority Handlers (Phase 3)
// pub mod low_priority_handlers_part1;
// pub mod low_priority_handlers_part2;

// pub use low_priority_handlers_part1::*;
// pub use low_priority_handlers_part2::*;
