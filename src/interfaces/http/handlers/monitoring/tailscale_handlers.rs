use axum::extract::State;
use axum::response::IntoResponse;

use crate::domain::services::tailscale_service;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

pub async fn get_tailscale_devices_handler(State(state): State<AppState>) -> impl IntoResponse {
    let result =
        tailscale_service::get_tailscale_devices_json(&state.http_client, &state.general_cache)
            .await;

    if result.get("error").is_some() {
        let err_msg = result["error"].as_str().unwrap_or("Unknown error");
        return api_error(
            axum::http::StatusCode::BAD_GATEWAY,
            format!("Tailscale API error: {}", err_msg),
        );
    }

    api_success(result)
}
