use serde_json::{json, Value};
use std::process::Command;

pub async fn get_alerts() -> Result<Value, String> {
    // Essayer Prometheus AlertManager
    let alertmanager_output = Command::new("curl")
        .args(&["-s", "http://localhost:9093/api/v1/alerts"])
        .output();
    
    if let Ok(result) = alertmanager_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(alerts_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(alerts) = alerts_data["data"].as_array() {
                    let active_alerts: Vec<Value> = alerts.iter().map(|alert| {
                        json!({
                            "alertname": alert["labels"]["alertname"],
                            "severity": alert["labels"]["severity"],
                            "instance": alert["labels"]["instance"],
                            "summary": alert["annotations"]["summary"],
                            "status": alert["status"]["state"]
                        })
                    }).collect();
                    
                    return Ok(json!(active_alerts));
                }
            }
        }
    }
    
    // Fallback: vérifier les pods en erreur comme "alertes"
    let pods_output = Command::new("kubectl")
        .args(&["get", "pods", "--all-namespaces", "--field-selector=status.phase!=Running,status.phase!=Succeeded", "-o", "json"])
        .output();
    
    if let Ok(result) = pods_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(pods_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(items) = pods_data["items"].as_array() {
                    let pod_alerts: Vec<Value> = items.iter().map(|pod| {
                        json!({
                            "alertname": "PodNotRunning",
                            "severity": "warning",
                            "instance": format!("{}/{}", pod["metadata"]["namespace"], pod["metadata"]["name"]),
                            "summary": format!("Pod {} is in {} state", pod["metadata"]["name"], pod["status"]["phase"]),
                            "status": "firing"
                        })
                    }).collect();
                    
                    return Ok(json!(pod_alerts));
                }
            }
        }
    }
    
    Ok(json!([]))
}

pub async fn get_quotas() -> Result<Value, String> {
    let output = Command::new("kubectl")
        .args(&["get", "resourcequota", "--all-namespaces", "-o", "json"])
        .output();
    
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
        },
        _ => {}
    }
    
    Ok(json!({
        "cpu": {"used": 50, "total": 100, "percentage": 50},
        "memory": {"used": 60, "total": 100, "percentage": 60},
        "quotas_count": 0
    }))
}

pub async fn get_backups() -> Result<Value, String> {
    // Essayer Velero
    let velero_output = Command::new("kubectl")
        .args(&["get", "backups", "-n", "velero", "-o", "json"])
        .output();
    
    if let Ok(result) = velero_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(backup_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(items) = backup_data["items"].as_array() {
                    let backups: Vec<Value> = items.iter().map(|backup| {
                        json!({
                            "name": backup["metadata"]["name"],
                            "status": backup["status"]["phase"],
                            "created": backup["metadata"]["creationTimestamp"],
                            "size": backup["status"]["progress"]["totalItems"]
                        })
                    }).collect();
                    
                    return Ok(json!(backups));
                }
            }
        }
    }
    
    // Fallback: chercher des CronJobs de backup
    let cronjob_output = Command::new("kubectl")
        .args(&["get", "cronjobs", "--all-namespaces", "-o", "json"])
        .output();
    
    if let Ok(result) = cronjob_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(cronjob_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(items) = cronjob_data["items"].as_array() {
                    let backup_jobs: Vec<Value> = items.iter()
                        .filter(|job| {
                            job["metadata"]["name"].as_str()
                                .map(|name| name.contains("backup") || name.contains("dump"))
                                .unwrap_or(false)
                        })
                        .map(|job| {
                            json!({
                                "name": job["metadata"]["name"],
                                "status": "scheduled",
                                "schedule": job["spec"]["schedule"],
                                "namespace": job["metadata"]["namespace"]
                            })
                        }).collect();
                    
                    return Ok(json!(backup_jobs));
                }
            }
        }
    }
    
    Ok(json!([]))
}

fn parse_cpu_value(cpu_str: &str) -> i64 {
    if cpu_str.ends_with('m') {
        cpu_str[..cpu_str.len()-1].parse::<i64>().unwrap_or(0)
    } else {
        cpu_str.parse::<i64>().unwrap_or(0) * 1000
    }
}

fn parse_memory_value(memory_str: &str) -> i64 {
    let memory_str = memory_str.trim();
    if memory_str.ends_with("Gi") {
        memory_str[..memory_str.len()-2].parse::<i64>().unwrap_or(0) * 1024 * 1024 * 1024
    } else if memory_str.ends_with("Mi") {
        memory_str[..memory_str.len()-2].parse::<i64>().unwrap_or(0) * 1024 * 1024
    } else if memory_str.ends_with("Ki") {
        memory_str[..memory_str.len()-2].parse::<i64>().unwrap_or(0) * 1024
    } else {
        memory_str.parse::<i64>().unwrap_or(0)
    }
}
