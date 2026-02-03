//! Alert Use Cases
//!
//! Application layer use cases for alert operations.

use crate::domain::entities::{Alert, AlertsResponse, AlertStats};
use crate::domain::ports::AlertRepository;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// Get active alerts use case
pub struct GetActiveAlertsUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl GetActiveAlertsUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<AlertsResponse> {
        self.repository.get_active_alerts().await
            .map_err(|e| KusanagiError::internal(format!("Failed to get active alerts: {}", e)))
    }
}

/// Get cached alerts use case
pub struct GetCachedAlertsUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl GetCachedAlertsUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<AlertsResponse> {
        self.repository.get_cached_alerts().await
            .map_err(|e| KusanagiError::internal(format!("Failed to get cached alerts: {}", e)))
    }
}

/// Get alert by fingerprint use case
pub struct GetAlertUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl GetAlertUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, fingerprint: &str) -> Result<Alert> {
        self.repository.get_alert(fingerprint).await
            .map_err(|_e| KusanagiError::not_found("Alert", fingerprint))
    }
}

/// Get alert statistics use case
pub struct GetAlertStatsUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl GetAlertStatsUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<AlertStats> {
        let alerts = self.repository.get_active_alerts().await
            .map_err(|e| KusanagiError::internal(format!("Failed to get alert stats: {}", e)))?;
        
        Ok(AlertStats {
            total: alerts.total,
            critical: alerts.critical.len() as i32,
            warning: alerts.warning.len() as i32,
            info: alerts.info.len() as i32,
            firing: alerts.firing,
            pending: alerts.pending,
        })
    }
}

/// Silence alert use case
pub struct SilenceAlertUseCase {
    repository: Arc<dyn AlertRepository>,
}

impl SilenceAlertUseCase {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, fingerprint: &str, duration_secs: u64) -> Result<()> {
        self.repository.silence_alert(fingerprint, duration_secs).await
            .map_err(|e| KusanagiError::internal(format!("Failed to silence alert: {}", e)))
    }
}

/// Alert service - aggregates all alert use cases
pub struct AlertUseCaseService {
    pub get_active: GetActiveAlertsUseCase,
    pub get_cached: GetCachedAlertsUseCase,
    pub get_alert: GetAlertUseCase,
    pub get_stats: GetAlertStatsUseCase,
    pub silence: SilenceAlertUseCase,
}

impl AlertUseCaseService {
    pub fn new(repository: Arc<dyn AlertRepository>) -> Self {
        Self {
            get_active: GetActiveAlertsUseCase::new(repository.clone()),
            get_cached: GetCachedAlertsUseCase::new(repository.clone()),
            get_alert: GetAlertUseCase::new(repository.clone()),
            get_stats: GetAlertStatsUseCase::new(repository.clone()),
            silence: SilenceAlertUseCase::new(repository),
        }
    }
}
