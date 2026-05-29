use k8s_openapi::api::core::v1::{Event, Node, PersistentVolumeClaim, Pod, Service};
use k8s_openapi::api::networking::v1::Ingress;
use k8s_openapi::api::storage::v1::VolumeAttachment;
use kube::{api::ListParams, Api, Client};
use serde_json::{json, Value};
use tracing::error;

#[tracing::instrument(name = "k8s_get_pods", skip(cache))]
pub async fn get_pods_status(
    cache: &crate::AdvancedCache<String>,
    force_refresh: bool,
) -> Result<Value, String> {
    const CACHE_KEY: &str = "kusanagi_pods_status";

    if force_refresh {
        cache.delete(CACHE_KEY).await;
    } else if let Some(cached) = cache.get(CACHE_KEY).await {
        if let Ok(value) = serde_json::from_str::<Value>(&cached) {
            return Ok(value);
        }
    }

    metrics::counter!("kubernetes_requests_total", "operation" => "get_pods").increment(1);
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

    let result = json!({
        "total_pods": total,
        "running_pods": running,
        "pending_pods": pending,
        "error_pods": pods_in_error.len(),
        "pods_in_error": pods_in_error,
        "pending_pods_list": pending_pods_list
    });

    // Cache results (30s)
    if let Ok(json_str) = serde_json::to_string(&result) {
        cache
            .set(
                CACHE_KEY.to_string(),
                json_str,
                Some(std::time::Duration::from_secs(30)),
            )
            .await;
    }

    Ok(result)
}

// Imports are at top

#[tracing::instrument(name = "k8s_get_nodes", skip(client, cache))]
pub async fn get_nodes_status(
    client: &reqwest::Client,
    cache: &crate::AdvancedCache<String>,
    force_refresh: bool,
) -> Result<Value, String> {
    const CACHE_KEY: &str = "kusanagi_nodes_status";

    if force_refresh {
        cache.delete(CACHE_KEY).await;
    } else if let Some(cached) = cache.get(CACHE_KEY).await {
        if let Ok(value) = serde_json::from_str::<Value>(&cached) {
            return Ok(value);
        }
    }

    metrics::counter!("kubernetes_requests_total", "operation" => "get_nodes").increment(1);
    let kube_client = Client::try_default().await.map_err(|e| e.to_string())?;
    let nodes_api: Api<Node> = Api::all(kube_client.clone());
    let list_params = ListParams::default();

    // Fetch nodes and pods in parallel for efficiency
    let (nodes_list, pods_list) = tokio::join!(nodes_api.list(&list_params), async {
        let pods_api: Api<Pod> = Api::all(kube_client);
        pods_api.list(&ListParams::default()).await.ok()
    });

    let list = nodes_list.map_err(|e| e.to_string())?;

    // Count pods per node
    let mut pod_count_by_node: std::collections::HashMap<String, i32> =
        std::collections::HashMap::new();
    if let Some(pods) = pods_list {
        for pod in pods {
            if let Some(node_name) = pod.spec.as_ref().and_then(|s| s.node_name.as_ref()) {
                *pod_count_by_node.entry(node_name.clone()).or_insert(0) += 1;
            }
        }
    }

    let mut ready = 0;
    let mut not_ready = 0;
    let mut total = 0;
    let mut total_cpu = 0.0;
    let mut total_memory_gb = 0.0;
    let mut nodes_data = Vec::new();

    // Fetch real metrics from Prometheus
    let metrics_map = fetch_node_metrics(client).await.unwrap_or_default();

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

        let memory_capacity_kb = node
            .status
            .as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("memory"))
            .map(|q| parse_k8s_quantity(&q.0) as f64 / 1024.0)
            .unwrap_or(0.0);

        let memory_capacity_gb = memory_capacity_kb / 1024.0 / 1024.0;

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
                let bytes = parse_k8s_quantity(&q.0);
                format_bytes(bytes)
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
        total_memory_gb += memory_capacity_gb;

        // Collect node IPs for better Prometheus matching
        let mut node_ips = Vec::new();
        if let Some(status) = &node.status {
            if let Some(addresses) = &status.addresses {
                for addr in addresses {
                    if addr.type_ == "InternalIP" || addr.type_ == "ExternalIP" {
                        node_ips.push(addr.address.clone());
                    }
                }
            }
        }

        // Metrics logic - try to find metrics with flexible name matching
        let (cpu_usage_core, memory_usage_bytes, disk_usage_percent) =
            find_node_metrics(name, &node_ips, &metrics_map);

        let cpu_usage_percent = if cpu_capacity > 0.0 {
            (cpu_usage_core / cpu_capacity) * 100.0
        } else {
            0.0
        };

        // memory_capacity_kb is in KB, memory_usage_bytes is in Bytes
        // Convert capacity to bytes for percentage calculation
        let memory_capacity_bytes = memory_capacity_kb * 1024.0;
        let memory_usage_percent = if memory_capacity_bytes > 0.0 {
            (memory_usage_bytes / memory_capacity_bytes) * 100.0
        } else {
            0.0
        };

        tracing::debug!(
            "Node {}: CPU={:.1}% ({:.2}/{:.0} cores), MEM={:.1}% ({:.0}/{:.0} bytes)",
            name,
            cpu_usage_percent,
            cpu_usage_core,
            cpu_capacity,
            memory_usage_percent,
            memory_usage_bytes,
            memory_capacity_bytes
        );

        // Get actual pod count for this node
        let actual_pod_count = pod_count_by_node.get(name).copied().unwrap_or(0);

        nodes_data.push(json!({
            "name": name,
            "status": if is_ready { "Ready" } else { "NotReady" },
            "architecture": architecture,
            "os": os,
            "kernel_version": kernel,
            "kubelet_version": kubelet,
            "cpu_capacity": format!("{} cores", cpu_capacity),
            "cpu_usage_percent": cpu_usage_percent,
            "memory_allocatable": memory_allocatable,
            "memory_usage_percent": memory_usage_percent,
            "disk_usage_percent": disk_usage_percent,
            "pod_capacity": pod_capacity,
            "pod_count": actual_pod_count,
            "age": age,
            "conditions": conditions_map
        }));
    }

    let result = json!({
        "total_nodes": total,
        "ready_nodes": ready,
        "not_ready_nodes": not_ready,
        "total_cpu": format!("{} cores", total_cpu),
        "total_memory": format!("{:.1} GB", total_memory_gb),
        "nodes": nodes_data
    });

    // Cache results (30s)
    if let Ok(json_str) = serde_json::to_string(&result) {
        cache
            .set(
                CACHE_KEY.to_string(),
                json_str,
                Some(std::time::Duration::from_secs(30)),
            )
            .await;
    }

    Ok(result)
}

