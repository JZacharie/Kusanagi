use k8s_openapi::api::batch::v1::{CronJob, Job};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::{
    api::{ListParams, PostParams},
    Api, Client,
};
use serde_json::{json, Value};
use tokio::process::Command;

pub async fn get_alerts() -> Result<Value, String> {
    use crate::legacy::alertmanager::get_cached_active_alerts;

    match get_cached_active_alerts().await {
        Ok(alerts_response) => {
            let mut all_alerts = Vec::new();

            for alert in &alerts_response.critical {
                all_alerts.push(json!({
                    "alertname": alert.name,
                    "severity": "critical",
                    "instance": alert.pod.as_ref().or(alert.namespace.as_ref()).unwrap_or(&"unknown".to_string()),
                    "summary": alert.summary,
                    "status": alert.state
                }));
            }

            for alert in &alerts_response.warning {
                all_alerts.push(json!({
                    "alertname": alert.name,
                    "severity": "warning",
                    "instance": alert.pod.as_ref().or(alert.namespace.as_ref()).unwrap_or(&"unknown".to_string()),
                    "summary": alert.summary,
                    "status": alert.state
                }));
            }

            for alert in &alerts_response.info {
                all_alerts.push(json!({
                    "alertname": alert.name,
                    "severity": "info",
                    "instance": alert.pod.as_ref().or(alert.namespace.as_ref()).unwrap_or(&"unknown".to_string()),
                    "summary": alert.summary,
                    "status": alert.state
                }));
            }

            Ok(json!(all_alerts))
        }
        Err(_) => Ok(json!([])),
    }
}

pub async fn get_quotas() -> Result<Value, String> {
    let output = Command::new("kubectl")
        .args(["get", "resourcequota", "--all-namespaces", "-o", "json"])
        .output()
        .await;

    match output {
        Ok(result) if result.status.success() => {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(quota_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(items) = quota_data["items"].as_array() {
                    let mut total_cpu_limit = 0;
                    let mut used_cpu = 0;
                    let mut total_memory_limit = 0;
                    let mut used_memory = 0;

                    for quota in items {
                        if let Some(status) = quota["status"].as_object() {
                            if let Some(hard) = status["hard"].as_object() {
                                if let Some(cpu) = hard["requests.cpu"].as_str() {
                                    total_cpu_limit += parse_cpu_value(cpu);
                                }
                                if let Some(memory) = hard["requests.memory"].as_str() {
                                    total_memory_limit += parse_memory_value(memory);
                                }
                            }
                            if let Some(used) = status["used"].as_object() {
                                if let Some(cpu) = used["requests.cpu"].as_str() {
                                    used_cpu += parse_cpu_value(cpu);
                                }
                                if let Some(memory) = used["requests.memory"].as_str() {
                                    used_memory += parse_memory_value(memory);
                                }
                            }
                        }
                    }

                    return Ok(json!({
                        "cpu": {
                            "used": used_cpu,
                            "total": total_cpu_limit,
                            "percentage": if total_cpu_limit > 0 { (used_cpu * 100) / total_cpu_limit } else { 0 }
                        },
                        "memory": {
                            "used": used_memory,
                            "total": total_memory_limit,
                            "percentage": if total_memory_limit > 0 { (used_memory * 100) / total_memory_limit } else { 0 }
                        },
                        "quotas_count": items.len()
                    }));
                }
            }
        }
        _ => {}
    }

    Ok(json!({
        "cpu": {"used": 50, "total": 100, "percentage": 50},
        "memory": {"used": 60, "total": 100, "percentage": 60},
        "quotas_count": 0
    }))
}

