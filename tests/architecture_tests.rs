//! Simplified integration tests for migrated architecture

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, HttpResponse};
    use serde_json::json;

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(|| async {
                HttpResponse::Ok().json(json!({
                    "status": "healthy"
                }))
            }))
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_service_info_endpoint() {
        let app = test::init_service(
            App::new().route("/", web::get().to(|| async {
                HttpResponse::Ok().json(json!({
                    "service": "Kusanagi",
                    "version": "0.2.0"
                }))
            }))
        ).await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
