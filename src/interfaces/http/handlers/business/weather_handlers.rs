//! Weather HTTP Handlers
//!
//! Interface layer for weather endpoints.
//! Migrated from Actix-web to Axum.

use axum::{extract::State, response::IntoResponse, Json};
use tracing::{debug, error};

use crate::application::use_cases::GetWeatherInput;
use crate::state::AppState;

/// Get current weather for multiple cities
///
/// # Endpoint
/// GET /api/weather/current
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
            Json(weather).into_response()
        }
        Err(e) => {
            error!("Failed to get weather: {}", e);
            Json(serde_json::json!({
                "cities": [],
                "error": format!("Failed to retrieve weather: {}", e)
            }))
            .into_response()
        }
    }
}
