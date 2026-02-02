//! Backup Entities
//!
//! Domain entities for backup operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Backup status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatus {
    pub total_cronjobs: usize,
    pub active_jobs: usize,
    pub succeeded_jobs: usize,
    pub failed_jobs: usize,
    pub cronjobs: Vec<CronJobInfo>,
}

/// CronJob information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobInfo {
    pub name: String,
    pub namespace: String,
    pub schedule: String,
    pub last_schedule: Option<DateTime<Utc>>,
    pub last_schedule_age: Option<String>,
    pub active_jobs: i32,
    pub suspend: bool,
    pub recent_jobs: Vec<JobInfo>,
}

/// Job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub name: String,
    pub status: JobStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration: Option<String>,
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Running,
    Succeeded,
    Failed,
    Unknown,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Succeeded => write!(f, "Succeeded"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Default for JobStatus {
    fn default() -> Self {
        JobStatus::Unknown
    }
}
