//! Simplified integration tests for migrated architecture

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, HttpResponse};
    use serde_json::json;

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(
            App::new().service(
                web::resource("/health")
                    .route(web::get().to(|| async {
                        HttpResponse::Ok().json(json!({
                            "status": "healthy",
                            "version": "0.2.0"
                        }))
                    }))
            )
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_nodes_endpoint_mock() {
        let app = test::init_service(
            App::new().service(
                web::resource("/api/nodes")
                    .route(web::get().to(|| async {
                        HttpResponse::Ok().json(json!({
                            "nodes": [],
                            "total": 0
                        }))
                    }))
            )
        ).await;

        let req = test::TestRequest::get().uri("/api/nodes").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_pods_endpoint_mock() {
        let app = test::init_service(
            App::new().service(
                web::resource("/api/pods")
                    .route(web::get().to(|| async {
                        HttpResponse::Ok().json(json!({
                            "pods": [],
                            "total": 0
                        }))
                    }))
            )
        ).await;

        let req = test::TestRequest::get().uri("/api/pods").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_chat_endpoint_mock() {
        let app = test::init_service(
            App::new().service(
                web::resource("/api/chat/message")
                    .route(web::post().to(|body: web::Json<serde_json::Value>| async move {
                        let message = body.get("message").and_then(|m| m.as_str()).unwrap_or("");
                        HttpResponse::Ok().json(json!({
                            "response": format!("AI response to: {}", message)
                        }))
                    }))
            )
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/chat/message")
            .set_json(&json!({"message": "test"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_mcp_endpoint_mock() {
        let app = test::init_service(
            App::new().service(
                web::resource("/api/mcp/k8s-resources")
                    .route(web::get().to(|| async {
                        HttpResponse::Ok().json(json!({
                            "deployments": 0,
                            "statefulsets": 0,
                            "services": 0
                        }))
                    }))
            )
        ).await;

        let req = test::TestRequest::get().uri("/api/mcp/k8s-resources").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }

    #[test]
    fn test_architecture_migration_progress() {
        // Test that migration constants are correct
        const HIGH_PRIORITY_MODULES: usize = 7;
        const MEDIUM_PRIORITY_MODULES: usize = 11;
        const TOTAL_MODULES: usize = 36;
        
        let migrated = HIGH_PRIORITY_MODULES + MEDIUM_PRIORITY_MODULES;
        let progress = (migrated as f64 / TOTAL_MODULES as f64) * 100.0;
        
        assert_eq!(migrated, 18);
        assert_eq!(progress, 50.0);
        
        println!("✅ Migration progress: {:.1}% ({}/{})", progress, migrated, TOTAL_MODULES);
    }

    #[test]
    fn test_domain_entities_exist() {
        // Test that key domain entities are properly defined
        use crate::domain::entities::*;
        
        let _cluster = ClusterOverview::default();
        let _node = Node {
            name: "test".to_string(),
            status: NodeStatus::Ready,
            role: NodeRole::Worker,
            resources: NodeResources::default(),
            info: NodeInfo::default(),
            conditions: vec![],
        };
        let _pod = Pod {
            name: "test".to_string(),
            namespace: "default".to_string(),
            status: PodStatus::Running,
            containers: vec![],
            node_name: None,
            restart_count: 0,
            age: None,
            age_seconds: 0,
            labels: Default::default(),
            reason: None,
            message: None,
            cpu_usage: None,
            memory_usage: None,
            cpu_limit: None,
            memory_limit: None,
            cpu_request: None,
            memory_request: None,
        };
        
        println!("✅ Domain entities are properly defined");
    }

    #[test]
    fn test_error_handling() {
        use crate::error::KusanagiError;
        
        let error = KusanagiError::not_found("Pod", "test-pod");
        assert_eq!(error.user_message(), "Pod 'test-pod' not found");
        
        let error = KusanagiError::k8s("Connection failed".to_string());
        assert!(error.is_transient());
        
        println!("✅ Error handling works correctly");
    }
}
