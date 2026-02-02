//! Chat HTTP Handlers
//!
//! HTTP handlers for chat operations.

use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;
use std::sync::Arc;

use crate::application::use_cases::chat_use_cases::*;
use crate::domain::entities::{ChatRequest, ChatCommand};
use crate::domain::ports::{ChatService, ChatHistoryRepository};
use crate::interfaces::http::{AppState, ErrorResponse};

#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
}

#[derive(Debug, Deserialize)]
pub struct AiQueryRequest {
    pub query: String,
    pub context: Option<String>,
    pub language: Option<String>,
}

/// Process chat message
#[post("/api/chat")]
pub async fn process_chat_message(
    data: web::Data<AppState>,
    body: web::Json<ChatRequest>,
) -> impl Responder {
    let chat_service = match data.get_chat_service() {
        Some(service) => service,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Chat service not available"
            })),
    };

    let history_repo = match data.get_chat_history_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Chat history repository not available"
            })),
    };

    let use_case = ProcessChatMessageUseCase::new(chat_service, history_repo);

    match use_case.execute(body.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.error_response(),
    }
}

/// Handle chat command
#[post("/api/chat/command")]
pub async fn handle_chat_command(
    data: web::Data<AppState>,
    body: web::Json<CommandRequest>,
) -> impl Responder {
    let chat_service = match data.get_chat_service() {
        Some(service) => service,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Chat service not available"
            })),
    };

    let use_case = HandleChatCommandUseCase::new(chat_service);
    let command = ChatCommand::from(body.command.as_str());

    match use_case.execute(command).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => e.error_response(),
    }
}

/// Query AI
#[post("/api/chat/query")]
pub async fn query_ai(
    data: web::Data<AppState>,
    body: web::Json<AiQueryRequest>,
) -> impl Responder {
    let chat_service = match data.get_chat_service() {
        Some(service) => service,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Chat service not available"
            })),
    };

    let use_case = QueryAiUseCase::new(chat_service);
    let context = body.context.as_deref().unwrap_or("");
    let lang = body.language.as_deref().unwrap_or("en");

    match use_case.execute(&body.query, context, lang).await {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({ "response": response })),
        Err(e) => e.error_response(),
    }
}

/// Get chat history
#[get("/api/chat/history")]
pub async fn get_chat_history(
    data: web::Data<AppState>,
    query: web::Query<HistoryQuery>,
) -> impl Responder {
    let history_repo = match data.get_chat_history_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Chat history repository not available"
            })),
    };

    let use_case = GetChatHistoryUseCase::new(history_repo);

    match use_case.execute(query.limit.unwrap_or(50)).await {
        Ok(history) => HttpResponse::Ok().json(history),
        Err(e) => e.error_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<usize>,
}

/// Clear chat history
#[post("/api/chat/clear")]
pub async fn clear_chat_history(
    data: web::Data<AppState>,
) -> impl Responder {
    let history_repo = match data.get_chat_history_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Chat history repository not available"
            })),
    };

    let use_case = ClearChatHistoryUseCase::new(history_repo);

    match use_case.execute().await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Chat history cleared"
        })),
        Err(e) => e.error_response(),
    }
}
