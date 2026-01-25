use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, error, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForecastDay {
    pub date: String,
    pub temp: f32,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeatherInfo {
    pub city: String,
    pub temp: f32,
    pub description: String,
    pub icon: String,
    pub humidity: u8,
    pub wind_speed: f32,
    pub feels_like: f32,
    pub pressure: u32,
    pub visibility: u32,
    pub last_updated: String,
    pub forecast: Vec<ForecastDay>,
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
            debug!("OPENWEATHER_API_KEY is empty, generating mock weather data for all cities");
            let mut results = Vec::new();
            for city in cities {
                results.push(self.get_mock_city_weather(city));
            }
            return Ok(WeatherResponse {
                cities: results,
                cached_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });
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
        // Current weather
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );
        debug!("Fetching current weather for {} from URL: {}", city, url);
        let resp = self.client.get(&url).send().await?.json::<serde_json::Value>().await?;

        let temp = resp["main"]["temp"].as_f64().unwrap_or(0.0) as f32;
        let description = resp["weather"][0]["description"].as_str().unwrap_or("unknown").to_string();
        let icon_code = resp["weather"][0]["icon"].as_str().unwrap_or("01d").to_string();
        let humidity = resp["main"]["humidity"].as_u64().unwrap_or(0) as u8;
        let wind_speed = resp["wind"]["speed"].as_f64().unwrap_or(0.0) as f32;
        let feels_like = resp["main"]["feels_like"].as_f64().unwrap_or(temp as f64) as f32;
        let pressure = resp["main"]["pressure"].as_u64().unwrap_or(1013) as u32;
        let visibility = resp["visibility"].as_u64().unwrap_or(10000) as u32;

        // 5-day forecast
        let forecast_url = format!(
            "https://api.openweathermap.org/data/2.5/forecast?q={}&appid={}&units=metric",
            city, self.api_key
        );
        let forecast_resp = self.client.get(&forecast_url).send().await?.json::<serde_json::Value>().await?;
        
        let mut forecast = Vec::new();
        if let Some(list) = forecast_resp["list"].as_array() {
            // Pick one entry per day (roughly mid-day)
            for item in list.iter().step_by(8).take(5) {
                forecast.push(ForecastDay {
                    date: item["dt_txt"].as_str().unwrap_or("").to_string(),
                    temp: item["main"]["temp"].as_f64().unwrap_or(0.0) as f32,
                    description: item["weather"][0]["description"].as_str().unwrap_or("").to_string(),
                    icon: item["weather"][0]["icon"].as_str().unwrap_or("01d").to_string(),
                });
            }
        }

        Ok(WeatherInfo {
            city: city.to_string(),
            temp,
            description,
            icon: icon_code, // Keep raw code for mapping to animated icons in frontend
            humidity,
            wind_speed,
            feels_like,
            pressure,
            visibility,
            last_updated: chrono::Local::now().format("%H:%M").to_string(),
            forecast,
        })
    }

    fn get_mock_city_weather(&self, city: &str) -> WeatherInfo {
        let (temp, desc, icon) = match city {
            "Lyon" => (5.2, "Cloudy", "04d"),
            "Mexico City" => (22.5, "Sunny", "01d"),
            "New York" => (-2.1, "Snowing", "13d"),
            _ => (15.0, "Clear", "01d"),
        };

        let forecast = vec![
            ForecastDay { date: "Tomorrow".to_string(), temp: temp + 2.0, description: "Partly Cloudy".to_string(), icon: "02d".to_string() },
            ForecastDay { date: "Day 2".to_string(), temp: temp + 1.0, description: "Cloudy".to_string(), icon: "03d".to_string() },
            ForecastDay { date: "Day 3".to_string(), temp: temp - 1.0, description: "Rainy".to_string(), icon: "10d".to_string() },
            ForecastDay { date: "Day 4".to_string(), temp: temp - 2.0, description: "Stormy".to_string(), icon: "11d".to_string() },
            ForecastDay { date: "Day 5".to_string(), temp: temp + 3.0, description: "Sunny".to_string(), icon: "01d".to_string() },
        ];

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
            last_updated: chrono::Local::now().format("%H:%M").to_string(),
            forecast,
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
