//! Weather HTTP Handlers
//!
//! Interface layer for weather endpoints.
//! Uses the GetWeatherUseCase from the application layer.

use actix_web::{web, HttpResponse};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::application::use_cases::{GetWeatherInput, GetWeatherUseCase};
use crate::domain::ports::WeatherRepository;

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
    use_case: web::Data<GetWeatherUseCase>,
    query: web::Query<WeatherQuery>,
) -> HttpResponse {
    debug!("Weather request received, refresh={}", query.refresh);

    let input = GetWeatherInput {
        force_refresh: query.refresh,
    };

    match use_case.execute(input).await {
        Ok(weather) => {
            debug!("Weather data retrieved successfully");
            HttpResponse::Ok().json(weather)
        }
        Err(e) => {
            error!("Failed to get weather: {}", e);
            // Return mock data on error to ensure frontend always gets valid JSON
            HttpResponse::Ok().json(serde_json::json!({
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
pub async fn refresh_weather_handler(use_case: web::Data<GetWeatherUseCase>) -> HttpResponse {
    info!("Manual weather refresh requested");

    match use_case.force_refresh().await {
        Ok(_) => {
            info!("Weather data refreshed successfully");
            HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "message": "Weather data refreshed successfully"
            }))
        }
        Err(e) => {
            error!("Failed to refresh weather: {}", e);
            HttpResponse::Ok().json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to refresh weather data: {}", e)
            }))
        }
    }
}

/// Configure weather routes
///
/// Adds weather endpoints to the Actix-Web service configuration
pub fn configure_weather_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/weather")
            .route("/current", web::get().to(get_weather_handler))
            .route("/refresh", web::post().to(refresh_weather_handler)),
    );
}

/// Create GetWeatherUseCase with repository
///
/// Helper function to create the use case with the weather repository
pub async fn create_weather_use_case() -> GetWeatherUseCase {
    use crate::infrastructure::repositories::WeatherRepositoryImpl;

    let repository: Arc<dyn WeatherRepository> = Arc::new(WeatherRepositoryImpl::new().await);
    GetWeatherUseCase::new(repository)
}
