use actix_web::{get, post, web, HttpResponse, Responder};
use crate::application::use_cases::newsfeed_use_cases::*;
use crate::infrastructure::repositories::newsfeed_repository::LegacyNewsfeedRepository;
use std::sync::Arc;

#[get("/api/news")]
async fn get_news() -> impl Responder {
    let newsfeed_repo = Arc::new(LegacyNewsfeedRepository);
    let use_case = GetNewsUseCase::new(newsfeed_repo);
    
    match use_case.execute().await {
        Ok(news) => HttpResponse::Ok().json(news),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/news/refresh")]
async fn refresh_news() -> impl Responder {
    let newsfeed_repo = Arc::new(LegacyNewsfeedRepository);
    let use_case = RefreshNewsUseCase::new(newsfeed_repo);
    
    match use_case.execute().await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "refreshed"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_news)
        .service(refresh_news);
}
