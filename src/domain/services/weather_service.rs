//! Weather Domain Service
//! 
//! Core business logic for weather operations.
//! This service is independent of infrastructure concerns.

use crate::domain::entities::{ForecastDay, WeatherInfo, WeatherResponse};
use chrono::Local;
use tracing::warn;

/// Service for weather domain operations
pub struct WeatherDomainService;

impl WeatherDomainService {
    /// Create a new instance
    pub fn new() -> Self {
        Self
    }

    /// Get default cities for weather display
    pub fn get_default_cities() -> Vec<&'static str> {
        vec!["Lyon", "Mexico City", "New York"]
    }

    /// Create mock weather data for a city when external APIs fail
    pub fn create_mock_weather(&self, city: &str) -> WeatherInfo {
        let (temp, desc, icon) = match city {
            "Lyon" => (5.2, "Cloudy", "04d"),
            "Mexico City" => (22.5, "Sunny", "01d"),
            "New York" => (-2.1, "Snowing", "13d"),
            _ => (15.0, "Clear", "01d"),
        };

        let forecast = self.generate_mock_forecast(temp);

        WeatherInfo {
            city: city.to_string(),
            temp,
            description: desc.to_string(),
            icon: icon.to_string(),
            humidity: 45,
            wind_speed: 12.5,
            feels_like: temp + 1.0,
            pressure: 1015,
            visibility: 10000,
            last_updated: Local::now().format("%H:%M").to_string(),
            forecast,
        }
    }

    /// Generate mock forecast based on current temperature
    fn generate_mock_forecast(&self, base_temp: f32) -> Vec<ForecastDay> {
        let mut forecast = Vec::new();
        
        for i in 1..=5 {
            let forecast_date = (Local::now() + chrono::Duration::days(i))
                .format("%Y-%m-%d 12:00:00")
                .to_string();

            let (f_temp, f_desc, f_icon) = match i {
                1 => (base_temp + 2.0, "Partly Cloudy", "02d"),
                2 => (base_temp + 1.0, "Cloudy", "03d"),
                3 => (base_temp - 1.0, "Rainy", "10d"),
                4 => (base_temp - 2.0, "Stormy", "11d"),
                _ => (base_temp + 3.0, "Sunny", "01d"),
            };

            forecast.push(ForecastDay {
                date: forecast_date,
                temp: f_temp,
                description: f_desc.to_string(),
                icon: f_icon.to_string(),
            });
        }

        forecast
    }

    /// Map weather description to icon code
    pub fn map_description_to_icon(&self, description: &str) -> &'static str {
        let desc_lower = description.to_lowercase();
        match desc_lower.as_str() {
            d if d.contains("sunny") || d.contains("clear") => "01d",
            d if d.contains("partly cloudy") => "02d",
            d if d.contains("cloudy") || d.contains("overcast") => "04d",
            d if d.contains("rain") || d.contains("drizzle") => "10d",
            d if d.contains("snow") => "13d",
            d if d.contains("thunder") || d.contains("storm") => "11d",
            _ => "03d",
        }
    }

    /// Check if cached weather data is still valid (less than 6 hours old)
    pub fn is_cache_valid(&self, cached_at: &str) -> bool {
        match chrono::NaiveDateTime::parse_from_str(cached_at, "%Y-%m-%d %H:%M:%S") {
            Ok(cached_time) => {
                let six_hours_ago = Local::now().naive_local() - chrono::Duration::hours(6);
                cached_time > six_hours_ago
            }
            Err(e) => {
                warn!("Failed to parse cache time: {}", e);
                false
            }
        }
    }

    /// Build a weather response from individual city weather infos
    pub fn build_response(&self, cities: Vec<WeatherInfo>) -> WeatherResponse {
        WeatherResponse {
            cities,
            cached_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

impl Default for WeatherDomainService {
    fn default() -> Self {
        Self::new()
    }
}
