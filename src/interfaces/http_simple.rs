use actix_web::{web, HttpResponse, Responder};
use crate::application::use_cases_simple::GetClusterOverviewUseCase;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/health", web::get().to(health_check))
            .route("/cluster/overview", web::get().to(get_cluster_overview))
    )
    .route("/", web::get().to(index))
    .route("/health", web::get().to(health_check));
}

async fn index() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "Kusanagi",
        "version": "0.2.0",
        "status": "running",
        "architecture": "hexagonal",
        "mode": if std::env::var("KUBERNETES_SERVICE_HOST").is_ok() { "kubernetes" } else { "local" }
    }))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn get_cluster_overview(
    use_case: web::Data<GetClusterOverviewUseCase>
) -> impl Responder {
    match use_case.execute().await {
        Ok(overview) => HttpResponse::Ok().json(overview),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}
