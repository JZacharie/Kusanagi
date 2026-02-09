use k8s_openapi::api::core::v1::{Event, Node, PersistentVolumeClaim, Pod, Service};
use k8s_openapi::api::networking::v1::Ingress;
use kube::{api::ListParams, Api, Client};
use serde_json::{json, Value};

pub async fn get_pods_status() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let pods: Api<Pod> = Api::all(client);

    // Use timeout for the list operation
    let list = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        pods.list(&ListParams::default()),
    )
    .await
    .map_err(|_| "Timeout fetching pods".to_string())?
    .map_err(|e| e.to_string())?;

    let mut running = 0;
    let mut pending = 0;
    let mut total = 0;
    let mut pods_in_error = Vec::new();
    let mut pending_pods_list = Vec::new();

    for pod in list {
        total += 1;
        let phase = pod
            .status
            .as_ref()
            .and_then(|s| s.phase.as_deref())
            .unwrap_or("Unknown");

        let mut is_error = false;
        let mut reason = String::new();
        let mut restart_count = 0;

        match phase {
            "Running" | "Succeeded" => running += 1,
            "Pending" => {
                pending += 1;
                let age = pod
                    .metadata
                    .creation_timestamp
                    .as_ref()
                    .map(calculate_age_from_timestamp)
                    .unwrap_or_default();

                pending_pods_list.push(json!({
                    "name": pod.metadata.name.as_deref().unwrap_or(""),
                    "namespace": pod.metadata.namespace.as_deref().unwrap_or(""),
                    "status": phase,
                    "reason": "Pending",
                    "restart_count": 0, // Pending pods usually have 0 restarts unless they are crashing loop
                    "age": age,
                    "node": pod.spec.as_ref().and_then(|s| s.node_name.as_deref()).unwrap_or(""),
                    "cpu_usage": 0,
                    "memory_usage": 0,
                    "cpu_limit": 0,
                    "memory_limit": 0
                }));
            }
            _ => {
                is_error = true;
                reason = phase.to_string();
            }
        }

        // Check container statuses for errors
        if let Some(status) = &pod.status {
            if let Some(container_statuses) = &status.container_statuses {
                for cs in container_statuses {
                    restart_count += cs.restart_count;

                    if let Some(state) = &cs.state {
                        if let Some(waiting) = &state.waiting {
                            if let Some(r) = &waiting.reason {
                                if r.contains("BackOff")
                                    || r.contains("Error")
                                    || r.contains("Pull")
                                {
                                    is_error = true;
                                    reason = r.clone();
                                }
                            }
                        }
                        if let Some(terminated) = &state.terminated {
                            if terminated.exit_code != 0 {
                                is_error = true;
                                reason = terminated.reason.clone().unwrap_or("Error".to_string());
                            }
                        }
                    }
                }
            }
        }

        if is_error {
            let age = pod
                .metadata
                .creation_timestamp
                .as_ref()
                .map(calculate_age_from_timestamp)
                .unwrap_or_default();

            pods_in_error.push(json!({
                "name": pod.metadata.name.as_deref().unwrap_or(""),
                "namespace": pod.metadata.namespace.as_deref().unwrap_or(""),
                "status": phase,
                "reason": if reason.is_empty() { phase } else { &reason },
                "restart_count": restart_count,
                "age": age,
                "node": pod.spec.as_ref().and_then(|s| s.node_name.as_deref()).unwrap_or(""),
                "cpu_usage": 0,
                "memory_usage": 0,
                "cpu_limit": 0,
                "memory_limit": 0
            }));
        }
    }

    Ok(json!({
        "total_pods": total,
        "running_pods": running,
        "pending_pods": pending,
        "error_pods": pods_in_error.len(),
        "pods_in_error": pods_in_error,
        "pending_pods_list": pending_pods_list
    }))
}

// Imports are at top

