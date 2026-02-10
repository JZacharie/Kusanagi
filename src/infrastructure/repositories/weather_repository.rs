//! Weather Repository Implementation
//!
//! Infrastructure adapter implementing the WeatherRepository port.
//! Uses Open-Meteo API (open source, no API key required).

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

/// City coordinates for Open-Meteo API
fn get_city_coordinates(city: &str) -> Option<(f64, f64)> {
    match city {
        "Lyon" => Some((45.7485, 4.8467)),
        "Mexico City" => Some((19.4326, -99.1332)),
        "New York" => Some((40.7128, -74.0060)),
        "Paris" => Some((48.8566, 2.3522)),
        "London" => Some((51.5074, -0.1278)),
        "Tokyo" => Some((35.6762, 139.6503)),
        "Sydney" => Some((-33.8688, 151.2093)),
        "Berlin" => Some((52.5200, 13.4050)),
        "Madrid" => Some((40.4168, -3.7038)),
        "Rome" => Some((41.9028, 12.4964)),
        _ => None,
    }
}

/// Map WMO weather codes to description and icon
fn map_weather_code(code: i64) -> (&'static str, &'static str) {
    match code {
        0 => ("Clear sky", "01d"),
        1 => ("Mainly clear", "02d"),
        2 => ("Partly cloudy", "03d"),
        3 => ("Overcast", "04d"),
        45 | 48 => ("Foggy", "50d"),
        51..=55 => ("Drizzle", "09d"),
        56..=57 => ("Freezing drizzle", "09d"),
        61..=65 => ("Rain", "10d"),
        66..=67 => ("Freezing rain", "10d"),
        71..=75 => ("Snow", "13d"),
        77 => ("Snow grains", "13d"),
        80..=82 => ("Showers", "09d"),
        85..=86 => ("Snow showers", "13d"),
        95 => ("Thunderstorm", "11d"),
        96 | 99 => ("Thunderstorm with hail", "11d"),
        _ => ("Unknown", "03d"),
    }
}

/// Weather repository implementation
pub struct WeatherRepositoryImpl {
    _api_key: String, // Kept for backward compatibility
    http_client: reqwest::Client,
    s3_client: Option<S3Client>,
    domain_service: WeatherDomainService,
}

impl WeatherRepositoryImpl {
    /// Create a new repository instance
    pub async fn new() -> Self {
        // Check for legacy API key (not needed for Open-Meteo)
        let api_key = env::var("OPENWEATHER_API_KEY").unwrap_or_default();
        if !api_key.is_empty() {
            info!(
                "OPENWEATHER_API_KEY is set but not used - using Open-Meteo (no API key required)"
            );
        }

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        let s3_client = Self::init_s3_client().await.ok();

        Self {
            _api_key: api_key,
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

    /// Fetch weather from Open-Meteo API (open source, no API key required)
    async fn fetch_from_open_meteo(&self, city: &str) -> Result<WeatherInfo> {
        let (lat, lon) = get_city_coordinates(city)
            .ok_or_else(|| KusanagiError::configuration(format!("Unknown city: {}", city)))?;

        // Open-Meteo API - no API key required!
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true&daily=temperature_2m_max,temperature_2m_min,weathercode&timezone=auto",
            lat, lon
        );

        debug!("Fetching weather for {} from Open-Meteo", city);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalService(format!("Open-Meteo API error: {}", e)))?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(KusanagiError::external_service(format!(
                "Open-Meteo API returned {}: {}",
                status, text
            )));
        }

        let resp = response
            .json::<Value>()
            .await
            .map_err(|e| KusanagiError::serialization(e.to_string()))?;

        // Parse current weather
        let current = &resp["current_weather"];
        let temp = current["temperature"].as_f64().unwrap_or(0.0) as f32;
        let wind_speed = current["windspeed"].as_f64().unwrap_or(0.0) as f32;
        let weather_code = current["weathercode"].as_i64().unwrap_or(0);

        let (description, icon) = map_weather_code(weather_code);

        // Parse forecast
        let forecast = self.parse_open_meteo_forecast(&resp).unwrap_or_default();

        // Open-Meteo doesn't provide humidity/pressure in basic endpoint
        // Using reasonable defaults
        Ok(WeatherInfo {
            city: city.to_string(),
            temp,
            description: description.to_string(),
            icon: icon.to_string(),
            humidity: 60, // Default value
            wind_speed,
            feels_like: temp,  // Simplified
            pressure: 1013,    // Default value
            visibility: 10000, // Default value
            last_updated: chrono::Local::now().format("%H:%M").to_string(),
            forecast,
        })
    }

    /// Parse forecast from Open-Meteo response
    fn parse_open_meteo_forecast(&self, resp: &Value) -> Option<Vec<ForecastDay>> {
        let daily = resp.get("daily")?;
        let times = daily.get("time")?.as_array()?;
        let max_temps = daily.get("temperature_2m_max")?.as_array()?;
        let min_temps = daily.get("temperature_2m_min")?.as_array()?;
        let codes = daily.get("weathercode")?.as_array()?;

        let mut forecast = Vec::new();

        for i in 0..times.len().min(5) {
            let date = times.get(i)?.as_str()?.to_string();
            let temp = ((max_temps.get(i)?.as_f64()? + min_temps.get(i)?.as_f64()?) / 2.0) as f32;
            let code = codes.get(i)?.as_i64()?;
            let (desc, icon) = map_weather_code(code);

            forecast.push(ForecastDay {
                date,
                temp,
                description: desc.to_string(),
                icon: icon.to_string(),
            });
        }

        Some(forecast)
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
        // Try Open-Meteo first (no API key required)
        match self.fetch_from_open_meteo(city).await {
            Ok(info) => return info,
            Err(e) => {
                warn!("Open-Meteo failed for {}: {}, trying fallback", city, e);
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
