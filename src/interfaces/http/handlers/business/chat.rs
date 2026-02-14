use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub language: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
}

pub async fn post_chat_handler(
    State(state): State<AppState>,
    Json(payload): Json<ChatRequest>,
) -> impl IntoResponse {
    let message = payload.message.trim();
    let language = if payload.language.is_empty() {
        "fr"
    } else {
        &payload.language
    };
    let response_text = state.chat_use_case.execute(message, language).await;

    Json(ChatResponse {
        response: response_text,
    })
}
