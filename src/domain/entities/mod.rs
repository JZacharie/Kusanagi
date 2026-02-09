// Core Domain Entities
use serde::{Deserialize, Serialize};

// ==================== Cluster Entities ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub name: String,
    pub version: String,
    pub status: String,
    pub nodes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
    pub role: String,
}

// ==================== Weather Entities ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastDay {
    pub date: String,
    pub temp: f32,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub cities: Vec<WeatherInfo>,
    pub cached_at: String,
}