pub async fn get_nodes_status() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let nodes_api: Api<Node> = Api::all(client);
    let list = nodes_api
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    let mut ready = 0;
    let mut not_ready = 0;
    let mut total = 0;
    let mut total_cpu = 0.0;
    let mut total_memory_gb = 0.0;
    let mut nodes_data = Vec::new();

    for node in list {
        total += 1;
        let name = node.metadata.name.as_deref().unwrap_or("");
        let mut is_ready = false;
        let mut conditions_map = std::collections::HashMap::new();

        // Extract status and conditions
        if let Some(status) = &node.status {
            if let Some(conditions) = &status.conditions {
                for cond in conditions {
                    conditions_map.insert(cond.type_.clone(), cond.status.clone());
                    if cond.type_ == "Ready" && cond.status == "True" {
                        is_ready = true;
                    }
                }
            }
        }

        if is_ready {
            ready += 1;
        } else {
            not_ready += 1;
        }

        // Extract node info
        let architecture = node
            .status
            .as_ref()
            .and_then(|s| s.node_info.as_ref())
            .map(|ni| ni.architecture.clone())
            .unwrap_or_default();

        let os = node
            .status
            .as_ref()
            .and_then(|s| s.node_info.as_ref())
            .map(|ni| format!("{} {}", ni.operating_system, ni.os_image))
            .unwrap_or_default();

        let kernel = node
            .status
            .as_ref()
            .and_then(|s| s.node_info.as_ref())
            .map(|ni| ni.kernel_version.clone())
            .unwrap_or_default();

        let kubelet = node
            .status
            .as_ref()
            .and_then(|s| s.node_info.as_ref())
            .map(|ni| ni.kubelet_version.clone())
            .unwrap_or_default();

        // Extract capacity
        let cpu_capacity = node
            .status
            .as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("cpu"))
            .and_then(|q| q.0.parse::<f64>().ok())
            .unwrap_or(0.0);

        let memory_capacity = node
            .status
            .as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("memory"))
            .map(|q| {
                let kb = parse_k8s_quantity(&q.0) as f64;
                kb / 1024.0 / 1024.0 // Convert KB to GB
            })
            .unwrap_or(0.0);

        let pod_capacity = node
            .status
            .as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("pods"))
            .and_then(|q| q.0.parse::<i32>().ok())
            .unwrap_or(0);

        // Extract allocatable
        let memory_allocatable = node
            .status
            .as_ref()
            .and_then(|s| s.allocatable.as_ref())
            .and_then(|a| a.get("memory"))
            .map(|q| {
                let kb = parse_k8s_quantity(&q.0);
                format_bytes(kb * 1024)
            })
            .unwrap_or_default();

        // Calculate age
        let age = node
            .metadata
            .creation_timestamp
            .as_ref()
            .map(calculate_age_from_timestamp)
            .unwrap_or_default();

        total_cpu += cpu_capacity;
        total_memory_gb += memory_capacity / 1024.0;

        nodes_data.push(json!({
            "name": name,
            "status": if is_ready { "Ready" } else { "NotReady" },
            "architecture": architecture,
            "os": os,
            "kernel_version": kernel,
            "kubelet_version": kubelet,
            "cpu_capacity": format!("{} cores", cpu_capacity),
            "cpu_usage_percent": 0.0,
            "memory_allocatable": memory_allocatable,
            "memory_usage_percent": 0.0,
            "pod_capacity": pod_capacity,
            "pod_count": 0,
            "age": age,
            "conditions": conditions_map
        }));
    }

    Ok(json!({
        "total_nodes": total,
        "ready_nodes": ready,
        "not_ready_nodes": not_ready,
        "total_cpu": format!("{} cores", total_cpu),
        "total_memory": format!("{:.1} GB", total_memory_gb),
        "nodes": nodes_data
    }))
}

