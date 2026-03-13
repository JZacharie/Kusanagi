use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use crate::domain::entities::A2UIMessage;
use crate::state::AppState;
use serde_json::json;

pub async fn post_a2ui_message_handler(
    State(state): State<AppState>,
    Json(message): Json<A2UIMessage>,
) -> impl IntoResponse {
    match state.a2ui_use_case.process_message(message).await {
        Ok(_) => Json(json!({"status": "ok"})).into_response(),
        Err(e) => Json(json!({"error": e.to_string()})).into_response(),
    }
}

pub async fn get_a2ui_surface_handler(
    State(state): State<AppState>,
    Path(surface_id): Path<String>,
) -> impl IntoResponse {
    match state.a2ui_use_case.get_surface(&surface_id).await {
        Ok(surface) => Json(surface).into_response(),
        Err(e) => Json(json!({"error": e.to_string()})).into_response(),
    }
}

pub async fn get_a2ui_data_handler(
    State(state): State<AppState>,
    Path(surface_id): Path<String>,
) -> impl IntoResponse {
    match state.a2ui_use_case.get_data_model(&surface_id).await {
        Ok(data) => Json(data).into_response(),
        Err(e) => Json(json!({"error": e.to_string()})).into_response(),
    }
}
