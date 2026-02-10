//! Weather Repository Implementation
//!
//! Infrastructure adapter implementing the WeatherRepository port.
//! Handles external API calls (OpenWeather, wttr.in) and S3 caching.

use crate::domain::entities::{ForecastDay, WeatherInfo, WeatherResponse};
use crate::domain::ports::WeatherRepository;
use crate::domain::services::weather_service::WeatherDomainService;
use crate::error::{AppError, KusanagiError, Result};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Region, Client as S3Client};
use serde_json::Value;
use std::env;
use tracing::{debug, error, info, warn};

const S3_BUCKET: &str = "kusanagi";
const S3_KEY: &str = "weather-cache.json";
const S3_REGION: &str = "us-east-1";
const DEFAULT_MINIO_ENDPOINT: &str = "http://192.168.0.170:9010";
#[allow(dead_code)]
const CACHE_DURATION_HOURS: i64 = 6;

/// Weather repository implementation
pub struct WeatherRepositoryImpl {
    api_key: String,
    http_client: reqwest::Client,
    s3_client: Option<S3Client>,
    domain_service: WeatherDomainService,
}

impl WeatherRepositoryImpl {
    /// Create a new repository instance
    pub async fn new() -> Self {
        let api_key = env::var("OPENWEATHER_API_KEY").unwrap_or_default();

        if api_key.is_empty() {
            info!("OPENWEATHER_API_KEY not set, will use fallback weather sources");
        }

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        let s3_client = Self::init_s3_client().await.ok();

        Self {
            api_key,
            http_client,
            s3_client,
            domain_service: WeatherDomainService::new(),
        }
    }

    /// Initialize S3 client for caching
    async fn init_s3_client() -> Result<S3Client> {
        let endpoint =
            env::var("S3_ENDPOINT").unwrap_or_else(|_| DEFAULT_MINIO_ENDPOINT.to_string());

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(S3_REGION))
            .endpoint_url(&endpoint)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build();