pub async fn get_cluster_overview() -> Result<Value, String> {
    let pods = get_pods_status().await.unwrap_or_else(|_| {
        json!({
            "total": 0,
            "running": 0,
            "pending": 0,
            "failed": 0
        })
    });

    let nodes = get_nodes_status().await.unwrap_or_else(|_| {
        json!({
            "total": 0,
            "ready": 0,
            "not_ready": 0
        })
    });

    let services_count = match get_services().await {
        Ok(json) => json.as_array().map(|v| v.len()).unwrap_or(0),
        Err(_) => 0,
    };

    Ok(json!({
        "nodes": nodes["total"],
        "pods": pods["total"],
        "services": services_count,
        "pods_running": pods["running"],
        "nodes_ready": nodes["ready"]
    }))
}

// Imports are at top

pub async fn get_services() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let services: Api<Service> = Api::all(client);
    let list = services
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    let services_json: Vec<Value> = list.iter().map(|svc| {
        let age = if let Some(ts) = &svc.metadata.creation_timestamp {
            calculate_age_from_timestamp(ts)
        } else {
            "0s".to_string()
        };

        // Ports
        let ports = svc.spec.as_ref()
            .and_then(|s| s.ports.as_ref())
            .map(|p| p.iter().map(|port| {
                format!("{}:{}/{}", port.port, port.node_port.unwrap_or(0), port.protocol.clone().unwrap_or_default())
            }).collect::<Vec<String>>().join(", "))
            .unwrap_or_default();

        // External IP
        let external_ip = svc.status.as_ref()
            .and_then(|s| s.load_balancer.as_ref())
            .and_then(|lb| lb.ingress.as_ref())
            .and_then(|i| i.first())
            .map(|ing| ing.ip.clone().unwrap_or_else(|| ing.hostname.clone().unwrap_or_default()))
            .unwrap_or_else(|| "<none>".to_string());

        json!({
            "name": svc.metadata.name.clone().unwrap_or_default(),
            "namespace": svc.metadata.namespace.clone().unwrap_or_default(),
            "type_": svc.spec.as_ref().and_then(|s| s.type_.clone()).unwrap_or_default(),
            "cluster_ip": svc.spec.as_ref().and_then(|s| s.cluster_ip.clone()).unwrap_or_default(),
            "external_ip": external_ip,
            "ports": ports,
            "age": age
        })
    }).collect();

    Ok(json!(services_json))
}

// Imports are at top

pub async fn get_ingress() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let ingresses: Api<Ingress> = Api::all(client);
    let list = ingresses
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    let ingresses_json: Vec<Value> = list
        .iter()
        .map(|ing| {
            let age = if let Some(ts) = &ing.metadata.creation_timestamp {
                calculate_age_from_timestamp(ts)
            } else {
                "0s".to_string()
            };

            let rules = ing
                .spec
                .as_ref()
                .and_then(|spec| spec.rules.as_ref())
                .map(|rules| {
                    rules
                        .iter()
                        .filter_map(|r| r.host.clone())
                        .collect::<Vec<String>>()
                })
                .unwrap_or_default();

            json!({
                "name": ing.metadata.name.clone().unwrap_or_default(),
                "namespace": ing.metadata.namespace.clone().unwrap_or_default(),
                "rules": rules,
                "age": age
            })
        })
        .collect();

    Ok(json!(ingresses_json))
}

// Imports are at top

