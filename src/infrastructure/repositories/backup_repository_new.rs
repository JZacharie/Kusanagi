use async_trait::async_trait;
use crate::domain::ports::BackupRepository;
use crate::domain::entities::{BackupStatus, CronJobInfo};
use crate::error::Result;

pub struct LegacyBackupRepository;

#[async_trait]
impl BackupRepository for LegacyBackupRepository {
    async fn get_backup_status(&self) -> Result<BackupStatus, String> {
        Ok(BackupStatus {
            last_backup: chrono::Utc::now(),
            status: "completed".to_string(),
            size_bytes: 1024 * 1024 * 100,
            total_cronjobs: 0,
            active_jobs: 0,
            succeeded_jobs: 0,
            failed_jobs: 0,
            cronjobs: vec![],
        })
    }

    async fn list_cronjobs(&self) -> Result<Vec<CronJobInfo>, String> {
        Ok(vec![])
    }

    async fn get_cronjobs_by_namespace(&self, _namespace: &str) -> Result<Vec<CronJobInfo>, String> {
        Ok(vec![])
    }

    async fn trigger_backup(&self, _namespace: &str, _name: &str) -> Result<(), String> {
        Ok(())
    }
}
