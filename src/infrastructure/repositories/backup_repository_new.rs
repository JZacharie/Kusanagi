use async_trait::async_trait;
use crate::domain::ports::{BackupRepository, BackupStatus, CronJobInfo};
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacyBackupRepository;

#[async_trait]
impl BackupRepository for LegacyBackupRepository {
    async fn get_status(&self) -> Result<BackupStatus> {
        legacy::backups::get_backups_status().await
            .map_err(|e| KusanagiError::external_api("Backup", &e.to_string()))
    }

    async fn list_cronjobs(&self) -> Result<Vec<CronJobInfo>> {
        // Would map from legacy backup module
        Ok(vec![])
    }

    async fn trigger_backup(&self, backup_name: &str) -> Result<()> {
        // Would trigger backup via legacy module
        Ok(())
    }
}
