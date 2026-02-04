use crate::domain::ports::{BackupRepository, BackupStatus, CronJobInfo};
use crate::error::Result;
use std::sync::Arc;

pub struct GetBackupStatusUseCase {
    backup_repo: Arc<dyn BackupRepository>,
}

impl GetBackupStatusUseCase {
    pub fn new(backup_repo: Arc<dyn BackupRepository>) -> Self {
        Self { backup_repo }
    }

    pub async fn execute(&self) -> Result<BackupStatus> {
        self.backup_repo.get_status().await
    }
}

pub struct TriggerBackupUseCase {
    backup_repo: Arc<dyn BackupRepository>,
}

impl TriggerBackupUseCase {
    pub fn new(backup_repo: Arc<dyn BackupRepository>) -> Self {
        Self { backup_repo }
    }

    pub async fn execute(&self, backup_name: &str) -> Result<()> {
        self.backup_repo.trigger_backup(backup_name).await
    }
}
