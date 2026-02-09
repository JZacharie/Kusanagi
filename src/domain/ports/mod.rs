// Domain Ports - Hexagonal Architecture
use super::entities::{ClusterInfo, NodeInfo, WeatherResponse};
use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

// ==================== Cluster Ports ====================
#[async_trait]
pub trait ClusterRepository: Send + Sync {
    async fn get_cluster_info(&self) -> Result<ClusterInfo>;
    async fn get_nodes(&self) -> Result<Vec<NodeInfo>>;
}

// ==================== Weather Ports ====================
#[async_trait]
pub trait WeatherRepository: Send + Sync {
    /// Get weather for multiple cities with caching
    async fn get_multi_city_weather(&self, force_refresh: bool) -> Result<WeatherResponse>;
    
    /// Force refresh weather data
    async fn force_refresh(&self) -> Result<()>;
}

/// Factory for creating WeatherRepository
pub type WeatherRepositoryFactory = Arc<dyn Fn() -> Box<dyn WeatherRepository> + Send + Sync>;
