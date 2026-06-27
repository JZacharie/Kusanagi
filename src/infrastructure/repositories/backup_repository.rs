//! Backup Repository Implementation
//!
//! Infrastructure adapter implementing the BackupRepository port.
//! Handles Kubernetes CronJob and Job operations.

use crate::domain::entities::{BackupsResponse, CronJobInfo, JobInfo};
use crate::domain::ports::BackupRepository;
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use kube::{
    api::{Api, ListParams, PostParams},
    Client,
};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::info;

/// Cache entry for backups
struct BackupCache {
    data: BackupsResponse,
    timestamp: Instant,
}

/// Backup repository implementation with caching
pub struct BackupRepositoryImpl {
    client: Arc<Client>,
    cache: Mutex<Option<BackupCache>>,
}

impl BackupRepositoryImpl {
    /// Create a new repository instance
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            client,
            cache: Mutex::new(None),
        }
    }

    /// Cache TTL in seconds
    const CACHE_TTL: Duration = Duration::from_secs(30);

    /// Check if cache is valid
    fn get_cached(&self) -> Option<BackupsResponse> {
        let cache = self.cache.lock().ok()?;
        if let Some(ref entry) = *cache {
            if entry.timestamp.elapsed() < Self::CACHE_TTL {
                return Some(BackupsResponse {
                    total_cronjobs: entry.data.total_cronjobs,
                    active_jobs: entry.data.active_jobs,
                    succeeded_jobs: entry.data.succeeded_jobs,
                    failed_jobs: entry.data.failed_jobs,
                    cronjobs: entry.data.cronjobs.clone(),
                });
            }
        }
        None
    }

    /// Store in cache
    fn set_cached(&self, data: BackupsResponse) {
        if let Ok(mut cache) = self.cache.lock() {
            *cache = Some(BackupCache {
                data,
                timestamp: Instant::now(),
            });
        }
    }

    /// Format duration as human-readable string
    fn format_duration(&self, duration: chrono::Duration) -> String {
        let days = duration.num_days();
        let hours = duration.num_hours() % 24;
        let minutes = duration.num_minutes() % 60;

        if days > 0 {
            format!("{}d {}h ago", days, hours)
        } else if hours > 0 {
            format!("{}h {}m ago", hours, minutes)
        } else {
            format!("{}m ago", minutes)
        }
    }

    /// Get jobs for a specific CronJob
    fn get_jobs_for_cronjob(
        &self,
        cronjob_name: &str,
        namespace: &str,
        jobs: &[Job],
        _now: &DateTime<Utc>,
    ) -> Vec<JobInfo> {
        jobs.iter()
            .filter(|job| {
                job.metadata
                    .owner_references
                    .as_ref()
                    .map(|refs| {
                        refs.iter()
                            .any(|r| r.name == cronjob_name && r.kind == "CronJob")
                    })
                    .unwrap_or(false)
                    && job
                        .metadata
                        .namespace
                        .as_ref()
                        .map(|ns| ns == namespace)
                        .unwrap_or(false)
            })
            .map(|job| {
                let name = job.metadata.name.clone().unwrap_or_default();
                let status = job.status.as_ref();

                let job_status = if status.and_then(|s| s.active.as_ref()).is_some() {
                    "Running"
                } else if status.and_then(|s| s.succeeded).unwrap_or(0) > 0 {
                    "Succeeded"
                } else if status.and_then(|s| s.failed).unwrap_or(0) > 0 {
                    "Failed"
                } else {
                    "Unknown"
                };

                let started_at = status
                    .and_then(|s| s.start_time.as_ref())
                    .map(|t| format!("{:?}", t.0));

                let completed_at = status
                    .and_then(|s| s.completion_time.as_ref())
                    .map(|t| format!("{:?}", t.0));

                let duration = if let (Some(start), Some(end)) = (&started_at, &completed_at) {
                    if let (Ok(s), Ok(e)) = (
                        DateTime::parse_from_rfc3339(start),
                        DateTime::parse_from_rfc3339(end),
                    ) {
                        let secs = (e.timestamp() - s.timestamp()) as f64;
                        Some(format!("{:.1}s", secs))
                    } else {
                        None
                    }
                } else {
                    None
                };

                JobInfo {
                    name,
                    status: job_status.to_string(),
                    started_at,
                    completed_at,
                    duration,
                }
            })
            .take(5) // Keep only 5 most recent
            .collect()
    }
}