#[tracing::instrument(name = "k8s_cluster_overview", skip(client, kube_client, cache))]
pub async fn get_cluster_overview(
    client: &reqwest::Client,
    kube_client: &Option<std::sync::Arc<kube::Client>>,
    cache: &crate::AdvancedCache<String>,
    force_refresh: bool,
) -> Result<Value, String> {
    metrics::counter!("kubernetes_requests_total", "operation" => "cluster_overview").increment(1);
    let pods = get_pods_status(cache, force_refresh)
        .await
        .unwrap_or_else(|_| {
            json!({
                "total_pods": 0,
                "running_pods": 0,
                "pending_pods": 0,
                "error_pods": 0
            })
        });

    let nodes = get_nodes_status(client, cache, force_refresh)
        .await
        .unwrap_or_else(|_| {
            json!({
                "total_nodes": 0,
                "ready_nodes": 0,
                "not_ready_nodes": 0
            })
        });

    let services_count = match get_services(kube_client, cache).await {
        Ok(json) => json.as_array().map(|v| v.len()).unwrap_or(0),
        Err(_) => 0,
    };

    Ok(json!({
        "nodes": nodes["total_nodes"],
        "pods": pods["total_pods"],
        "services": services_count,
        "pods_running": pods["running_pods"],
        "nodes_ready": nodes["ready_nodes"]
    }))
}

// Imports are at top

#[tracing::instrument(name = "k8s_get_services", skip(kube_client, cache))]
pub async fn get_services(
    kube_client: &Option<std::sync::Arc<kube::Client>>,
    cache: &crate::AdvancedCache<String>,
) -> Result<Value, String> {
    const CACHE_KEY: &str = "kusanagi_services";

    if let Some(cached) = cache.get(CACHE_KEY).await {
        if let Ok(value) = serde_json::from_str::<Value>(&cached) {
            return Ok(value);
        }
    }

    let client = if let Some(kc) = kube_client {
        kc.as_ref().clone()
    } else {
        Client::try_default().await.map_err(|e| e.to_string())?
    };

    let services: Api<Service> = Api::all(client);
    let list = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        services.list(&ListParams::default()),
    )
    .await
    .map_err(|_| "Timeout fetching services".to_string())?
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

    let result = json!(services_json);

    // Cache the result
    cache
        .set(
            CACHE_KEY.to_string(),
            result.to_string(),
            Some(std::time::Duration::from_secs(300)),
        )
        .await;

    Ok(result)
}

// Imports are at top

