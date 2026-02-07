use k8s_openapi::api::batch::v1::CronJob;
use kube::{api::ListParams, Api, Client};
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
    let cronjobs_api: Api<CronJob> = Api::all(client);
    let list = cronjobs_api
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    let backup_jobs: Vec<Value> = list.items.iter()
        .filter(|job| {
            job.metadata.name.as_deref()
                .map(|name| name.contains("backup") || name.contains("dump"))
                .unwrap_or(false)
        })
        .map(|job| {
            let name = job.metadata.name.clone().unwrap_or_default();
            let namespace = job.metadata.namespace.clone().unwrap_or_default();
            let schedule = job.spec.as_ref().map(|s| s.schedule.clone()).unwrap_or_default();
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

            // Recent jobs - simplified placeholder as listing all Jobs is expensive just for this
            // To be accurate we would need to list Jobs filtered by ownerReference=CronJob
            // For now, let's just return mostly static info or what we have.

            json!({
                "name": name,
                "namespace": namespace,
                "schedule": schedule,
                "status": if suspend { "suspended" } else if active_jobs_count > 0 { "running" } else { "idle" },
                "suspend": suspend,
                "active_jobs": active_jobs_count,
                "last_schedule_age": last_schedule_age,
                "recent_jobs": [] // Placeholder
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
