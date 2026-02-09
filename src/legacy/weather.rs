use actix_web::{web, HttpResponse, Result};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{config::Region, Client as S3Client};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, error, info, warn};

const S3_BUCKET: &str = "kusanagi";
const S3_KEY: &str = "weather-cache.json";
const S3_REGION: &str = "us-east-1";
const MINIO_ENDPOINT: &str = "http://192.168.0.170:9010"; // Hardcoded from chat_storage.rs for consistency

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
    s3_client: Option<S3Client>,
}

impl WeatherClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("OPENWEATHER_API_KEY").unwrap_or_else(|_| "".to_string());

        if api_key.is_empty() {
            warn!("OPENWEATHER_API_KEY not set, using mock weather data");
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?;

        // Initialize S3 Client (MinIO)
        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(S3_REGION))
            .endpoint_url(MINIO_ENDPOINT)
            .load()
            .await;

        let s3_client = Some(S3Client::new(&config));

        Ok(Self {
            api_key,
            client,
            s3_client,
        })
    }

    pub async fn get_multi_city_weather(
        &self,
        force_refresh: bool,
    ) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
        // 1. Check S3 cache first if not forced refresh
        if !force_refresh {
            if let Some(cached) = self.fetch_from_s3().await {
                // Check if cache is fresh (less than 6 hours)
                if let Ok(cached_time) =
                    chrono::NaiveDateTime::parse_from_str(&cached.cached_at, "%Y-%m-%d %H:%M:%S")
                {
                    let six_hours_ago =
                        chrono::Local::now().naive_local() - chrono::Duration::hours(6);
                    if cached_time > six_hours_ago {
                        debug!(
                            "Returning cached weather data from S3 (cached at {})",
                            cached.cached_at
                        );
                        return Ok(cached);
                    } else {
                        debug!(
                            "S3 cache expired (cached at {}), refreshing...",
                            cached.cached_at
                        );
                    }
                }
            }
        }

        let cities = vec!["Lyon", "Mexico City", "New York"];
        let mut results = Vec::new();

        if self.api_key.is_empty() {
            debug!("OPENWEATHER_API_KEY is empty, trying wttr.in fallback for all cities");
            for city in cities {
                match self.fetch_fallback_weather(city).await {
                    Ok(info) => results.push(info),
                    Err(e) => {
                        warn!(
                            "wttr.in fallback failed for {}: {}, using final mock fallback",
                            city, e
                        );
                        results.push(self.get_mock_city_weather(city));
                    }
                }
            }
        } else {
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
        }

        let response = WeatherResponse {
            cities: results,
            cached_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        // 2. Save to S3
        if let Err(e) = self.save_to_s3(&response).await {
            error!("Failed to save weather to S3: {}", e);
        }

        Ok(response)
    }

    async fn fetch_from_s3(&self) -> Option<WeatherResponse> {
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
                    Ok(weather) => Some(weather),
                    Err(e) => {
                        error!("Failed to parse S3 weather cache: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                debug!(
                    "Available buckets: {:?}",
                    client.list_buckets().send().await
                );
                warn!("Failed to fetch from S3 (might apply first run): {}", e);
                None
            }
        }
    }

    async fn save_to_s3(
        &self,
        weather: &WeatherResponse,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.s3_client.as_ref().ok_or("No S3 client")?;
        let body = serde_json::to_string(weather)?;

        client
            .put_object()
            .bucket(S3_BUCKET)
            .key(S3_KEY)
            .body(body.into_bytes().into())
            .send()
            .await?;

        info!("Weather data cached to S3 successfully");
        Ok(())
    }

    async fn fetch_city_weather(
        &self,
        city: &str,
    ) -> Result<WeatherInfo, Box<dyn std::error::Error>> {
        // Current weather
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );
        debug!("Fetching current weather for {} from URL: {}", city, url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

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
        let forecast_url = format!(
            "https://api.openweathermap.org/data/2.5/forecast?q={}&appid={}&units=metric",
            city, self.api_key
        );
        let forecast_resp = self
            .client
            .get(&forecast_url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let mut forecast = Vec::new();
        if let Some(list) = forecast_resp["list"].as_array() {
            // Pick one entry per day (roughly mid-day)
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

    async fn fetch_fallback_weather(
        &self,
        city: &str,
    ) -> Result<WeatherInfo, Box<dyn std::error::Error>> {
        let url = format!("https://wttr.in/{}?format=j1", city);
        let resp = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

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

        // Simple icon mapping for wttr.in condition codes or text
        let icon_code = match description.to_lowercase().as_str() {
            d if d.contains("sunny") || d.contains("clear") => "01d",
            d if d.contains("partly cloudy") => "02d",
            d if d.contains("cloudy") || d.contains("overcast") => "04d",
            d if d.contains("rain") || d.contains("drizzle") => "10d",
            d if d.contains("snow") => "13d",
            d if d.contains("thunder") || d.contains("storm") => "11d",
            _ => "03d",
        };

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
                    icon: "03d".to_string(), // Default icon for forecast in fallback
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

    fn get_mock_city_weather(&self, city: &str) -> WeatherInfo {
        let (temp, desc, icon) = match city {
            "Lyon" => (5.2, "Cloudy", "04d"),
            "Mexico City" => (22.5, "Sunny", "01d"),
            "New York" => (-2.1, "Snowing", "13d"),
            _ => (15.0, "Clear", "01d"),
        };

        let mut forecast = Vec::new();
        for i in 1..=5 {
            let forecast_date = (chrono::Local::now() + chrono::Duration::days(i))
                .format("%Y-%m-%d 12:00:00")
                .to_string();

            let (f_temp, f_desc, f_icon) = match i {
                1 => (temp + 2.0, "Partly Cloudy", "02d"),
                2 => (temp + 1.0, "Cloudy", "03d"),
                3 => (temp - 1.0, "Rainy", "10d"),
                4 => (temp - 2.0, "Stormy", "11d"),
                _ => (temp + 3.0, "Sunny", "01d"),
            };

            forecast.push(ForecastDay {
                date: forecast_date,
                temp: f_temp,
                description: f_desc.to_string(),
                icon: f_icon.to_string(),
            });
        }

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

// Public helper for startup refresh
pub async fn force_refresh() -> Result<(), Box<dyn std::error::Error>> {
    let client = WeatherClient::new().await?;
    info!("Forcing weather refresh...");
    client.get_multi_city_weather(true).await?;
    Ok(())
}

// API Handlers
pub async fn get_weather_handler() -> Result<HttpResponse> {
    match WeatherClient::new().await {
        Ok(client) => match client.get_multi_city_weather(false).await {
            Ok(weather) => Ok(HttpResponse::Ok().json(weather)),
            Err(e) => {
                error!("Weather error: {}", e);
                Ok(HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": e.to_string()})))
            }
        },
        Err(e) => {
            error!("Failed to create weather client: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": e.to_string()})))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/weather").route("/current", web::get().to(get_weather_handler)));
}
