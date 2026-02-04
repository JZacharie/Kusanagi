//! Backup Port
//!
//! Port defining the interface for backup operations.

use async_trait::async_trait;
use crate::error::Result;
use crate::domain::entities::{BackupStatus, CronJobInfo};

/// Port for backup operations
#[async_trait]
pub trait BackupRepository: Send + Sync {
    /// Get backup status
    async fn get_backup_status(&self) -> Result<BackupStatus>;
    
    /// List all CronJobs
    async fn list_cronjobs(&self) -> Result<Vec<CronJobInfo>>;
    
    /// Get CronJobs by namespace
    async fn get_cronjobs_by_namespace(&self, namespace: &str) -> Result<Vec<CronJobInfo>>;
    
    /// Trigger a backup job
    async fn trigger_backup(&self, namespace: &str, name: &str) -> Result<()>;
}
