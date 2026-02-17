//! Weather HTTP Handlers
//!
//! Interface layer for weather endpoints.
//! Migrated from Actix-web to Axum.

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use tracing::{debug, error};

use crate::application::use_cases::GetWeatherInput;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

/// Get current weather for multiple cities
#[utoipa::path(
    get,
    path = "/api/weather/current",
    responses(
        (status = 200, description = "Weather data retrieved successfully"),
        (status = 500, description = "Failed to retrieve weather")
    ),
    tag = "weather"
)]
pub async fn get_weather_handler(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Weather request received");

    let input = GetWeatherInput {
        force_refresh: false,
    };

    match state.weather_use_case.execute(input).await {
        Ok(weather) => {
            debug!(
                "Weather retrieved successfully for {} cities",
                weather.cities.len()
            );
            api_success(serde_json::to_value(weather).unwrap_or_default())
        }
        Err(e) => {
            error!("Failed to get weather: {}", e);
            api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to retrieve weather: {}", e),
            )
        }
    }
}
