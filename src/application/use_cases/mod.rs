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

// ==================== Alert Use Cases ====================
use crate::domain::entities::AlertsResponse;
use crate::domain::ports::AlertRepository;

/// Input data for alert queries
#[derive(Debug, Clone)]
pub struct GetAlertsInput {
    pub force_refresh: bool,
}

impl Default for GetAlertsInput {
    fn default() -> Self {
        Self {
            force_refresh: false,
        }
    }
}

/// Use case for retrieving alerts
pub struct GetAlertsUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl GetAlertsUseCase {
    /// Create a new use case instance
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    /// Execute the use case
    pub async fn execute(&self, input: GetAlertsInput) -> Result<AlertsResponse> {
        if input.force_refresh {
            self.repository.refresh_alerts().await
        } else {
            self.repository.get_cached_alerts().await
        }
    }

    /// Get active alerts (bypass cache)
    pub async fn get_active_alerts(&self) -> Result<AlertsResponse> {
        self.repository.get_active_alerts().await
    }

    /// Get cached alerts
    pub async fn get_cached_alerts(&self) -> Result<AlertsResponse> {
        self.repository.get_cached_alerts().await
    }

    /// Refresh alerts cache
    pub async fn refresh_alerts(&self) -> Result<AlertsResponse> {
        self.repository.refresh_alerts().await
    }

    /// Check if running in local mode
    pub fn is_local_mode(&self) -> bool {
        self.repository.is_local_mode()
    }
}

// ==================== Backup Use Cases ====================
use crate::domain::entities::BackupsResponse;
use crate::domain::ports::BackupRepository;

/// Use case for backup operations
pub struct BackupUseCase {
    repository: Arc<dyn BackupRepository>,
}

impl BackupUseCase {
    /// Create a new use case instance
    pub fn new(repository: Arc<dyn BackupRepository>) -> Self {
        Self { repository }
    }

    /// Get backup status
    pub async fn get_backups_status(&self) -> Result<BackupsResponse> {
        self.repository.get_backups_status().await
    }

    /// Trigger a backup
    pub async fn trigger_backup(&self, namespace: &str, name: &str) -> Result<String> {
        self.repository.trigger_backup(namespace, name).await
    }
}

// ==================== Security Use Cases ====================
use crate::domain::entities::{SecurityReport, SecuritySummary};
use crate::domain::ports::SecurityRepository;

/// Use case for security operations
pub struct GetSecurityUseCase {
    repository: Arc<dyn SecurityRepository>,
}

impl GetSecurityUseCase {
    /// Create a new use case instance
    pub fn new(repository: Arc<dyn SecurityRepository>) -> Self {
        Self { repository }
    }

    /// Get security summary across all reports
    pub async fn get_summary(&self) -> Result<SecuritySummary> {
        self.repository.get_security_summary().await
    }

    /// Get list of all security reports
    pub async fn get_reports(&self) -> Result<Vec<String>> {
        self.repository.get_security_reports().await
    }

    /// Get a specific security report
    pub async fn get_report(&self, category: &str, name: &str) -> Result<SecurityReport> {
        self.repository.get_security_report(category, name).await
    }

    /// Check if running in local mode
    pub fn is_local_mode(&self) -> bool {
        self.repository.is_local_mode()
    }
}

// ==================== HomeAssistant Use Cases ====================
use crate::domain::entities::{HomeAssistantSensorsResponse, HomeAssistantDevicesResponse};
use crate::domain::ports::HomeAssistantRepository;

/// Input data for HomeAssistant queries
#[derive(Debug, Clone)]
pub struct GetHomeAssistantInput {
    pub force_refresh: bool,
}

impl Default for GetHomeAssistantInput {
    fn default() -> Self {
        Self {
            force_refresh: false,
        }
    }
}

/// Use case for retrieving HomeAssistant information
pub struct GetHomeAssistantUseCase {
    repository: Arc<dyn HomeAssistantRepository>,
}

impl GetHomeAssistantUseCase {
    /// Create a new use case instance
    pub fn new(repository: Arc<dyn HomeAssistantRepository>) -> Self {
        Self { repository }
    }

    /// Get sensors from Home Assistant
    pub async fn get_sensors(&self) -> Result<HomeAssistantSensorsResponse> {
        self.repository.get_sensors().await
    }

    /// Get devices from Home Assistant
    pub async fn get_devices(&self) -> Result<HomeAssistantDevicesResponse> {
        self.repository.get_devices().await
    }
}
