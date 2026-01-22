use actix_web::{web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info, warn};
use chrono;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeatherInfo {
    pub city: String,
    pub temp: f32,
    pub description: String,
    pub icon: String,
    pub humidity: u8,
    pub wind_speed: f32,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub cities: Vec<WeatherInfo>,
    pub cached_at: String,
}

pub struct WeatherClient {
    api_key: String,
    client: reqwest::Client,
}

impl WeatherClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("OPENWEATHER_API_KEY")
            .unwrap_or_else(|_| "".to_string());
        
        if api_key.is_empty() {
            warn!("OPENWEATHER_API_KEY not set, using mock weather data");
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        Ok(Self {
            api_key,
            client,
        })
    }

    pub async fn get_multi_city_weather(&self) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
        let cities = vec!["Lyon", "Mexico City", "New York"];
        let mut results = Vec::new();

        if self.api_key.is_empty() {
            return Ok(self.get_mock_weather());
        }

        for city in cities {
            match self.fetch_city_weather(city).await {
                Ok(info) => results.push(info),
                Err(e) => {
                    error!("Failed to fetch weather for {}: {}", city, e);
                    // Fallback to mock for this city if API fails
                    results.push(self.get_mock_city_weather(city));
                }
            }
        }

        Ok(WeatherResponse {
            cities: results,
            cached_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
    }

    async fn fetch_city_weather(&self, city: &str) -> Result<WeatherInfo, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let resp = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;

        let temp = resp["main"]["temp"].as_f64().unwrap_or(0.0) as f32;
        let description = resp["weather"][0]["description"].as_str().unwrap_or("unknown").to_string();
        let icon_code = resp["weather"][0]["icon"].as_str().unwrap_or("01d").to_string();
        let humidity = resp["main"]["humidity"].as_u64().unwrap_or(0) as u8;
        let wind_speed = resp["wind"]["speed"].as_f64().unwrap_or(0.0) as f32;

        Ok(WeatherInfo {
            city: city.to_string(),
            temp,
            description,
            icon: self.map_icon(&icon_code),
            humidity,
            wind_speed,
            last_updated: chrono::Local::now().format("%H:%M").to_string(),
        })
    }

    fn map_icon(&self, code: &str) -> String {
        // Map OpenWeather codes to Emojis for simplicity in our cyberpunk UI
        let icon = match code {
            "01d" | "01n" => "☀️",
            "02d" | "02n" => "⛅",
            "03d" | "03n" | "04d" | "04n" => "☁️",
            "09d" | "09n" | "10d" | "10n" => "🌧️",
            "11d" | "11n" => "⚡",
            "13d" | "13n" => "❄️",
            "50d" | "50n" => "🌫️",
            _ => "🌡️",
        };
        icon.to_string()
    }

    fn get_mock_weather(&self) -> WeatherResponse {
        WeatherResponse {
            cities: vec![
                self.get_mock_city_weather("Lyon"),
                self.get_mock_city_weather("Mexico City"),
                self.get_mock_city_weather("New York"),
            ],
            cached_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    fn get_mock_city_weather(&self, city: &str) -> WeatherInfo {
        let (temp, desc, icon) = match city {
            "Lyon" => (5.2, "Cloudy", "☁️"),
            "Mexico City" => (22.5, "Sunny", "☀️"),
            "New York" => (-2.1, "Snowing", "❄️"),
            _ => (15.0, "Clear", "☀️"),
        };

        WeatherInfo {
            city: city.to_string(),
            temp,
            description: desc.to_string(),
            icon: icon.to_string(),
            humidity: 45,
            wind_speed: 12.5,
            last_updated: chrono::Local::now().format("%H:%M").to_string(),
        }
    }
}

// API Handlers
pub async fn get_weather_handler() -> Result<HttpResponse> {
    match WeatherClient::new() {
        Ok(client) => match client.get_multi_city_weather().await {
            Ok(weather) => Ok(HttpResponse::Ok().json(weather)),
            Err(e) => {
                error!("Weather error: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})))
            }
        },
        Err(e) => {
            error!("Failed to create weather client: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/weather")
            .route("/current", web::get().to(get_weather_handler)),
    );
}
