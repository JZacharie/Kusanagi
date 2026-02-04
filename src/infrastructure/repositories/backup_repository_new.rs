use async_trait::async_trait;
use crate::domain::ports::{BackupRepository, BackupStatus, CronJobInfo};
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacyBackupRepository;

#[async_trait]
impl BackupRepository for LegacyBackupRepository {
    async fn get_backup_status(&self) -> Result<BackupStatus, String> {
        Ok(BackupStatus::default())
    }

    async fn get_status(&self) -> Result<BackupStatus> {
        Ok(BackupStatus::default())
    }

    async fn list_cronjobs(&self) -> Result<Vec<CronJobInfo>> {
        Ok(vec![])
    }

    async fn get_cronjobs_by_namespace(&self, _namespace: &str) -> Result<Vec<CronJobInfo>, String> {
        Ok(vec![])
    }

    async fn trigger_backup(&self, _namespace: &str, _name: &str) -> Result<(), String> {
        Ok(())
    }
}
