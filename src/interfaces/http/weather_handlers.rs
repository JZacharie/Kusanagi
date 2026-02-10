//! Weather HTTP Handlers
//!
//! Interface layer for weather endpoints.
//! Uses the GetWeatherUseCase from the application layer.

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::{
    application::use_cases::{GetWeatherInput, GetWeatherUseCase},
    state::AppState,
};

/// Query parameters for weather endpoint
#[derive(Debug, serde::Deserialize)]
pub struct WeatherQuery {
    /// Force refresh of weather data
    #[serde(default)]
    pub refresh: bool,
}

/// Get current weather for multiple cities
///
/// # Endpoint
/// GET /api/weather/current
///
/// # Query Parameters
/// - `refresh`: Force cache refresh (optional, default: false)
///
/// # Response
/// Returns a JSON object with weather data for configured cities
pub async fn get_weather_handler(
    State(state): State<AppState>,
    Query(query): Query<WeatherQuery>,
) -> impl IntoResponse {
    debug!("Weather request received, refresh={}", query.refresh);

    let input = GetWeatherInput {
        force_refresh: query.refresh,
    };

    match state.weather_use_case.execute(input).await {
        Ok(weather) => {
            debug!("Weather data retrieved successfully");
            Json(weather).into_response()
        }
        Err(e) => {
            error!("Failed to get weather: {}", e);
            // Return mock data on error to ensure frontend always gets valid JSON
            Json(serde_json::json!({
                "cities": [
                    {
                        "city": "Paris",
                        "temp": 15.5,
                        "description": "Partly cloudy",
                        "icon": "02d",
                        "humidity": 65,
                        "wind_speed": 12.0,
                        "feels_like": 14.0,
                        "pressure": 1013,
                        "visibility": 10000,
                        "last_updated": "12:00",
                        "forecast": []
                    }
                ],
                "cached_at": chrono::Utc::now().to_rfc3339(),
                "source": "error_fallback"
            }))
            .into_response()
        }
    }
}

/// Force refresh weather data
///
/// # Endpoint
/// POST /api/weather/refresh
///
/// # Response
/// Returns 200 OK on success, error on failure
pub async fn refresh_weather_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    info!("Manual weather refresh requested");

    match state.weather_use_case.force_refresh().await {
        Ok(_) => {
            info!("Weather data refreshed successfully");
            Json(serde_json::json!({
                "status": "success",
                "message": "Weather data refreshed successfully"
            }))
            .into_response()
        }
        Err(e) => {
            error!("Failed to refresh weather: {}", e);
            Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to refresh weather data: {}", e)
            }))
            .into_response()
        }
    }
}


