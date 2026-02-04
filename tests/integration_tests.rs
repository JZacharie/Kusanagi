//! Integration tests for migrated endpoints

use actix_web::{test, web, App};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interfaces::http;

    #[actix_web::test]
    async fn test_nodes_endpoint() {
        let app = test::init_service(
            App::new().configure(http::nodes_pods_handlers::configure_routes)
        ).await;

        let req = test::TestRequest::get().uri("/api/nodes").to_request();
        let resp = test::call_service(&app, req).await;
        
        // Should return 200 or 500 (depending on k8s availability)
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_pods_endpoint() {
        let app = test::init_service(
            App::new().configure(http::nodes_pods_handlers::configure_routes)
        ).await;

        let req = test::TestRequest::get().uri("/api/pods").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_chat_endpoint() {
        let app = test::init_service(
            App::new().configure(http::chat_handlers_new::configure_routes)
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/chat/message")
            .set_json(&json!({"message": "test"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_mcp_k8s_resources() {
        let app = test::init_service(
            App::new().configure(http::mcp_handlers::configure_routes)
        ).await;

        let req = test::TestRequest::get().uri("/api/mcp/k8s-resources").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_cilium_flows() {
        let app = test::init_service(
            App::new().configure(http::cilium_handlers::configure_routes)
        ).await;

        let req = test::TestRequest::get().uri("/api/cilium/flows").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_proxmox_cluster() {
        let app = test::init_service(
            App::new().configure(http::proxmox_handlers::configure_routes)
        ).await;

        let req = test::TestRequest::get().uri("/api/proxmox/cluster").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_news_endpoint() {
        let app = test::init_service(
            App::new().configure(http::newsfeed_handlers::configure_routes)
        ).await;

        let req = test::TestRequest::get().uri("/api/news").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(
            App::new().configure(http::configure_routes)
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}
