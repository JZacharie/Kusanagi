//! Weather HTTP Handlers
//! 
//! Interface layer for weather endpoints.
//! Uses the GetWeatherUseCase from the application layer.

use actix_web::{web, HttpResponse, Result};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::application::use_cases::{GetWeatherUseCase, GetWeatherInput};
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
) -> Result<HttpResponse> {
    debug!("Weather request received, refresh={}", query.refresh);

    let input = GetWeatherInput {
        force_refresh: query.refresh,
    };

    match use_case.execute(input).await {
        Ok(weather) => {
            debug!("Weather data retrieved successfully");
            Ok(HttpResponse::Ok().json(weather))
        }
        Err(e) => {
            error!("Failed to get weather: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to retrieve weather data: {}", e)
            })))
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
    use_case: web::Data<GetWeatherUseCase>,
) -> Result<HttpResponse> {
    info!("Manual weather refresh requested");

    match use_case.force_refresh().await {
        Ok(_) => {
            info!("Weather data refreshed successfully");
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "message": "Weather data refreshed successfully"
            })))
        }
        Err(e) => {
            error!("Failed to refresh weather: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to refresh weather data: {}", e)
            })))
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
    use crate::infrastructure::repositories::{WeatherRepositoryImpl, create_weather_repository};
    
    let repository = Arc::new(create_weather_repository().await) as Arc<dyn WeatherRepository>;
    GetWeatherUseCase::new(repository)
}
