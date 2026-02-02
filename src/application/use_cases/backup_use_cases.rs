//! Backup Use Cases
//!
//! Application layer use cases for backup operations.

use crate::domain::entities::{BackupStatus, CronJobInfo};
use crate::domain::ports::BackupRepository;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// Get backup status use case
pub struct GetBackupStatusUseCase {
    repository: Arc<dyn BackupRepository>,
}

impl GetBackupStatusUseCase {
    pub fn new(repository: Arc<dyn BackupRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<BackupStatus> {
        self.repository.get_backup_status().await
            .map_err(|e| KusanagiError::internal(format!("Failed to get backup status: {}", e)))
    }
}

/// List CronJobs use case
pub struct ListCronJobsUseCase {
    repository: Arc<dyn BackupRepository>,
}

impl ListCronJobsUseCase {
    pub fn new(repository: Arc<dyn BackupRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<Vec<CronJobInfo>> {
        match namespace {
            Some(ns) => self.repository.get_cronjobs_by_namespace(ns).await,
            None => self.repository.list_cronjobs().await,
        }.map_err(|e| KusanagiError::internal(format!("Failed to list CronJobs: {}", e)))
    }
}

/// Trigger backup use case
pub struct TriggerBackupUseCase {
    repository: Arc<dyn BackupRepository>,
}

impl TriggerBackupUseCase {
    pub fn new(repository: Arc<dyn BackupRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: &str, name: &str) -> Result<()> {
        self.repository.trigger_backup(namespace, name).await
            .map_err(|e| KusanagiError::internal(format!("Failed to trigger backup: {}", e)))
    }
}

/// Backup statistics DTO
#[derive(Debug, Clone, serde::Serialize)]
pub struct BackupStats {
    pub total_cronjobs: usize,
    pub active_jobs: usize,
    pub succeeded_jobs: usize,
    pub failed_jobs: usize,
    pub healthy_percentage: f64,
}

/// Get backup statistics use case
pub struct GetBackupStatsUseCase {
    repository: Arc<dyn BackupRepository>,
}

impl GetBackupStatsUseCase {
    pub fn new(repository: Arc<dyn BackupRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<BackupStats> {
        let status = self.repository.get_backup_status().await
            .map_err(|e| KusanagiError::internal(format!("Failed to get backup stats: {}", e)))?;
        
        let total = status.total_cronjobs;
        let healthy = if total > 0 {
            (status.succeeded_jobs as f64 / total as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(BackupStats {
            total_cronjobs: total,
            active_jobs: status.active_jobs,
            succeeded_jobs: status.succeeded_jobs,
            failed_jobs: status.failed_jobs,
            healthy_percentage: healthy,
        })
    }
}

/// Backup service - aggregates all backup use cases
pub struct BackupUseCaseService {
    pub get_status: GetBackupStatusUseCase,
    pub list_cronjobs: ListCronJobsUseCase,
    pub trigger_backup: TriggerBackupUseCase,
    pub get_stats: GetBackupStatsUseCase,
}

impl BackupUseCaseService {
    pub fn new(repository: Arc<dyn BackupRepository>) -> Self {
        Self {
            get_status: GetBackupStatusUseCase::new(repository.clone()),
            list_cronjobs: ListCronJobsUseCase::new(repository.clone()),
            trigger_backup: TriggerBackupUseCase::new(repository.clone()),
            get_stats: GetBackupStatsUseCase::new(repository),
        }
    }
}