pub async fn get_backups() -> Result<Value, String> {
    // We primarily look for CronJobs with "backup" or "dump" in the name
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let cronjobs_api: Api<CronJob> = Api::all(client.clone());
    let jobs_api: Api<Job> = Api::all(client);

    // Fetch CronJobs
    let cj_list = cronjobs_api
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    // Fetch all Jobs (to avoid N+1 queries)
    let jobs_list = jobs_api
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    // Map Jobs by OwnerReference (UID)
    let mut jobs_by_owner: std::collections::HashMap<String, Vec<Job>> =
        std::collections::HashMap::new();
    for job in jobs_list {
        if let Some(owners) = &job.metadata.owner_references {
            for owner in owners {
                if owner.kind == "CronJob" {
                    jobs_by_owner
                        .entry(owner.uid.clone())
                        .or_default()
                        .push(job.clone());
                }
            }
        }
    }

    let backup_jobs: Vec<Value> = cj_list.items.iter()
        .filter(|job| {
            job.metadata.name.as_deref()
                .map(|name| name.contains("backup") || name.contains("dump"))
                .unwrap_or(false)
        })
        .map(|job| {
            let name = job.metadata.name.as_deref().unwrap_or("");
            let namespace = job.metadata.namespace.as_deref().unwrap_or("");
            let uid = job.metadata.uid.as_deref().unwrap_or("");
            let schedule = job.spec.as_ref().map(|s| s.schedule.as_str()).unwrap_or("");
            let suspend = job.spec.as_ref().map(|s| s.suspend.unwrap_or(false)).unwrap_or(false);

            // Calculate last schedule age
            let last_schedule_age = if let Some(status) = &job.status {
                if let Some(last_time) = &status.last_schedule_time {
                    let created_secs = last_time.0.as_second();
                    let now_secs = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;
                    let age_seconds = now_secs - created_secs;

                    if age_seconds >= 3600 { // If age is 1 hour or more
                        format!("{}h", age_seconds / 3600)
                    } else if age_seconds >= 60 { // If age is 1 minute or more
                        format!("{}m", age_seconds / 60)
                    } else {
                        format!("{}s", age_seconds)
                    }
                } else {
                    "-".to_string()
                }
            } else {
                "-".to_string()
            };

            // Active jobs count
            let active_jobs_count = job.status.as_ref()
                .and_then(|s| s.active.as_ref())
                .map(|a| a.len())
                .unwrap_or(0);

            // Get recent jobs
            let mut recent_jobs_json = Vec::new();
            if let Some(jobs) = jobs_by_owner.get_mut(uid) {
                // Sort by creation timestamp desc
                jobs.sort_by(|a, b| {
                    let t_a = a.metadata.creation_timestamp.as_ref().map(|t| t.0.as_second()).unwrap_or(0);
                    let t_b = b.metadata.creation_timestamp.as_ref().map(|t| t.0.as_second()).unwrap_or(0);
                    t_b.cmp(&t_a)
                });

                // Take 5 most recent
                for job in jobs.iter().take(5) {
                     let status = if job.status.as_ref().map(|s| s.succeeded.unwrap_or(0)).unwrap_or(0) > 0 {
                         "Succeeded"
                     } else if job.status.as_ref().map(|s| s.failed.unwrap_or(0)).unwrap_or(0) > 0 {
                         "Failed"
                     } else if job.status.as_ref().map(|s| s.active.unwrap_or(0)).unwrap_or(0) > 0 {
                         "Running"
                     } else {
                         "Unknown"
                     };

                     recent_jobs_json.push(json!({
                         "name": job.metadata.name.clone().unwrap_or_default(),
                         "status": status,
                         "age": crate::domain::services::kubernetes_service::calculate_age_from_timestamp(
                             job.metadata.creation_timestamp.as_ref().unwrap()
                         )
                     }));
                }
            }

            json!({
                "name": name,
                "namespace": namespace,
                "schedule": schedule,
                "status": if suspend { "suspended" } else if active_jobs_count > 0 { "running" } else { "idle" },
                "suspend": suspend,
                "active_jobs": active_jobs_count,
                "last_schedule_age": last_schedule_age,
                "recent_jobs": recent_jobs_json
            })
        }).collect();

    let total = backup_jobs.len();
    let active = backup_jobs
        .iter()
        .filter(|j| j["active_jobs"].as_u64().unwrap_or(0) > 0)
        .count();

    Ok(json!({
        "total_cronjobs": total,
        "active_jobs": active,
        "succeeded_jobs": 0, // Need to check Jobs history
        "failed_jobs": 0,
        "cronjobs": backup_jobs
    }))
}

pub async fn trigger_cronjob(namespace: &str, cronjob_name: &str) -> Result<String, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let cronjobs: Api<CronJob> = Api::namespaced(client.clone(), namespace);
    let jobs: Api<Job> = Api::namespaced(client, namespace);

    // Get CronJob to fetch spec
    let cronjob = cronjobs
        .get(cronjob_name)
        .await
        .map_err(|e| format!("Failed to get CronJob: {}", e))?;

    let job_template = cronjob
        .spec
        .as_ref()
        .and_then(|s| s.job_template.spec.as_ref())
        .ok_or("CronJob has no job template spec")?;

    let job_name = format!("{}-manual-{}", cronjob_name, chrono::Utc::now().timestamp());

    let mut job = Job {
        metadata: kube::core::ObjectMeta {
            name: Some(job_name.clone()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: Some(job_template.clone()),
        ..Default::default()
    };

    // Add owner reference so it shows up in the list
    if let Some(uid) = cronjob.metadata.uid {
        job.metadata.owner_references = Some(vec![OwnerReference {
            api_version: "batch/v1".to_string(),
            kind: "CronJob".to_string(),
            name: cronjob_name.to_string(),
            uid: uid,
            controller: Some(true),
            block_owner_deletion: Some(true),
        }]);
    }

    jobs.create(&PostParams::default(), &job)
        .await
        .map_err(|e| format!("Failed to create Job: {}", e))?;

    Ok(format!("Job {} created", job_name))
}

fn parse_cpu_value(cpu_str: &str) -> i64 {
    if let Some(stripped) = cpu_str.strip_suffix('m') {
        stripped.parse::<i64>().unwrap_or(0)
    } else {
        cpu_str.parse::<i64>().unwrap_or(0) * 1000
    }
}

fn parse_memory_value(memory_str: &str) -> i64 {
    let memory_str = memory_str.trim();
    if let Some(stripped) = memory_str.strip_suffix("Gi") {
        stripped.parse::<i64>().unwrap_or(0) * 1024 * 1024 * 1024
    } else if let Some(stripped) = memory_str.strip_suffix("Mi") {
        stripped.parse::<i64>().unwrap_or(0) * 1024 * 1024
    } else if let Some(stripped) = memory_str.strip_suffix("Ki") {
        stripped.parse::<i64>().unwrap_or(0) * 1024
    } else {
        memory_str.parse::<i64>().unwrap_or(0)
    }
}
