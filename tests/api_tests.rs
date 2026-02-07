#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App, web};
    use serde_json::json;

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(crate::health_check))
        ).await;
        
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
    }

    #[actix_web::test]
    async fn test_service_info() {
        let app = test::init_service(
            App::new().route("/api", web::get().to(crate::service_info))
        ).await;
        
        let req = test::TestRequest::get().uri("/api").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["service"], "Kusanagi");
        assert_eq!(body["version"], "0.2.0");
    }

    #[actix_web::test]
    async fn test_system_status() {
        let app = test::init_service(
            App::new().route("/api/system/status", web::get().to(crate::system_status))
        ).await;
        
        let req = test::TestRequest::get().uri("/api/system/status").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["uptime_secs"].as_u64().is_some());
        assert!(body["memory_usage_mb"].as_f64().is_some());
        assert_eq!(body["version"], "0.2.0");
    }

    #[actix_web::test]
    async fn test_cluster_overview() {
        let client = kube::Client::try_default().await.ok();
        if client.is_none() {
            // Skip test if no k8s cluster available
            return;
        }
        
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(client.unwrap()))
                .route("/api/cluster/overview", web::get().to(crate::cluster_overview))
        ).await;
        
        let req = test::TestRequest::get().uri("/api/cluster/overview").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["nodes"].as_u64().is_some());
        assert!(body["pods"].as_u64().is_some());
    }

    #[actix_web::test]
    async fn test_alerts_endpoint() {
        let app = test::init_service(
            App::new().route("/api/alerts", web::get().to(crate::alerts))
        ).await;
        
        let req = test::TestRequest::get().uri("/api/alerts").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["alerts"].is_array());
        assert!(body["count"].as_u64().is_some());
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let app = test::init_service(
            App::new().route("/metrics", web::get().to(crate::prometheus_metrics))
        ).await;
        
        let req = test::TestRequest::get().uri("/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}