#[async_trait]
impl BackupRepository for BackupRepositoryImpl {
    async fn get_backups_status(&self) -> Result<BackupsResponse> {
        // Check cache first
        if let Some(cached) = self.get_cached() {
            info!(
                "📦 Returning cached backups data ({} cronjobs)",
                cached.total_cronjobs
            );
            return Ok(cached);
        }

        // Get all CronJobs
        let cronjobs_api: Api<CronJob> = Api::all(self.client.as_ref().clone());
        let cronjobs = cronjobs_api
            .list(&ListParams::default())
            .await
            .map_err(|e| {
                KusanagiError::external_service(format!("Failed to list CronJobs: {}", e))
            })?;

        // Get all Jobs
        let jobs_api: Api<Job> = Api::all(self.client.as_ref().clone());
        let jobs = jobs_api
            .list(&ListParams::default())
            .await
            .map_err(|e| KusanagiError::external_service(format!("Failed to list Jobs: {}", e)))?;

        let now = Utc::now();

        // Process CronJobs
        let mut cronjob_infos: Vec<CronJobInfo> = Vec::new();
        let mut total_active = 0;
        let mut total_succeeded = 0;
        let mut total_failed = 0;

        for cj in &cronjobs.items {
            let name = cj.metadata.name.clone().unwrap_or_default();
            let namespace = cj
                .metadata
                .namespace
                .clone()
                .unwrap_or_else(|| "default".to_string());

            let schedule = cj.spec.schedule.clone();

            let status = cj.status.as_ref();
            let last_schedule = status
                .and_then(|s| s.last_schedule_time.as_ref())
                .map(|t| format!("{:?}", t.0));

            let last_schedule_age = status.and_then(|s| s.last_schedule_time.as_ref()).map(|t| {
                let ts_str = format!("{:?}", t.0);
                let ts = DateTime::parse_from_rfc3339(&ts_str)
                    .ok()
                    .map(|d| d.with_timezone(&Utc));
                if let Some(ts) = ts {
                    self.format_duration(now.signed_duration_since(ts))
                } else {
                    "Unknown".to_string()
                }
            });

            let active_jobs = status
                .map(|s| s.active.as_ref().map(|a| a.len()).unwrap_or(0) as i32)
                .unwrap_or(0);
            let suspend = cj.spec.suspend.unwrap_or(false);

            // Find recent jobs for this CronJob
            let recent_jobs = self.get_jobs_for_cronjob(&name, &namespace, &jobs.items, &now);

            // Count job statuses
            for job in &recent_jobs {
                match job.status.as_str() {
                    "Running" => total_active += 1,
                    "Succeeded" => total_succeeded += 1,
                    "Failed" => total_failed += 1,
                    _ => {}
                }
            }

            cronjob_infos.push(CronJobInfo {
                name,
                namespace,
                schedule,
                last_schedule,
                last_schedule_age,
                active_jobs,
                suspend,
                recent_jobs,
            });
        }

        let response = BackupsResponse {
            total_cronjobs: cronjob_infos.len(),
            active_jobs: total_active,
            succeeded_jobs: total_succeeded,
            failed_jobs: total_failed,
            cronjobs: cronjob_infos,
        };

        // Store in cache
        self.set_cached(response.clone());
        info!(
            "💾 Cached backups data ({} cronjobs)",
            response.total_cronjobs
        );

        Ok(response)
    }

    async fn trigger_backup(&self, namespace: &str, name: &str) -> Result<String> {
        let cronjobs_api: Api<CronJob> = Api::namespaced(self.client.as_ref().clone(), namespace);

        // Get the CronJob
        let cronjob = cronjobs_api.get(name).await.map_err(|e| {
            KusanagiError::external_service(format!("Failed to get CronJob: {}", e))
        })?;

        // Create a Job from the CronJob template
        let job_spec = cronjob
            .spec
            .job_template
            .spec
            .ok_or_else(|| KusanagiError::configuration("CronJob has no job template"))?;

        let job = Job {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(format!(
                    "{}-manual-{}",
                    name,
                    chrono::Utc::now().timestamp()
                )),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            spec: Some(job_spec),
            status: None,
        };

        let jobs_api: Api<Job> = Api::namespaced(self.client.as_ref().clone(), namespace);
        jobs_api
            .create(&PostParams::default(), &job)
            .await
            .map_err(|e| KusanagiError::external_service(format!("Failed to create Job: {}", e)))?;

        info!("Triggered backup for CronJob {}/{}", namespace, name);
        Ok(format!("Backup triggered for {}/{}", namespace, name))
    }
}