pub async fn get_storage(client: &reqwest::Client) -> Result<Value, String> {
    let kube_client = Client::try_default().await.map_err(|e| e.to_string())?;

    let pvcs: Api<PersistentVolumeClaim> = Api::all(kube_client);
    let pvc_list = pvcs
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    // Fetch usage metrics from Prometheus
    let usage_map = fetch_storage_usage(client).await.unwrap_or_default();

    let pvcs_json: Vec<Value> = pvc_list
        .iter()
        .map(|pvc| {
            let name = pvc.metadata.name.clone().unwrap_or_default();
            let namespace = pvc.metadata.namespace.clone().unwrap_or_default();
            let status = pvc
                .status
                .as_ref()
                .and_then(|s| s.phase.clone())
                .unwrap_or("Unknown".to_string());
            let storage_class = pvc
                .spec
                .as_ref()
                .and_then(|s| s.storage_class_name.clone())
                .unwrap_or_default();

            let capacity_str = pvc
                .status
                .as_ref()
                .and_then(|s| s.capacity.as_ref())
                .and_then(|c| c.get("storage"))
                .map(|q| q.0.clone())
                .unwrap_or("0".to_string());

            let capacity_bytes = parse_k8s_quantity(&capacity_str);

            // Get usage from map
            let key = format!("{}/{}", namespace, name);
            let used_bytes = usage_map.get(&key).cloned().unwrap_or(0);

            let usage_percent = if capacity_bytes > 0 {
                (used_bytes as f64 / capacity_bytes as f64) * 100.0
            } else {
                0.0
            };

            json!({
                "name": name,
                "namespace": namespace,
                "status": status,
                "storage_class": storage_class,
                "capacity": capacity_str,
                "capacity_bytes": capacity_bytes,
                "used_bytes": used_bytes,
                "usage_percent": usage_percent
            })
        })
        .collect();

    let total_capacity: u64 = pvcs_json
        .iter()
        .map(|v| v["capacity_bytes"].as_u64().unwrap_or(0))
        .sum();
    let total_formatted = format_bytes(total_capacity);

    Ok(json!({
        "pvc_count": pvcs_json.len(),
        "pvc_total_capacity": total_formatted,
        "pvcs": pvcs_json
    }))
}

async fn fetch_storage_usage(
    client: &reqwest::Client,
) -> Result<std::collections::HashMap<String, u64>, String> {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let query = "kubelet_volume_stats_used_bytes";
    let url = format!("{}/api/v1/query", prometheus_url);

    let response = client
        .get(&url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Prometheus request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Prometheus returned status: {}", response.status()));
    }

    let body: Value = response.json().await.map_err(|e| e.to_string())?;

    let mut usage_map = std::collections::HashMap::new();

    if let Some(results) = body
        .get("data")
        .and_then(|d| d.get("result"))
        .and_then(|r| r.as_array())
    {
        for result in results {
            if let (Some(metric), Some(value)) = (result.get("metric"), result.get("value")) {
                let namespace = metric
                    .get("namespace")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                let pvc_name = metric
                    .get("persistentvolumeclaim")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");

                if !namespace.is_empty() && !pvc_name.is_empty() {
                    let bytes = value
                        .get(1)
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0) as u64;
                    usage_map.insert(format!("{}/{}", namespace, pvc_name), bytes);
                }
            }
        }
    }

    Ok(usage_map)
}

pub fn parse_k8s_quantity(q: &str) -> u64 {
    let q = q.trim();
    if q.is_empty() {
        return 0;
    }

    let digits: String = q.chars().take_while(|c| c.is_ascii_digit()).collect();
    let suffix: String = q.chars().skip_while(|c| c.is_ascii_digit()).collect();

    let value = digits.parse::<u64>().unwrap_or(0);

    match suffix.as_str() {
        "Ki" => value * 1024,
        "Mi" => value * 1024 * 1024,
        "Gi" => value * 1024 * 1024 * 1024,
        "Ti" => value * 1024 * 1024 * 1024 * 1024,
        "Pi" => value * 1024 * 1024 * 1024 * 1024 * 1024,
        "m" => 0, // Millibytes not relevant for storage
        "" => value,
        _ => value, // Unknown suffix, return raw
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    let mut b = bytes as f64;
    let mut i = 0;
    while b >= 1024.0 && i < units.len() - 1 {
        b /= 1024.0;
        i += 1;
    }
    format!("{:.1} {}", b, units[i])
}

// Imports are at top

pub async fn get_events() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let events_api: Api<Event> = Api::all(client);
    let mut events_list = events_api
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?
        .items;

    // Sort by lastTimestamp
    events_list.sort_by(|a, b| a.last_timestamp.cmp(&b.last_timestamp));

    // Take last 20 (latest)
    let events: Vec<Value> = events_list
        .iter()
        .rev()
        .take(20)
        .map(|event| {
            json!({
                "type": event.type_.clone().unwrap_or_default(),
                "reason": event.reason.clone().unwrap_or_default(),
                "message": event.message.clone().unwrap_or_default(),
                "namespace": event.metadata.namespace.clone().unwrap_or_default(),
                "object": event.involved_object.name.clone().unwrap_or_default(),
                "timestamp": event.last_timestamp.clone()
            })
        })
        .collect();

    Ok(json!(events))
}

