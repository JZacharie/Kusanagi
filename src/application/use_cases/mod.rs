// Use Cases - Business Logic
use crate::domain::entities::{ClusterInfo, NodeInfo, WeatherResponse};
use crate::domain::ports::{ClusterRepository, WeatherRepository};
use crate::error::Result;
use std::sync::Arc;

// ==================== Cluster Use Cases ====================
pub struct ClusterUseCase {
    repository: Arc<dyn ClusterRepository>,
}

impl ClusterUseCase {
    pub fn new(repository: Arc<dyn ClusterRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_cluster_status(&self) -> Result<ClusterInfo> {
        self.repository.get_cluster_info().await
    }

    pub async fn list_nodes(&self) -> Result<Vec<NodeInfo>> {
        self.repository.get_nodes().await
    }
}

// ==================== Weather Use Cases ====================

/// Input data for weather queries
#[derive(Debug, Clone)]
pub struct GetWeatherInput {
    pub force_refresh: bool,
}

impl Default for GetWeatherInput {
    fn default() -> Self {
        Self {
            force_refresh: false,
        }
    }
}

/// Use case for retrieving weather information
pub struct GetWeatherUseCase {
    repository: Arc<dyn WeatherRepository>,
}

impl GetWeatherUseCase {
    /// Create a new use case instance
    pub fn new(repository: Arc<dyn WeatherRepository>) -> Self {
        Self { repository }
    }

    /// Execute the use case
    pub async fn execute(&self, input: GetWeatherInput) -> Result<WeatherResponse> {
        self.repository.get_multi_city_weather(input.force_refresh).await
    }

    /// Force refresh weather data
    pub async fn force_refresh(&self) -> Result<()> {
        self.repository.force_refresh().await
    }
}
