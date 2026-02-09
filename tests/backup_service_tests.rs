//! Tests for backup service

use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;

// Mock types
#[derive(Debug, Clone)]
struct Backup {
    name: String,
    namespace: String,
    status: BackupStatus,
    created_at: DateTime<Utc>,
    size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum BackupStatus {
    Completed,
    InProgress,
    Failed,
    Scheduled,
}

#[derive(Debug, Clone)]
struct CronJob {
    name: String,
    namespace: String,
    schedule: String,
    last_run: Option<DateTime<Utc>>,
    next_run: Option<DateTime<Utc>>,
    is_active: bool,
}

// Repository
trait BackupRepository: Send + Sync {
    fn list_backups(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Backup>> + Send + '_>>;
    fn get_cronjobs(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<CronJob>> + Send + '_>>;
    fn trigger_backup(
        &self,
        name: &str,
        namespace: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>>;
}

struct InMemoryBackupRepository {
    backups: Arc<Mutex<Vec<Backup>>>,
    cronjobs: Arc<Mutex<Vec<CronJob>>>,
}

impl InMemoryBackupRepository {
    fn new() -> Self {
        Self {
            backups: Arc::new(Mutex::new(vec![])),
            cronjobs: Arc::new(Mutex::new(vec![])),
        }
    }

    async fn add_backup(&self, backup: Backup) {
        self.backups.lock().await.push(backup);
    }

    async fn add_cronjob(&self, cronjob: CronJob) {
        self.cronjobs.lock().await.push(cronjob);
    }
}

impl BackupRepository for InMemoryBackupRepository {
    fn list_backups(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<Backup>> + Send + '_>> {
        let backups = self.backups.clone();
        Box::pin(async move { backups.lock().await.clone() })
    }

    fn get_cronjobs(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<CronJob>> + Send + '_>> {
        let cronjobs = self.cronjobs.clone();
        Box::pin(async move { cronjobs.lock().await.clone() })
    }

    fn trigger_backup(
        &self,
        name: &str,
        _namespace: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send + '_>> {
        let name = name.to_string();
        Box::pin(async move {
            if name.is_empty() {
                Err("Invalid backup name".to_string())
            } else {
                Ok(())
            }
        })
    }
}

// Service
struct BackupService<R: BackupRepository> {
    repository: Arc<R>,
}

impl<R: BackupRepository> BackupService<R> {
    fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    async fn get_backup_summary(&self) -> BackupSummary {
        let backups = self.repository.list_backups().await;

        BackupSummary {
            total: backups.len(),
            completed: backups
                .iter()
                .filter(|b| b.status == BackupStatus::Completed)
                .count(),
            failed: backups
                .iter()
                .filter(|b| b.status == BackupStatus::Failed)
                .count(),
            in_progress: backups
                .iter()
                .filter(|b| b.status == BackupStatus::InProgress)
                .count(),
            total_size_bytes: backups.iter().map(|b| b.size_bytes).sum(),
        }
    }

    async fn get_recent_backups(&self, hours: i64) -> Vec<Backup> {
        let backups = self.repository.list_backups().await;
        let cutoff = Utc::now() - Duration::hours(hours);

        backups
            .into_iter()
            .filter(|b| b.created_at >= cutoff)
            .collect()
    }

    async fn get_failed_backups(&self) -> Vec<Backup> {
        let backups = self.repository.list_backups().await;
        backups
            .into_iter()
            .filter(|b| b.status == BackupStatus::Failed)
            .collect()
    }

    async fn get_active_cronjobs(&self) -> Vec<CronJob> {
        let cronjobs = self.repository.get_cronjobs().await;
        cronjobs.into_iter().filter(|c| c.is_active).collect()
    }

    async fn get_overdue_cronjobs(&self) -> Vec<CronJob> {
        let cronjobs = self.repository.get_cronjobs().await;
        let now = Utc::now();

        cronjobs
            .into_iter()
            .filter(|c| c.is_active && c.next_run.map(|nr| nr < now).unwrap_or(false))
            .collect()
    }

    async fn trigger_backup(&self, name: &str, namespace: &str) -> Result<String, String> {
        self.repository.trigger_backup(name, namespace).await?;
        Ok(format!("Backup {} triggered successfully", name))
    }
}

#[derive(Debug, Clone)]
struct BackupSummary {
    total: usize,
    completed: usize,
    failed: usize,
    in_progress: usize,
    total_size_bytes: u64,
}

#[tokio::test]
async fn test_backup_summary_empty() {
    let repo = Arc::new(InMemoryBackupRepository::new());
    let service = BackupService::new(repo);

    let summary = service.get_backup_summary().await;

    assert_eq!(summary.total, 0);
    assert_eq!(summary.total_size_bytes, 0);
}

#[tokio::test]
async fn test_backup_summary_with_data() {
    let repo = Arc::new(InMemoryBackupRepository::new());

    repo.add_backup(Backup {
        name: "backup-1".to_string(),
        namespace: "default".to_string(),
        status: BackupStatus::Completed,
        created_at: Utc::now(),
        size_bytes: 1024 * 1024 * 100, // 100 MB
    })
    .await;

    repo.add_backup(Backup {
        name: "backup-2".to_string(),
        namespace: "default".to_string(),
        status: BackupStatus::Completed,
        created_at: Utc::now(),
        size_bytes: 1024 * 1024 * 200, // 200 MB
    })
    .await;

    repo.add_backup(Backup {
        name: "backup-3".to_string(),
        namespace: "default".to_string(),
        status: BackupStatus::Failed,
        created_at: Utc::now(),
        size_bytes: 0,
    })
    .await;

    let service = BackupService::new(repo);
    let summary = service.get_backup_summary().await;

    assert_eq!(summary.total, 3);
    assert_eq!(summary.completed, 2);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.total_size_bytes, 1024 * 1024 * 300);
}

#[tokio::test]
async fn test_get_recent_backups() {
    let repo = Arc::new(InMemoryBackupRepository::new());

    let now = Utc::now();

    repo.add_backup(Backup {
        name: "recent-backup".to_string(),
        namespace: "default".to_string(),
        status: BackupStatus::Completed,
        created_at: now,
        size_bytes: 100,
    })
    .await;

    repo.add_backup(Backup {
        name: "old-backup".to_string(),
        namespace: "default".to_string(),
        status: BackupStatus::Completed,
        created_at: now - Duration::hours(25),
        size_bytes: 100,
    })
    .await;

    let service = BackupService::new(repo);
    let recent = service.get_recent_backups(24).await;

    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].name, "recent-backup");
}

#[tokio::test]
async fn test_get_failed_backups() {
    let repo = Arc::new(InMemoryBackupRepository::new());

    repo.add_backup(Backup {
        name: "failed-1".to_string(),
        namespace: "default".to_string(),
        status: BackupStatus::Failed,
        created_at: Utc::now(),
        size_bytes: 0,
    })
    .await;

    repo.add_backup(Backup {
        name: "completed-1".to_string(),
        namespace: "default".to_string(),
        status: BackupStatus::Completed,
        created_at: Utc::now(),
        size_bytes: 100,
    })
    .await;

    let service = BackupService::new(repo);
    let failed = service.get_failed_backups().await;

    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0].name, "failed-1");
}

