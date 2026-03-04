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

pub use alert_repository::{start_background_refresh, AlertRepositoryImpl};
pub use backup_repository::BackupRepositoryImpl;
pub use homeassistant_repository::{create_homeassistant_repository, HomeAssistantRepositoryImpl};
pub use security_repository::{create_security_repository, SecurityRepositoryImpl};
pub use weather_repository::{create_weather_repository, WeatherRepositoryImpl};

pub mod cluster_repository;
pub use cluster_repository::KubernetesClusterRepository;

pub mod kubernetes;
pub use kubernetes::KubernetesRepositoryImpl;

pub mod mock;
pub use mock::{MockClusterRepository, NoOpBackupRepository};
