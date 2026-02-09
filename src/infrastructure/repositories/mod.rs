// Repository Implementations
use crate::domain::entities::{ClusterInfo, NodeInfo};
use crate::domain::ports::ClusterRepository;
use crate::error::Result;
use async_trait::async_trait;

pub mod alert_repository;
pub mod backup_repository;
pub mod homeassistant_repository;
pub mod security_repository;
pub mod weather_repository;

pub use alert_repository::{AlertRepositoryImpl, start_background_refresh};
pub use backup_repository::BackupRepositoryImpl;
pub use homeassistant_repository::{HomeAssistantRepositoryImpl, create_homeassistant_repository};
pub use security_repository::{SecurityRepositoryImpl, create_security_repository};
pub use weather_repository::{WeatherRepositoryImpl, create_weather_repository};

pub struct MockClusterRepository;

#[async_trait]
impl ClusterRepository for MockClusterRepository {
    async fn get_cluster_info(&self) -> Result<ClusterInfo> {
        Ok(ClusterInfo {
            name: "kusanagi-cluster".to_string(),
            version: "v1.28.0".to_string(),
            status: "healthy".to_string(),
            nodes: 3,
        })
    }

    async fn get_nodes(&self) -> Result<Vec<NodeInfo>> {
        Ok(vec![
            NodeInfo {
                name: "master-01".to_string(),
                status: "Ready".to_string(),
                role: "control-plane".to_string(),
            },
            NodeInfo {
                name: "worker-01".to_string(),
                status: "Ready".to_string(),
                role: "worker".to_string(),
            },
        ])
    }
}
