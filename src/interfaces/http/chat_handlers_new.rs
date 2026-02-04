use actix_web::{get, post, web, HttpResponse, Responder};
use crate::application::use_cases::chat_use_cases_new::*;
use crate::infrastructure::repositories::chat_repository::{LegacyChatRepository, LegacyAiService};
use std::sync::Arc;

#[post("/api/chat/message")]
async fn process_chat_message(body: web::Json<serde_json::Value>) -> impl Responder {
    let message = body.get("message").and_then(|m| m.as_str()).unwrap_or("");
    
    let chat_repo = Arc::new(LegacyChatRepository);
    let ai_service = Arc::new(LegacyAiService);
    let use_case = ProcessChatUseCase::new(chat_repo, ai_service);
    
    match use_case.execute(message).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "response": response
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/chat/history")]
async fn get_chat_history(query: web::Query<serde_json::Value>) -> impl Responder {
    let limit = query.get("limit").and_then(|l| l.as_u64()).map(|l| l as usize);
    
    let chat_repo = Arc::new(LegacyChatRepository);
    let use_case = GetChatHistoryUseCase::new(chat_repo);
    
    match use_case.execute(limit).await {
        Ok(history) => HttpResponse::Ok().json(history),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(process_chat_message)
        .service(get_chat_history);
}