#[tokio::test]
async fn test_get_active_cronjobs() {
    let repo = Arc::new(InMemoryBackupRepository::new());

    repo.add_cronjob(CronJob {
        name: "active-job".to_string(),
        namespace: "default".to_string(),
        schedule: "0 0 * * *".to_string(),
        last_run: None,
        next_run: None,
        is_active: true,
    })
    .await;

    repo.add_cronjob(CronJob {
        name: "inactive-job".to_string(),
        namespace: "default".to_string(),
        schedule: "0 0 * * *".to_string(),
        last_run: None,
        next_run: None,
        is_active: false,
    })
    .await;

    let service = BackupService::new(repo);
    let active = service.get_active_cronjobs().await;

    assert_eq!(active.len(), 1);
    assert_eq!(active[0].name, "active-job");
}

#[tokio::test]
async fn test_get_overdue_cronjobs() {
    let repo = Arc::new(InMemoryBackupRepository::new());
    let now = Utc::now();

    repo.add_cronjob(CronJob {
        name: "overdue-job".to_string(),
        namespace: "default".to_string(),
        schedule: "0 0 * * *".to_string(),
        last_run: None,
        next_run: Some(now - Duration::hours(1)),
        is_active: true,
    })
    .await;

    repo.add_cronjob(CronJob {
        name: "future-job".to_string(),
        namespace: "default".to_string(),
        schedule: "0 0 * * *".to_string(),
        last_run: None,
        next_run: Some(now + Duration::hours(1)),
        is_active: true,
    })
    .await;

    let service = BackupService::new(repo);
    let overdue = service.get_overdue_cronjobs().await;

    assert_eq!(overdue.len(), 1);
    assert_eq!(overdue[0].name, "overdue-job");
}

#[tokio::test]
async fn test_trigger_backup_success() {
    let repo = Arc::new(InMemoryBackupRepository::new());
    let service = BackupService::new(repo);

    let result = service.trigger_backup("my-backup", "default").await;

    assert!(result.is_ok());
    assert!(result.unwrap().contains("triggered successfully"));
}

#[tokio::test]
async fn test_trigger_backup_failure() {
    let repo = Arc::new(InMemoryBackupRepository::new());
    let service = BackupService::new(repo);

    let result = service.trigger_backup("", "default").await;

    assert!(result.is_err());
}
