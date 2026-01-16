use chrono::{DateTime, Utc};
use k8s_openapi::api::batch::v1::{CronJob, Job};
use kube::{
    api::{Api, ListParams},
    Client,
};
use serde::Serialize;
use tracing::info;

/// Backups response for the API
#[derive(Clone, Debug, Serialize)]
pub struct BackupsResponse {
    pub total_cronjobs: usize,
    pub active_jobs: usize,
    pub succeeded_jobs: usize,
    pub failed_jobs: usize,
    pub cronjobs: Vec<CronJobInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CronJobInfo {
    pub name: String,
    pub namespace: String,
    pub schedule: String,
    pub last_schedule: Option<String>,
    pub last_schedule_age: Option<String>,
    pub active_jobs: i32,
    pub suspend: bool,
    pub recent_jobs: Vec<JobInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct JobInfo {
    pub name: String,
    pub status: String, // Running, Succeeded, Failed
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub duration: Option<String>,
}

/// Get backup CronJobs and their recent Jobs
pub async fn get_backups_status() -> Result<BackupsResponse, String> {
    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create Kubernetes client: {}", e))?;

    // Get all CronJobs
    let cronjobs_api: Api<CronJob> = Api::all(client.clone());
    let cronjobs = cronjobs_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list CronJobs: {}", e))?;

    // Get all Jobs
    let jobs_api: Api<Job> = Api::all(client);
    let jobs = jobs_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list Jobs: {}", e))?;

    let now = Utc::now();

    // Process CronJobs
    let mut cronjob_infos: Vec<CronJobInfo> = cronjobs
        .items
        .iter()
        .map(|cj| {
            let name = cj.metadata.name.clone().unwrap_or_default();
            let namespace = cj
                .metadata
                .namespace
                .clone()
                .unwrap_or_else(|| "default".to_string());

            let spec = cj.spec.as_ref();
            let schedule = spec
                .map(|s| s.schedule.clone())
                .unwrap_or_else(|| "Unknown".to_string());

            let status = cj.status.as_ref();
            let last_schedule = status
                .and_then(|s| s.last_schedule_time.as_ref())
                .map(|t| t.0.to_rfc3339());

            let last_schedule_age = status
                .and_then(|s| s.last_schedule_time.as_ref())
                .map(|t| {
                    let ts = DateTime::parse_from_rfc3339(&t.0.to_rfc3339())
                        .ok()
                        .map(|d| d.with_timezone(&Utc));
                    if let Some(ts) = ts {
                        format_duration(now.signed_duration_since(ts))
                    } else {
                        "Unknown".to_string()
                    }
                });

            let active_jobs = status.map(|s| s.active.as_ref().map(|a| a.len()).unwrap_or(0) as i32).unwrap_or(0);
            let suspend = spec.map(|s| s.suspend.unwrap_or(false)).unwrap_or(false);

            // Find recent jobs for this CronJob
            let recent_jobs = get_jobs_for_cronjob(&name, &namespace, &jobs.items, &now);

            CronJobInfo {
                name,
                namespace,
                schedule,
                last_schedule,
                last_schedule_age,
                active_jobs,
                suspend,
                recent_jobs,
            }
        })
        .collect();

    // Sort by namespace, then name
    cronjob_infos.sort_by(|a, b| {
        let ns_cmp = a.namespace.cmp(&b.namespace);
        if ns_cmp == std::cmp::Ordering::Equal {
            a.name.cmp(&b.name)
        } else {
            ns_cmp
        }
    });

    // Calculate statistics from all jobs
    let mut active_count = 0;
    let mut succeeded_count = 0;
    let mut failed_count = 0;

    for job in &jobs.items {
        let status = job.status.as_ref();
        if let Some(status) = status {
            if status.active.unwrap_or(0) > 0 {
                active_count += 1;
            } else if status.succeeded.unwrap_or(0) > 0 {
                succeeded_count += 1;
            } else if status.failed.unwrap_or(0) > 0 {
                failed_count += 1;
            }
        }
    }

    info!(
        "Backups: {} CronJobs, {} Jobs ({} active, {} succeeded, {} failed)",
        cronjob_infos.len(),
        jobs.items.len(),
        active_count,
        succeeded_count,
        failed_count
    );

    Ok(BackupsResponse {
        total_cronjobs: cronjob_infos.len(),
        active_jobs: active_count,
        succeeded_jobs: succeeded_count,
        failed_jobs: failed_count,
        cronjobs: cronjob_infos,
    })
}

/// Get jobs that belong to a specific CronJob
fn get_jobs_for_cronjob(
    cronjob_name: &str,
    namespace: &str,
    all_jobs: &[Job],
    now: &DateTime<Utc>,
) -> Vec<JobInfo> {
    let mut jobs: Vec<JobInfo> = all_jobs
        .iter()
        .filter(|job| {
            // Check if job belongs to this cronjob via owner reference
            let job_ns = job
                .metadata
                .namespace
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or("default");

            if job_ns != namespace {
                return false;
            }

            // Check owner references
            if let Some(owners) = &job.metadata.owner_references {
                return owners.iter().any(|o| o.kind == "CronJob" && o.name == cronjob_name);
            }

            false
        })
        .map(|job| {
            let name = job.metadata.name.clone().unwrap_or_default();
            let status = job.status.as_ref();

            let job_status = if let Some(s) = status {
                if s.active.unwrap_or(0) > 0 {
                    "Running".to_string()
                } else if s.succeeded.unwrap_or(0) > 0 {
                    "Succeeded".to_string()
                } else if s.failed.unwrap_or(0) > 0 {
                    "Failed".to_string()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };

            let started_at = status
                .and_then(|s| s.start_time.as_ref())
                .map(|t| t.0.to_rfc3339());

            let completed_at = status
                .and_then(|s| s.completion_time.as_ref())
                .map(|t| t.0.to_rfc3339());

            let duration = calculate_job_duration(status, now);

            JobInfo {
                name,
                status: job_status,
                started_at,
                completed_at,
                duration,
            }
        })
        .collect();

    // Sort by start time (newest first), limit to 5
    jobs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    jobs.truncate(5);

    jobs
}

fn calculate_job_duration(
    status: Option<&k8s_openapi::api::batch::v1::JobStatus>,
    now: &DateTime<Utc>,
) -> Option<String> {
    let start = status
        .and_then(|s| s.start_time.as_ref())
        .and_then(|t| DateTime::parse_from_rfc3339(&t.0.to_rfc3339()).ok())
        .map(|d| d.with_timezone(&Utc))?;

    let end = status
        .and_then(|s| s.completion_time.as_ref())
        .and_then(|t| DateTime::parse_from_rfc3339(&t.0.to_rfc3339()).ok())
        .map(|d| d.with_timezone(&Utc))
        .unwrap_or(*now);

    let duration = end.signed_duration_since(start);
    Some(format_duration(duration))
}

fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 0 {
        return "just now".to_string();
    }

    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