        Ok(S3Client::from_conf(s3_config))
    }

    /// Fetch weather from S3 cache
    async fn fetch_from_cache(&self) -> Option<WeatherResponse> {
        let client = self.s3_client.as_ref()?;

        match client
            .get_object()
            .bucket(S3_BUCKET)
            .key(S3_KEY)
            .send()
            .await
        {
            Ok(resp) => {
                let data = resp.body.collect().await.ok()?.into_bytes();
                match serde_json::from_slice::<WeatherResponse>(&data) {
                    Ok(weather) => {
                        debug!("Successfully loaded weather from S3 cache");
                        Some(weather)
                    }
                    Err(e) => {
                        error!("Failed to parse S3 weather cache: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                debug!("Failed to fetch from S3 cache: {}", e);
                None
            }
        }
    }

    /// Save weather to S3 cache
    async fn save_to_cache(&self, weather: &WeatherResponse) -> Result<()> {
        let client = self
            .s3_client
            .as_ref()
            .ok_or_else(|| KusanagiError::configuration("S3 client not available"))?;

        let body = serde_json::to_string(weather)
            .map_err(|e| KusanagiError::serialization(e.to_string()))?;

        client
            .put_object()
            .bucket(S3_BUCKET)
            .key(S3_KEY)
            .body(body.into_bytes().into())
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("S3 error: {}", e)))?;

        info!("Weather data cached to S3 successfully");
        Ok(())
    }

    /// Fetch weather from OpenWeather API
    async fn fetch_from_openweather(&self, city: &str) -> Result<WeatherInfo> {
        if self.api_key.is_empty() {
            return Err(KusanagiError::configuration("OpenWeather API key not set"));
        }

        // Current weather
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        debug!("Fetching current weather for {} from OpenWeather", city);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("OpenWeather API error: {}", e)))?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(KusanagiError::external_service(format!(
                "OpenWeather API returned {}: {}",
                status, text
            )));
        }

        let resp = response
            .json::<Value>()
            .await
            .map_err(|e| KusanagiError::serialization(e.to_string()))?;

        let temp = resp["main"]["temp"].as_f64().unwrap_or(0.0) as f32;
        let description = resp["weather"][0]["description"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let icon_code = resp["weather"][0]["icon"]
            .as_str()
            .unwrap_or("01d")
            .to_string();
        let humidity = resp["main"]["humidity"].as_u64().unwrap_or(0) as u8;
        let wind_speed = resp["wind"]["speed"].as_f64().unwrap_or(0.0) as f32;
        let feels_like = resp["main"]["feels_like"].as_f64().unwrap_or(temp as f64) as f32;
        let pressure = resp["main"]["pressure"].as_u64().unwrap_or(1013) as u32;
        let visibility = resp["visibility"].as_u64().unwrap_or(10000) as u32;

        // 5-day forecast
        let forecast = self.fetch_forecast(city).await.unwrap_or_default();

        Ok(WeatherInfo {
            city: city.to_string(),
            temp,
            description,
            icon: icon_code,
            humidity,
            wind_speed,
            feels_like,
            pressure,
            visibility,
            last_updated: chrono::Local::now().format("%H:%M").to_string(),
            forecast,
        })
    }

    /// Fetch 5-day forecast from OpenWeather
    async fn fetch_forecast(&self, city: &str) -> Result<Vec<ForecastDay>> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/forecast?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(e.to_string()))?
            .json::<Value>()
            .await
            .map_err(|e| KusanagiError::serialization(e.to_string()))?;

        let mut forecast = Vec::new();

        if let Some(list) = resp["list"].as_array() {
            // Pick one entry per day (roughly mid-day, every 8th entry = 24h)
            for item in list.iter().step_by(8).take(5) {
                forecast.push(ForecastDay {
                    date: item["dt_txt"].as_str().unwrap_or("").to_string(),
                    temp: item["main"]["temp"].as_f64().unwrap_or(0.0) as f32,
                    description: item["weather"][0]["description"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    icon: item["weather"][0]["icon"]
                        .as_str()
                        .unwrap_or("01d")
                        .to_string(),
                });
            }
        }

        Ok(forecast)
    }

    /// Fetch weather from wttr.in fallback API
    async fn fetch_from_wttrin(&self, city: &str) -> Result<WeatherInfo> {
        let url = format!("https://wttr.in/{}?format=j1", city);

        debug!("Fetching weather for {} from wttr.in fallback", city);

        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("wttr.in error: {}", e)))?
            .json::<Value>()
            .await
            .map_err(|e| KusanagiError::serialization(e.to_string()))?;

        let current = &resp["current_condition"][0];

        let temp = current["temp_C"]
            .as_str()
            .unwrap_or("0")
            .parse::<f32>()
            .unwrap_or(0.0);

        let description = current["weatherDesc"][0]["value"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        let humidity = current["humidity"]
            .as_str()
            .unwrap_or("0")
            .parse::<u8>()
            .unwrap_or(0);

        let wind_speed = current["windspeedKmph"]
            .as_str()
            .unwrap_or("0")
            .parse::<f32>()
            .unwrap_or(0.0);

        let feels_like = current["FeelsLikeC"]
            .as_str()
            .unwrap_or("0")
            .parse::<f32>()
            .unwrap_or(temp);

        let pressure = current["pressure"]
            .as_str()
            .unwrap_or("1013")
            .parse::<u32>()
            .unwrap_or(1013);

        let visibility = current["visibility"]
            .as_str()
            .unwrap_or("10")
            .parse::<u32>()
            .unwrap_or(10)
            * 1000;

        let icon_code = self.domain_service.map_description_to_icon(&description);

        // Build forecast
        let mut forecast = Vec::new();
        if let Some(weather_list) = resp["weather"].as_array() {
            for day in weather_list.iter().take(5) {
                let date = day["date"].as_str().unwrap_or("").to_string();
                let hourly = &day["hourly"][4]; // Mid-day roughly

                forecast.push(ForecastDay {
                    date,
                    temp: day["avgtempC"]
                        .as_str()
                        .unwrap_or("0")
                        .parse::<f32>()
                        .unwrap_or(0.0),
                    description: hourly["weatherDesc"][0]["value"]
                        .as_str()
                        .unwrap_or("")
                        .to_string(),
                    icon: "03d".to_string(), // Default for forecast
                });
            }
        }

        Ok(WeatherInfo {
            city: city.to_string(),
            temp,
            description,
            icon: icon_code.to_string(),
            humidity,
            wind_speed,
            feels_like,
            pressure,
            visibility,
            last_updated: chrono::Local::now().format("%H:%M").to_string(),
            forecast,
        })
    }

    /// Fetch weather for a single city with fallback logic
    async fn fetch_city_weather(&self, city: &str) -> WeatherInfo {
        // Try OpenWeather first if API key is available
        if !self.api_key.is_empty() {
            match self.fetch_from_openweather(city).await {
                Ok(info) => return info,
                Err(e) => {
                    warn!("OpenWeather failed for {}: {}, trying fallback", city, e);
                }
            }
        }

        // Try wttr.in fallback
        match self.fetch_from_wttrin(city).await {
            Ok(info) => info,
            Err(e) => {
                warn!(
                    "wttr.in fallback failed for {}: {}, using mock data",
                    city, e
                );
                self.domain_service.create_mock_weather(city)
            }
        }
    }
}

#[async_trait]
impl WeatherRepository for WeatherRepositoryImpl {
    async fn get_multi_city_weather(&self, force_refresh: bool) -> Result<WeatherResponse> {
        // 1. Check cache first if not forced refresh
        if !force_refresh {
            if let Some(cached) = self.fetch_from_cache().await {
                if self.domain_service.is_cache_valid(&cached.cached_at) {
                    debug!("Returning cached weather data from S3");
                    return Ok(cached);
                } else {
                    debug!("S3 cache expired, refreshing...");
                }
            }
        }

        // 2. Fetch weather for all cities
        let cities = WeatherDomainService::get_default_cities();
        let mut results = Vec::new();

        for city in cities {
            let weather = self.fetch_city_weather(city).await;
            results.push(weather);
        }

        let response = self.domain_service.build_response(results);

        // 3. Save to cache
        if let Err(e) = self.save_to_cache(&response).await {
            error!("Failed to save weather to cache: {}", e);
            // Don't fail the request if caching fails
        }

        Ok(response)
    }

    async fn force_refresh(&self) -> Result<()> {
        info!("Forcing weather refresh...");
        self.get_multi_city_weather(true).await?;
        Ok(())
    }
}

/// Factory function for creating weather repository
pub async fn create_weather_repository() -> Box<dyn WeatherRepository> {
    Box::new(WeatherRepositoryImpl::new().await)
}
