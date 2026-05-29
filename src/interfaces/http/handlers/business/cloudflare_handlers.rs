use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;

pub async fn get_cloudflare_analytics_handler(State(state): State<AppState>) -> impl IntoResponse {
    match state.business_use_case.execute().await {
        Ok(analytics) => Json(analytics).into_response(),
        Err(e) => {
            tracing::error!("Failed to get Cloudflare analytics: {}", e);
            Json(json!({"error": e.to_string()})).into_response()
        }
    }
}
