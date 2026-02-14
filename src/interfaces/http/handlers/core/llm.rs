//! LLM HTTP Handlers
//! Axum handlers for LLM service endpoints

use crate::domain::entities::llm::{LlmConfigInfo, LlmHealthResponse};
use crate::domain::services::llm_service::LlmService;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tracing::error;

/// LLM health check endpoint
/// GET /api/llm/health
pub async fn llm_health_check(State(_state): State<AppState>) -> impl IntoResponse {
    let service = LlmService::new();
    let config = service.config();

    match service.health_check().await {
        Ok(_) => {
            let response = LlmHealthResponse {
                healthy: true,
                provider: format!("{:?}", config.provider),
                model: config.model.clone(),
                error: None,
            };
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            error!("LLM health check failed: {}", e);
            let response = LlmHealthResponse {
                healthy: false,
                provider: format!("{:?}", config.provider),
                model: config.model.clone(),
                error: Some(e.to_string()),
            };
            (StatusCode::SERVICE_UNAVAILABLE, Json(response))
        }
    }
}

/// LLM configuration info endpoint
/// GET /api/llm/config
pub async fn llm_config_info(State(_state): State<AppState>) -> impl IntoResponse {
    let service = LlmService::new();
    let config = service.config();

    let info = LlmConfigInfo {
        provider: format!("{:?}", config.provider),
        base_url: config.base_url.clone(),
        model: config.model.clone(),
        timeout_secs: config.timeout_secs,
        max_retries: config.max_retries,
        has_api_key: config.api_key.is_some(),
        is_valid: config.is_valid(),
    };

    (StatusCode::OK, Json(info))
}