#[tracing::instrument(name = "k8s_get_ingress", skip(kube_client, cache))]
pub async fn get_ingress(
    kube_client: &Option<std::sync::Arc<kube::Client>>,
    cache: &crate::AdvancedCache<String>,
) -> Result<Value, String> {
    const CACHE_KEY: &str = "kusanagi_ingress";

    if let Some(cached) = cache.get(CACHE_KEY).await {
        if let Ok(value) = serde_json::from_str::<Value>(&cached) {
            return Ok(value);
        }
    }

    let client = if let Some(kc) = kube_client {
        kc.as_ref().clone()
    } else {
        Client::try_default().await.map_err(|e| e.to_string())?
    };

    let ingresses: Api<Ingress> = Api::all(client);
    let list = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        ingresses.list(&ListParams::default()),
    )
    .await
    .map_err(|_| "Timeout fetching ingresses".to_string())?
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

    let result = json!(ingresses_json);

    // Cache the result
    cache
        .set(
            CACHE_KEY.to_string(),
            result.to_string(),
            Some(std::time::Duration::from_secs(300)),
        )
        .await;

    Ok(result)
}

// Imports are at top

pub async fn get_storage(client: &reqwest::Client) -> Result<Value, String> {
    let kube_client = Client::try_default().await.map_err(|e| e.to_string())?;

    let pvcs: Api<PersistentVolumeClaim> = Api::all(kube_client);
    let pvc_list = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        pvcs.list(&ListParams::default()),
    )
    .await
    .map_err(|_| "Timeout fetching PVCs".to_string())?
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
            let (used_bytes, prom_capacity) = usage_map.get(&key).cloned().unwrap_or((0, 0));

            // Use Prometheus capacity if available (more accurate for NFS/CSI), fallback to PVC status
            let effective_capacity = if prom_capacity > 0 {
                prom_capacity
            } else {
                capacity_bytes
            };

            let usage_percent = if effective_capacity > 0 {
                (used_bytes as f64 / effective_capacity as f64) * 100.0
            } else {
                0.0
            };

            json!({
                "name": name,
                "namespace": namespace,
                "status": status,
                "storage_class": storage_class,
                "capacity": if prom_capacity > 0 { format_bytes(prom_capacity) } else { capacity_str },
                "capacity_bytes": effective_capacity,
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

pub async fn get_storage_analysis(client: &reqwest::Client) -> Result<Value, String> {
    let kube_client = Client::try_default().await.map_err(|e| e.to_string())?;

    // 1. Fetch PVCs
    let pvcs_api: Api<PersistentVolumeClaim> = Api::all(kube_client.clone());
    let pvc_list = pvcs_api.list(&ListParams::default()).await.map_err(|e| {
        error!("❌ Storage Analysis: Failed to list PVCs: {}", e);
        e.to_string()
    })?;

    // 2. Fetch VolumeAttachments
    let va_api: Api<VolumeAttachment> = Api::all(kube_client);
    let va_list = va_api.list(&ListParams::default()).await.map_err(|e| {
        error!(
            "❌ Storage Analysis: Failed to list VolumeAttachments: {}",
            e
        );
        e.to_string()
    })?;

    // 3. Fetch Proxmox volumes
    let proxmox_volumes = crate::domain::services::proxmox_service::get_all_proxmox_volumes(client)
        .await
        .unwrap_or(json!([]));

    let mut attached_pvcs = std::collections::HashSet::new();

    for va in &va_list.items {
        let spec = &va.spec;
        let source = &spec.source;
        if let Some(pvc_name) = &source.persistent_volume_name {
            attached_pvcs.insert(pvc_name.clone());
        }
    }

    let mut unattached_pvcs = Vec::new();
    let mut all_k8s_pv_names = std::collections::HashSet::new();

    for pvc in &pvc_list.items {
        let name = pvc.metadata.name.clone().unwrap_or_default();
        let namespace = pvc.metadata.namespace.clone().unwrap_or_default();
        let status = pvc
            .status
            .as_ref()
            .and_then(|s| s.phase.clone())
            .unwrap_or("Unknown".to_string());

        let volume_name = pvc
            .spec
            .as_ref()
            .and_then(|s| s.volume_name.clone())
            .unwrap_or(name.clone());
        all_k8s_pv_names.insert(volume_name.clone());

        if !attached_pvcs.contains(&volume_name) && status == "Bound" {
            unattached_pvcs.push(json!({
                "name": name,
                "volume_name": volume_name,
                "namespace": namespace,
                "status": status,
                "reason": "No VolumeAttachment found"
            }));
        }
    }

    let mut orphaned_proxmox_volumes = Vec::new();
    if let Some(volumes) = proxmox_volumes.as_array() {
        for vol in volumes {
            if let Some(volid) = vol["volid"].as_str() {
                // E.g. "bpool:vm-9999-pvc-a76a..."
                let mut found_k8s_match = false;

                // Usually the 'pvc-xxxxx' part is the PV name in K8s
                if let Some(pos) = volid.find("pvc-") {
                    let pvc_id = format!(
                        "pvc-{}",
                        &volid[pos + 4..].trim_end_matches(".raw").trim_matches('\'')
                    );
                    if all_k8s_pv_names.contains(&pvc_id) {
                        found_k8s_match = true;
                    }
                }

                if volid.contains("pvc-") && !found_k8s_match {
                    orphaned_proxmox_volumes.push(vol.clone());
                }
            }
        }
    }

    Ok(json!({
        "unattached_pvcs": unattached_pvcs,
        "orphaned_proxmox_volumes": orphaned_proxmox_volumes,
        "proxmox_volumes": proxmox_volumes
    }))
}

async fn fetch_storage_usage(
    client: &reqwest::Client,
) -> Result<std::collections::HashMap<String, (u64, u64)>, String> {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let query = "{__name__=~'kubelet_volume_stats_used_bytes|kubelet_volume_stats_capacity_bytes'}";
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
                    let metric_name = metric
                        .get("__name__")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let bytes = value
                        .get(1)
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0) as u64;

                    let entry = usage_map
                        .entry(format!("{}/{}", namespace, pvc_name))
                        .or_insert((0, 0));

                    if metric_name == "kubelet_volume_stats_used_bytes" {
                        entry.0 = bytes;
                    } else if metric_name == "kubelet_volume_stats_capacity_bytes" {
                        entry.1 = bytes;
                    }
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

    // Split numeric part and suffix
    let split_pos = q.find(|c: char| !c.is_numeric() && c != '.');
    let (num_str, suffix) = match split_pos {
        Some(pos) => q.split_at(pos),
        None => (q, ""),
    };

    let val = num_str.parse::<f64>().unwrap_or(0.0);
    let suffix = suffix.trim();

    let multiplier: f64 = match suffix {
        // Binary SI
        "Ki" | "ki" => 1024.0,
        "Mi" | "mi" => 1024.0 * 1024.0,
        "Gi" | "gi" => 1024.0 * 1024.0 * 1024.0,
        "Ti" | "ti" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "Pi" | "pi" => 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "Ei" | "ei" => 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0,
        // Decimal SI
        "k" | "K" => 1000.0,
        "M" => 1000.0 * 1000.0,
        "g" | "G" => 1000.0 * 1000.0 * 1000.0,
        "t" | "T" => 1000.0 * 1000.0 * 1000.0 * 1000.0,
        "p" | "P" => 1000.0 * 1000.0 * 1000.0 * 1000.0 * 1000.0,
        "e" | "E" => 1000.0 * 1000.0 * 1000.0 * 1000.0 * 1000.0 * 1000.0,
        "m" => 0.001, // milli
        "" => 1.0,
        _ => 1.0,
    };

    (val * multiplier) as u64
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut b = bytes as f64;
    let mut i = 0;
    while b >= 1024.0 && i < units.len() - 1 {
        b /= 1024.0;
        i += 1;
    }
    if b < 10.0 && i > 0 {
        format!("{:.2} {}", b, units[i])
    } else {
        format!("{:.1} {}", b, units[i])
    }
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

pub async fn force_delete_pod(
    client: &Option<std::sync::Arc<kube::Client>>,
    namespace: &str,
    name: &str,
) -> Result<Value, String> {
    let client = if let Some(kc) = client {
        kc.as_ref().clone()
    } else {
        Client::try_default().await.map_err(|e| e.to_string())?
    };
    let pods: Api<Pod> = Api::namespaced(client, namespace);

    // Force delete = grace period 0
    let dp = kube::api::DeleteParams {
        grace_period_seconds: Some(0),
        dry_run: false,
        preconditions: None,
        propagation_policy: None,
    };

    match pods.delete(name, &dp).await {
        Ok(_) => Ok(json!({
            "success": true,
            "message": format!("Pod {} deleted successfully (force)", name)
        })),
        Err(e) => Err(format!("Failed to delete pod {}: {}", name, e)),
    }
}

pub async fn get_pod_logs(namespace: &str, name: &str) -> Result<String, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let pods: Api<Pod> = Api::namespaced(client, namespace);
    let log_params = kube::api::LogParams {
        container: None,
        follow: false,
        limit_bytes: None,
        pretty: false,
        previous: false,
        since_seconds: None,
        tail_lines: Some(100),
        timestamps: true,
        since_time: None,
    };

    let logs = pods
        .logs(name, &log_params)
        .await
        .map_err(|e| format!("Failed to fetch logs: {}", e))?;

    Ok(logs)
}

// Helper function to calculate age from k8s timestamp (kube 3.0 uses jiff::Timestamp)
pub fn calculate_age_from_timestamp(
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

// End of file

pub async fn delete_pod(
    client: &Option<std::sync::Arc<kube::Client>>,
    namespace: &str,
    name: &str,
) -> Result<Value, String> {
    let client = if let Some(kc) = client {
        kc.as_ref().clone()
    } else {
        Client::try_default().await.map_err(|e| e.to_string())?
    };
    let pods: Api<Pod> = Api::namespaced(client, namespace);
    let dp = kube::api::DeleteParams::default();

    pods.delete(name, &dp).await.map_err(|e| e.to_string())?;

    Ok(json!({
        "success": true,
        "message": format!("Pod {}/{} deleted", namespace, name)
    }))
}

pub async fn delete_error_pods(
    client: &Option<std::sync::Arc<kube::Client>>,
) -> Result<Value, String> {
    let client = if let Some(kc) = client {
        kc.as_ref().clone()
    } else {
        Client::try_default().await.map_err(|e| e.to_string())?
    };
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

/// Helper to fetch node metrics from Prometheus
/// Returns Map<NodeName, (CpuUsageCores, MemoryUsageBytes, DiskUsagePercent)>
/// Tries multiple metric sources: node_exporter, kubelet, or kubernetes metrics
pub async fn fetch_node_metrics(
    client: &reqwest::Client,
) -> Result<std::collections::HashMap<String, (f64, f64, f64)>, String> {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let url = format!("{}/api/v1/query", prometheus_url);
    let mut metrics_map: std::collections::HashMap<String, (f64, f64, f64)> =
        std::collections::HashMap::new();

    tracing::debug!(
        "Fetching node metrics from Prometheus at {}",
        prometheus_url
    );

    // Try multiple CPU queries in order of preference
    let cpu_queries = [
        // node_exporter: CPU usage in cores
        "sum(rate(node_cpu_seconds_total{mode!=\"idle\"}[5m])) by (node, instance)",
        // Alternative: 100 - idle percentage
        "100 - (avg by (node, instance) (irate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)",
        // kubelet: container CPU usage (removed id=\"/\" filter as it might be missing or vary)
        "sum(rate(container_cpu_usage_seconds_total[5m])) by (node, instance)",
        // kubernetes metric: node CPU usage
        "sum(rate(node_cpu_usage_seconds_total[5m])) by (node, instance)",
    ];

    for query in &cpu_queries {
        match query_prometheus(client, &url, query).await {
            Ok(results) => {
                if !results.is_empty() {
                    tracing::info!("CPU metrics found using query: {}", query);
                    for (node, value) in results {
                        // If the query returns percentage (0-100), convert to cores
                        // If it returns cores (could be > 100 for multi-core), keep as is
                        let cpu_cores = if value > 100.0 {
                            // Likely already in percentage, convert to cores assuming 1 core = 100%
                            value / 100.0
                        } else {
                            value
                        };
                        metrics_map.insert(node, (cpu_cores, 0.0, 0.0));
                    }
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("CPU query failed: {} - {}", query, e);
            }
        }
    }

    // Try multiple memory queries
    let mem_queries = [
        // node_exporter: memory used in bytes
        "sum(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes) by (node, instance)",
        // Alternative calculation
        "sum(node_memory_MemTotal_bytes - node_memory_Buffers_bytes - node_memory_Cached_bytes - node_memory_MemFree_bytes) by (node, instance)",
        // kubelet: container memory usage
        "sum(container_memory_working_set_bytes) by (node, instance)",
        // kubernetes metric
        "sum(node_memory_usage_bytes) by (node, instance)",
    ];

    for query in &mem_queries {
        match query_prometheus(client, &url, query).await {
            Ok(results) => {
                if !results.is_empty() {
                    tracing::info!("Memory metrics found using query: {}", query);
                    for (node, value) in results {
                        if let Some(entry) = metrics_map.get_mut(&node) {
                            entry.1 = value;
                        } else {
                            // Node exists in memory metrics but not CPU
                            metrics_map.insert(node, (0.0, value, 0.0));
                        }
                    }
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Memory query failed: {} - {}", query, e);
            }
        }
    }

    // Fetch Disk queries (Root partition)
    let disk_queries = [
        // node_exporter: Disk usage percentage for /
        "100 - (node_filesystem_avail_bytes{mountpoint=\"/\"} / node_filesystem_size_bytes{mountpoint=\"/\"} * 100)",
    ];

    for query in &disk_queries {
        match query_prometheus(client, &url, query).await {
            Ok(results) => {
                if !results.is_empty() {
                    tracing::info!("Disk metrics found using query: {}", query);
                    for (node, value) in results {
                        if let Some(entry) = metrics_map.get_mut(&node) {
                            entry.2 = value;
                        } else {
                            // Node exists in disk metrics but not CPU/MEM
                            metrics_map.insert(node, (0.0, 0.0, value));
                        }
                    }
                    break;
                }
            }
            Err(e) => {
                tracing::warn!("Disk query failed: {} - {}", query, e);
            }
        }
    }

    tracing::info!("Fetched metrics for {} nodes", metrics_map.len());
    for (node, (cpu, mem, disk)) in &metrics_map {
        tracing::debug!(
            "Node {}: CPU={:.2} cores, MEM={:.2} bytes, DISK={:.1}%",
            node,
            cpu,
            mem,
            disk
        );
    }

    Ok(metrics_map)
}

/// Query Prometheus and return a map of node -> value
async fn query_prometheus(
    client: &reqwest::Client,
    url: &str,
    query: &str,
) -> Result<std::collections::HashMap<String, f64>, String> {
    let response = client
        .get(url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let mut results_map = std::collections::HashMap::new();

    if let Some(results) = body
        .get("data")
        .and_then(|d| d.get("result"))
        .and_then(|r| r.as_array())
    {
        for result in results {
            if let (Some(metric), Some(value)) = (result.get("metric"), result.get("value")) {
                // Try different node labels
                let node_name = metric
                    .get("node")
                    .or_else(|| metric.get("instance"))
                    .or_else(|| metric.get("kubernetes_node"))
                    .and_then(|s| s.as_str());

                if let Some(node) = node_name {
                    // Clean up node name (remove port if present)
                    let clean_node = node.split(':').next().unwrap_or(node).to_string();

                    if let Some(val_str) = value.get(1).and_then(|v| v.as_str()) {
                        if let Ok(val) = val_str.parse::<f64>() {
                            results_map.insert(clean_node, val);
                        }
                    }
                }
            }
        }
    }

    Ok(results_map)
}

/// Find node metrics with flexible name matching
/// Handles cases where Prometheus node name differs from Kubernetes node name
fn find_node_metrics(
    k8s_node_name: &str,
    node_ips: &[String],
    metrics_map: &std::collections::HashMap<String, (f64, f64, f64)>,
) -> (f64, f64, f64) {
    // First try exact match
    if let Some(metrics) = metrics_map.get(k8s_node_name) {
        return *metrics;
    }

    // Try matching without domain suffix (e.g., "node1.cluster.local" -> "node1")
    let k8s_short = k8s_node_name.split('.').next().unwrap_or(k8s_node_name);

    for (metric_node, metrics) in metrics_map {
        // Exact match after cleaning
        let metric_short = metric_node.split('.').next().unwrap_or(metric_node);

        if k8s_short == metric_short {
            tracing::debug!(
                "Matched node {} to metrics for {} (short: {})",
                k8s_node_name,
                metric_node,
                metric_short
            );
            return *metrics;
        }

        // Check if one contains the other
        if metric_node.contains(k8s_short) || k8s_node_name.contains(metric_short) {
            tracing::debug!("Partial match: {} ~ {}", k8s_node_name, metric_node);
            return *metrics;
        }
    }

    // Try IP address matching - check if any of the node's IPs match the metric source
    for ip in node_ips {
        for (metric_node, metrics) in metrics_map {
            // metric_node might be "192.168.1.1:9100" or just "192.168.1.1"
            let metric_ip = metric_node.split(':').next().unwrap_or(metric_node);

            if ip == metric_ip {
                tracing::info!("Matched node {} to metrics via IP {}", k8s_node_name, ip);
                return *metrics;
            }
        }
    }

    // Last resort: check if any metric_node (IP) is contained in any of our node IPs (or vice-versa)
    for (metric_node, metrics) in metrics_map {
        let metric_ip = metric_node.split(':').next().unwrap_or(metric_node);
        for ip in node_ips {
            if ip.contains(metric_ip) || metric_ip.contains(ip) {
                tracing::info!(
                    "Partial IP match: {} ~ {} for node {}",
                    ip,
                    metric_ip,
                    k8s_node_name
                );
                return *metrics;
            }
        }
    }

    tracing::warn!("No metrics found for node {}", k8s_node_name);
    (0.0, 0.0, 0.0)
}

/// Fetch resource usage metrics per namespace from Prometheus
pub async fn get_namespace_metrics(
    client: &reqwest::Client,
    window: Option<String>,
) -> Result<Value, String> {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let url = format!("{}/api/v1/query", prometheus_url);

    let window_val = window.unwrap_or_else(|| "5m".to_string());

    // CPU Query:
    // For short windows like 5m, we use rate(...[5m])
    // For longer windows like 1d or 30d, we use rate over that period to get average usage
    let cpu_query = format!("sum(rate(container_cpu_usage_seconds_total{{container!=\"\"}}[{window_val}])) by (namespace)");

    // Memory Query:
    // For short windows, current value
    // For longer windows, average over time
    let mem_query = if window_val == "5m" {
        "sum(container_memory_working_set_bytes{container!=\"\"}) by (namespace)".to_string()
    } else {
        format!("sum(avg_over_time(container_memory_working_set_bytes{{container!=\"\"}}[{window_val}])) by (namespace)")
    };

    let mut namespace_data: std::collections::HashMap<String, (f64, f64)> =
        std::collections::HashMap::new();

    // Fetch CPU
    if let Ok(results) = query_prometheus_by_namespace(client, &url, &cpu_query).await {
        for (ns, val) in results {
            namespace_data.entry(ns).or_insert((0.0, 0.0)).0 = val;
        }
    }

    // Fetch Memory
    if let Ok(results) = query_prometheus_by_namespace(client, &url, &mem_query).await {
        for (ns, val) in results {
            namespace_data.entry(ns).or_insert((0.0, 0.0)).1 = val;
        }
    }

    let mut response_list = Vec::new();
    for (name, (cpu, mem)) in namespace_data {
        response_list.push(json!({
            "namespace": name,
            "cpu_usage": cpu,
            "memory_usage_bytes": mem
        }));
    }

    Ok(json!(response_list))
}

/// Helper to query Prometheus and return a map of namespace -> value
async fn query_prometheus_by_namespace(
    client: &reqwest::Client,
    url: &str,
    query: &str,
) -> Result<std::collections::HashMap<String, f64>, String> {
    let response = client
        .get(url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let body: Value = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let mut results_map = std::collections::HashMap::new();

    if let Some(results) = body
        .get("data")
        .and_then(|d| d.get("result"))
        .and_then(|r| r.as_array())
    {
        for result in results {
            if let (Some(metric), Some(value)) = (result.get("metric"), result.get("value")) {
                if let Some(ns) = metric.get("namespace").and_then(|s| s.as_str()) {
                    if let Some(val_str) = value.get(1).and_then(|v| v.as_str()) {
                        if let Ok(val) = val_str.parse::<f64>() {
                            results_map.insert(ns.to_string(), val);
                        }
                    }
                }
            }
        }
    }

    Ok(results_map)
}

/// Fetch cluster-wide resource metrics (Usage, Requests, Limits, Capacity)
pub async fn get_cluster_resource_metrics(client: &reqwest::Client) -> Result<Value, String> {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let url = format!("{}/api/v1/query", prometheus_url);

    // 1. Fetch Capacity and Allocatable from K8s API
    let kube_client = Client::try_default().await.map_err(|e| e.to_string())?;
    let nodes_api: Api<Node> = Api::all(kube_client);
    let nodes = nodes_api
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    let mut cpu_capacity = 0.0;
    let mut cpu_allocatable = 0.0;
    let mut mem_capacity_bytes = 0.0;
    let mut mem_allocatable_bytes = 0.0;

    for node in nodes {
        if let Some(status) = &node.status {
            if let Some(cap) = &status.capacity {
                if let Some(cpu) = cap.get("cpu") {
                    cpu_capacity += cpu.0.parse::<f64>().unwrap_or(0.0);
                }
                if let Some(mem) = cap.get("memory") {
                    mem_capacity_bytes += parse_k8s_quantity(&mem.0) as f64;
                }
            }
            if let Some(alloc) = &status.allocatable {
                if let Some(cpu) = alloc.get("cpu") {
                    cpu_allocatable += cpu.0.parse::<f64>().unwrap_or(0.0);
                }
                if let Some(mem) = alloc.get("memory") {
                    mem_allocatable_bytes += parse_k8s_quantity(&mem.0) as f64;
                }
            }
        }
    }

    // 2. Fetch Usage, Requests, Limits from Prometheus
    let queries = [
        (
            "cpu_usage",
            "sum(rate(node_cpu_seconds_total{mode!=\"idle\"}[5m]))",
        ),
        (
            "cpu_requests",
            "sum(kube_pod_container_resource_requests{resource=\"cpu\"})",
        ),
        (
            "cpu_limits",
            "sum(kube_pod_container_resource_limits{resource=\"cpu\"})",
        ),
        (
            "mem_usage",
            "sum(node_memory_MemTotal_bytes - node_memory_MemAvailable_bytes)",
        ),
        (
            "mem_requests",
            "sum(kube_pod_container_resource_requests{resource=\"memory\"})",
        ),
        (
            "mem_limits",
            "sum(kube_pod_container_resource_limits{resource=\"memory\"})",
        ),
    ];

    let mut prometheus_metrics = std::collections::HashMap::new();
    for (name, query) in &queries {
        let val = query_prometheus_scalar(client, &url, query)
            .await
            .unwrap_or(0.0);
        prometheus_metrics.insert(name.to_string(), val);
    }

    Ok(json!({
        "cpu": {
            "usage": prometheus_metrics.get("cpu_usage").unwrap_or(&0.0),
            "requests": prometheus_metrics.get("cpu_requests").unwrap_or(&0.0),
            "limits": prometheus_metrics.get("cpu_limits").unwrap_or(&0.0),
            "allocatable": cpu_allocatable,
            "capacity": cpu_capacity
        },
        "memory": {
            "usage": prometheus_metrics.get("mem_usage").unwrap_or(&0.0),
            "requests": prometheus_metrics.get("mem_requests").unwrap_or(&0.0),
            "limits": prometheus_metrics.get("mem_limits").unwrap_or(&0.0),
            "allocatable": mem_allocatable_bytes,
            "capacity": mem_capacity_bytes
        }
    }))
}

/// Helper to query Prometheus and extract a scalar value
async fn query_prometheus_scalar(client: &reqwest::Client, url: &str, query: &str) -> Option<f64> {
    let response = client
        .get(url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let body: serde_json::Value = response.json().await.ok()?;

    body.get("data")?
        .get("result")?
        .as_array()?
        .first()?
        .get("value")?
        .as_array()?
        .get(1)?
        .as_str()?
        .parse::<f64>()
        .ok()
}

/// Fetch failed jobs from Prometheus
pub async fn get_failed_jobs(client: &reqwest::Client) -> Result<Value, String> {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let url = format!("{}/api/v1/query", prometheus_url);
    let query = "kube_job_failed{condition=\"true\"} > 0";

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
    let mut failed_jobs = Vec::new();

    if let Some(results) = body
        .get("data")
        .and_then(|d| d.get("result"))
        .and_then(|r| r.as_array())
    {
        for result in results {
            if let Some(metric) = result.get("metric") {
                let job_name = metric
                    .get("job_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                // Filter out scan-vulnerabilityreport jobs as requested by user
                if job_name.starts_with("scan-vulnerabilityreport-") {
                    continue;
                }

                failed_jobs.push(json!({
                    "job_name": job_name,
                    "namespace": metric.get("namespace").and_then(|v| v.as_str()).unwrap_or("unknown")
                }));
            }
        }
    }

    Ok(json!({
        "total": failed_jobs.len(),
        "failed_jobs": failed_jobs
    }))
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
        assert_eq!(parse_k8s_quantity("1.5Gi"), 1610612736);
        assert_eq!(parse_k8s_quantity("1.5G"), 1500000000);
        assert_eq!(parse_k8s_quantity("100M"), 100000000);
        assert_eq!(parse_k8s_quantity("100Mi"), 104857600);
        assert_eq!(parse_k8s_quantity(""), 0);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1024), "1.00 KiB");
        assert_eq!(format_bytes(1048576), "1.00 MiB");
        assert_eq!(format_bytes(1073741824), "1.00 GiB");
    }

    #[test]
    fn test_find_node_metrics() {
        let mut metrics_map = std::collections::HashMap::new();
        metrics_map.insert("node1".to_string(), (1.0, 1024.0, 0.0));
        metrics_map.insert("192.168.1.10:9100".to_string(), (2.0, 2048.0, 0.0));
        metrics_map.insert("node3.cluster.local".to_string(), (3.0, 4096.0, 0.0));

        // Exact match
        assert_eq!(
            find_node_metrics("node1", &[], &metrics_map),
            (1.0, 1024.0, 0.0)
        );

        // IP match (via node_ips)
        assert_eq!(
            find_node_metrics("node-x", &["192.168.1.10".to_string()], &metrics_map),
            (2.0, 2048.0, 0.0)
        );

        // Short name match
        assert_eq!(
            find_node_metrics("node3", &[], &metrics_map),
            (3.0, 4096.0, 0.0)
        );

        // No match
        assert_eq!(
            find_node_metrics("unknown", &[], &metrics_map),
            (0.0, 0.0, 0.0)
        );
    }
}
