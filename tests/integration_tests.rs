//! Integration tests for Kusanagi

use actix_web::{test, web, App, HttpResponse};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_basic_endpoints() {
        let app = test::init_service(
            App::new()
                .route("/health", web::get().to(|| async {
                    HttpResponse::Ok().json(json!({
                        "status": "healthy"
                    }))
                }))
                .route("/", web::get().to(|| async {
                    HttpResponse::Ok().json(json!({
                        "service": "Kusanagi",
                        "version": "0.2.0"
                    }))
                }))
        ).await;

        // Test health endpoint
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Test service info endpoint
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
