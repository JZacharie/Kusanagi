use crate::state::AppState;
use axum::{extract::State, response::IntoResponse};

/// Prometheus metrics endpoint
/// Returns the current metrics in Prometheus format
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    state.prometheus_handle.render()
}