// Helper function to calculate age from k8s timestamp (kube 3.0 uses jiff::Timestamp)
fn calculate_age_from_timestamp(
    ts: &k8s_openapi::apimachinery::pkg::apis::meta::v1::Time,
) -> String {
    // Convert jiff::Timestamp to seconds since epoch
    let created_secs = ts.0.as_second();
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let diff_secs = now_secs - created_secs;

    if diff_secs < 0 {
        return "0s".to_string();
    }

    let days = diff_secs / 86400;
    let hours = (diff_secs % 86400) / 3600;
    let minutes = (diff_secs % 3600) / 60;
    let seconds = diff_secs % 60;

    if days > 0 {
        format!("{}d", days)
    } else if hours > 0 {
        format!("{}h", hours)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_k8s_quantity() {
        assert_eq!(parse_k8s_quantity("1024"), 1024);
        assert_eq!(parse_k8s_quantity("1Ki"), 1024);
        assert_eq!(parse_k8s_quantity("1Mi"), 1048576);
        assert_eq!(parse_k8s_quantity("1Gi"), 1073741824);
        assert_eq!(parse_k8s_quantity(""), 0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.0 KiB");
        assert_eq!(format_bytes(1048576), "1.0 MiB");
        assert_eq!(format_bytes(1073741824), "1.0 GiB");
    }
}

// End of file

pub async fn delete_pod(namespace: &str, name: &str) -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let pods: Api<Pod> = Api::namespaced(client, namespace);
    let dp = kube::api::DeleteParams::default();

    pods.delete(name, &dp).await.map_err(|e| e.to_string())?;

    Ok(json!({
        "success": true,
        "message": format!("Pod {}/{} deleted", namespace, name)
    }))
}

pub async fn delete_error_pods() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let pods: Api<Pod> = Api::all(client.clone());
    let list = pods
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    let mut deleted_count = 0;
    let mut errors = Vec::new();

    for pod in list {
        let name = pod.metadata.name.clone().unwrap_or_default();
        let namespace = pod.metadata.namespace.clone().unwrap_or_default();
        let phase = pod
            .status
            .as_ref()
            .and_then(|s| s.phase.as_deref())
            .unwrap_or("Unknown");

        let mut should_delete = false;

        if phase == "Failed" || phase == "Unknown" {
            should_delete = true;
        } else if phase == "Pending" {
            if let Some(status) = &pod.status {
                if let Some(container_statuses) = &status.container_statuses {
                    for cs in container_statuses {
                        if let Some(state) = &cs.state {
                            if let Some(waiting) = &state.waiting {
                                if let Some(reason) = &waiting.reason {
                                    if reason == "ImagePullBackOff"
                                        || reason == "ErrImagePull"
                                        || reason == "CrashLoopBackOff"
                                    {
                                        should_delete = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if should_delete {
            let pods_ns: Api<Pod> = Api::namespaced(client.clone(), &namespace);
            let dp = kube::api::DeleteParams::default();
            match pods_ns.delete(&name, &dp).await {
                Ok(_) => deleted_count += 1,
                Err(e) => errors.push(format!("Failed to delete {}/{}: {}", namespace, name, e)),
            }
        }
    }

    Ok(json!({
        "success": true,
        "deleted": deleted_count,
        "errors": errors
    }))
}
