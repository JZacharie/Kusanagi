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

// ==================== Alert Ports ====================
use super::entities::AlertsResponse;

#[async_trait]
pub trait AlertRepository: Send + Sync {
    /// Get all active alerts from Alertmanager
    async fn get_active_alerts(&self) -> Result<AlertsResponse>;

    /// Get cached active alerts (with cache logic)
    async fn get_cached_alerts(&self) -> Result<AlertsResponse>;

    /// Force refresh alerts cache
    async fn refresh_alerts(&self) -> Result<AlertsResponse>;

    /// Check if running in local mode (mock data)
    fn is_local_mode(&self) -> bool;
}

// ==================== Backup Ports ====================
use super::entities::BackupsResponse;

#[async_trait]
pub trait BackupRepository: Send + Sync {
    /// Get backup status (CronJobs and Jobs)
    async fn get_backups_status(&self) -> Result<BackupsResponse>;

    /// Trigger a CronJob manually
    async fn trigger_backup(&self, namespace: &str, name: &str) -> Result<String>;
}

// ==================== HomeAssistant Ports ====================
use super::entities::{HomeAssistantDevicesResponse, HomeAssistantSensorsResponse};

#[async_trait]
pub trait HomeAssistantRepository: Send + Sync {
    /// Get all sensors from Home Assistant
    async fn get_sensors(&self) -> Result<HomeAssistantSensorsResponse>;

    /// Get all devices from Home Assistant
    async fn get_devices(&self) -> Result<HomeAssistantDevicesResponse>;
}

// ==================== Security Ports ====================
use super::entities::{SecurityReport, SecuritySummary};

#[async_trait]
pub trait SecurityRepository: Send + Sync {
    /// Get security summary across all reports
    async fn get_security_summary(&self) -> Result<SecuritySummary>;

    /// Get list of all security report keys
    async fn get_security_reports(&self) -> Result<Vec<String>>;

    /// Get a specific security report by category and name
    async fn get_security_report(&self, category: &str, name: &str) -> Result<SecurityReport>;

    /// Trigger a manual vulnerability scan
    async fn trigger_scan(&self) -> Result<String>;

    /// Check if running in local mode (mock data)
    fn is_local_mode(&self) -> bool;
}

pub mod kubernetes;
pub mod a2ui_repository;

pub use kubernetes::KubernetesRepository;
pub use a2ui_repository::A2UIRepository;

// ==================== Transcription Ports ====================
#[async_trait]
pub trait TranscriptionRepository: Send + Sync {
    /// Store a transcription in persistent storage
    async fn store_transcription(&self, filename: &str, text: &str) -> Result<String>;
}
