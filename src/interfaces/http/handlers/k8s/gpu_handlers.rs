use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Pod;
use kube::api::{ListParams, Patch, PatchParams};
use kube::{Api, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::domain::services::kubernetes_service::calculate_age_from_timestamp;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct GpuQuery {
    pub refresh: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct ScaleWorkloadRequest {
    pub namespace: String,
    pub deployment: String,
    pub replicas: i32,
}

/// Fetch GPU overview: nodes telemetry, pods running on GPU nodes, and managed workloads
pub async fn get_gpu_status_handler(
    State(state): State<AppState>,
    Query(query): Query<GpuQuery>,
) -> Response {
    const CACHE_KEY: &str = "kusanagi_gpu_status";

    let force_refresh = query.refresh.unwrap_or(false);

    if !force_refresh {
        if let Some(cached) = state.general_cache.get(CACHE_KEY).await {
            if let Ok(value) = serde_json::from_str::<Value>(&cached) {
                return api_success(value);
            }
        }
    }

    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let client = &state.http_client;

    // 1. Gather GPU metrics per node from Prometheus DCGM
    let gpu_metrics_by_node = fetch_dcgm_metrics_by_node(client, &prometheus_url).await;

    // 2. Query Kubernetes API for GPU nodes and Pods
    let kube_client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to connect to Kubernetes: {}", e),
            )
        }
    };

    let pods_api: Api<Pod> = Api::all(kube_client.clone());
    let pods_list = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        pods_api.list(&ListParams::default()),
    )
    .await
    {
        Ok(Ok(list)) => list.items,
        _ => Vec::new(),
    };

    // Filter pods running on vm168, vm169, or nodes with GPU metrics
    let mut gpu_pods = Vec::new();
    let mut node_pod_counts: HashMap<String, usize> = HashMap::new();

    for pod in pods_list {
        let node_name = pod
            .spec
            .as_ref()
            .and_then(|s| s.node_name.as_deref())
            .unwrap_or("");

        let is_gpu_node = node_name.contains("vm168")
            || node_name.contains("vm169")
            || gpu_metrics_by_node.contains_key(node_name);

        if is_gpu_node {
            *node_pod_counts.entry(node_name.to_string()).or_insert(0) += 1;

            let name = pod.metadata.name.as_deref().unwrap_or("unknown");
            let namespace = pod.metadata.namespace.as_deref().unwrap_or("default");
            let phase = pod
                .status
                .as_ref()
                .and_then(|s| s.phase.as_deref())
                .unwrap_or("Unknown");

            let mut restart_count = 0;
            let mut container_images = Vec::new();
            let mut has_gpu_env = false;

            if let Some(spec) = &pod.spec {
                for c in &spec.containers {
                    if let Some(img) = &c.image {
                        container_images.push(img.clone());
                    }
                    if let Some(env) = &c.env {
                        for e in env {
                            if e.name.contains("NVIDIA") || e.name.contains("CUDA") {
                                has_gpu_env = true;
                            }
                        }
                    }
                }
            }

            let mut reason = String::new();
            if let Some(status) = &pod.status {
                if let Some(cs_list) = &status.container_statuses {
                    for cs in cs_list {
                        restart_count += cs.restart_count;
                        if let Some(waiting) = cs.state.as_ref().and_then(|st| st.waiting.as_ref()) {
                            if let Some(r) = &waiting.reason {
                                reason = r.clone();
                            }
                        }
                    }
                }
            }

            let age = pod
                .metadata
                .creation_timestamp
                .as_ref()
                .map(calculate_age_from_timestamp)
                .unwrap_or_default();

            gpu_pods.push(json!({
                "name": name,
                "namespace": namespace,
                "status": phase,
                "node": node_name,
                "restart_count": restart_count,
                "reason": reason,
                "age": age,
                "images": container_images,
                "has_gpu_env": has_gpu_env
            }));
        }
    }

    // Sort GPU pods by namespace, then name
    gpu_pods.sort_by(|a, b| {
        let ns_a = a["namespace"].as_str().unwrap_or("");
        let ns_b = b["namespace"].as_str().unwrap_or("");
        if ns_a == ns_b {
            let n_a = a["name"].as_str().unwrap_or("");
            let n_b = b["name"].as_str().unwrap_or("");
            n_a.cmp(n_b)
        } else {
            ns_a.cmp(ns_b)
        }
    });

    // 3. Format node telemetry for vm168, vm169 (and any other GPU node)
    let mut nodes_summary = Vec::new();
    let target_gpu_nodes = ["vm168", "vm169"];

    for node_prefix in target_gpu_nodes {
        // Find matching metrics from Prometheus
        let metrics = gpu_metrics_by_node
            .iter()
            .find(|(k, _)| k.contains(node_prefix))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();

        let pods_on_node = node_pod_counts
            .iter()
            .find(|(k, _)| k.contains(node_prefix))
            .map(|(_, count)| *count)
            .unwrap_or(0);

        nodes_summary.push(json!({
            "name": node_prefix,
            "status": if metrics.utilization >= 0.0 { "Ready" } else { "Unknown" },
            "utilization": (metrics.utilization * 10.0).round() / 10.0,
            "temperature": (metrics.temperature * 10.0).round() / 10.0,
            "power_usage": (metrics.power_draw * 10.0).round() / 10.0,
            "memory_used_mb": (metrics.memory_used * 10.0).round() / 10.0,
            "memory_total_mb": (metrics.memory_total * 10.0).round() / 10.0,
            "memory_free_mb": ((metrics.memory_total - metrics.memory_used).max(0.0) * 10.0).round() / 10.0,
            "memory_utilization": if metrics.memory_total > 0.0 {
                ((metrics.memory_used / metrics.memory_total) * 1000.0).round() / 10.0
            } else {
                0.0
            },
            "pods_count": pods_on_node
        }));
    }

    // 4. Query Managed GPU Workloads (e.g., facefusion, llama-cpp, etc.)
    let mut managed_workloads = Vec::new();
    let known_workloads = [
        ("facefusion", "facefusion", "FaceFusion AI (Face Swap / Video Enhancer)"),
        ("llama-cpp", "llama-cpp", "Llama.cpp Server (Qwen3.8-27B GGUF)"),
    ];

    for (ns, name, desc) in known_workloads {
        let deployments: Api<Deployment> = Api::namespaced(kube_client.clone(), ns);
        let (replicas, available_replicas, exists) = match deployments.get(name).await {
            Ok(dep) => {
                let spec_replicas = dep.spec.and_then(|s| s.replicas).unwrap_or(0);
                let status_replicas = dep.status.and_then(|s| s.available_replicas).unwrap_or(0);
                (spec_replicas, status_replicas, true)
            }
            Err(_) => (0, 0, false),
        };

        if exists {
            managed_workloads.push(json!({
                "name": name,
                "namespace": ns,
                "description": desc,
                "replicas": replicas,
                "available_replicas": available_replicas,
                "running": replicas > 0 && available_replicas > 0,
                "status": if replicas == 0 { "Stopped" } else if available_replicas >= replicas { "Running" } else { "Scaling" }
            }));
        }
    }

    let result = json!({
        "nodes": nodes_summary,
        "pods": gpu_pods,
        "pods_count": gpu_pods.len(),
        "workloads": managed_workloads
    });

    // Cache for 10 seconds
    if let Ok(json_str) = serde_json::to_string(&result) {
        state
            .general_cache
            .set(
                CACHE_KEY.to_string(),
                json_str,
                Some(std::time::Duration::from_secs(10)),
            )
            .await;
    }

    api_success(result)
}

/// Scale a workload (e.g. facefusion replicas 0 or 1)
pub async fn scale_workload_handler(
    State(state): State<AppState>,
    axum::Json(payload): axum::Json<ScaleWorkloadRequest>,
) -> Response {
    let kube_client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to connect to Kubernetes: {}", e),
            )
        }
    };

    let deployments: Api<Deployment> = Api::namespaced(kube_client, &payload.namespace);
    let patch_json = json!({
        "spec": {
            "replicas": payload.replicas
        }
    });

    let patch = Patch::Merge(&patch_json);
    let params = PatchParams::default();

    match deployments.patch(&payload.deployment, &params, &patch).await {
        Ok(_) => {
            // Invalidate GPU cache so UI sees update immediately
            state.general_cache.delete("kusanagi_gpu_status").await;
            api_success(json!({
                "success": true,
                "message": format!(
                    "Deployment {}/{} scaled to {} replicas",
                    payload.namespace, payload.deployment, payload.replicas
                )
            }))
        }
        Err(e) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to scale deployment: {}", e),
        ),
    }
}

#[derive(Default, Clone)]
struct NodeGpuMetrics {
    utilization: f64,
    temperature: f64,
    power_draw: f64,
    memory_used: f64,
    memory_total: f64,
}

async fn fetch_dcgm_metrics_by_node(
    client: &reqwest::Client,
    prometheus_url: &str,
) -> HashMap<String, NodeGpuMetrics> {
    let mut map: HashMap<String, NodeGpuMetrics> = HashMap::new();
    let url = format!("{}/api/v1/query", prometheus_url);

    // Queries to fetch DCGM metrics grouped by node or hostname
    let queries = [
        ("util", "avg(DCGM_FI_DEV_GPU_UTIL) by (kubernetes_node, Hostname, node, hostname, instance)"),
        ("temp", "avg(DCGM_FI_DEV_GPU_TEMP) by (kubernetes_node, Hostname, node, hostname, instance)"),
        ("power", "avg(DCGM_FI_DEV_POWER_USAGE) by (kubernetes_node, Hostname, node, hostname, instance)"),
        ("mem_used", "avg(DCGM_FI_DEV_FB_USED) by (kubernetes_node, Hostname, node, hostname, instance)"),
        ("mem_free", "avg(DCGM_FI_DEV_FB_FREE) by (kubernetes_node, Hostname, node, hostname, instance)"),
    ];

    for (metric_type, query) in queries {
        if let Ok(resp) = client
            .get(&url)
            .query(&[("query", query)])
            .timeout(std::time::Duration::from_secs(4))
            .send()
            .await
        {
            if !resp.status().is_success() {
                continue;
            }
            if let Ok(body) = resp.json::<Value>().await {
                if let Some(results) = body.get("data").and_then(|d| d.get("result")).and_then(|r| r.as_array()) {
                    for res in results {
                        if let (Some(metric), Some(value)) = (res.get("metric"), res.get("value")) {
                            let node_name = metric
                                .get("kubernetes_node")
                                .or_else(|| metric.get("Hostname"))
                                .or_else(|| metric.get("hostname"))
                                .or_else(|| metric.get("node"))
                                .or_else(|| metric.get("instance"))
                                .and_then(|s| s.as_str());

                            if let Some(node) = node_name {
                                let clean_node = node.split(':').next().unwrap_or(node).to_string();
                                if let Some(val_str) = value.get(1).and_then(|v| v.as_str()) {
                                    if let Ok(val) = val_str.parse::<f64>() {
                                        let entry = map.entry(clean_node).or_default();
                                        match metric_type {
                                            "util" => entry.utilization = val,
                                            "temp" => entry.temperature = val,
                                            "power" => entry.power_draw = val,
                                            "mem_used" => entry.memory_used = val,
                                            "mem_free" => entry.memory_total = entry.memory_used + val,
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Also support external GPU-HOT fallback if DCGM is empty
    if map.is_empty() {
        let hot_url = std::env::var("GPU_HOT_URL")
            .unwrap_or_else(|_| "https://gpu-hot.p.zacharie.org".to_string());
        let api_url = format!("{}/api/gpu-data", hot_url);
        if let Ok(resp) = client
            .get(&api_url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(data) = resp.json::<Value>().await {
                    if let Some(gpus) = data.get("gpus").and_then(|g| g.as_object()) {
                        for (key, gpu_info) in gpus {
                            let node_key = if key.contains("168") {
                                "vm168".to_string()
                            } else if key.contains("169") {
                                "vm169".to_string()
                            } else {
                                "vm168".to_string()
                            };

                            let mut entry = NodeGpuMetrics::default();
                            entry.utilization = gpu_info["utilization"].as_f64().unwrap_or(0.0);
                            entry.temperature = gpu_info["temperature"].as_f64().unwrap_or(0.0);
                            entry.power_draw = gpu_info["power_draw"].as_f64().unwrap_or(0.0);
                            entry.memory_used = gpu_info["memory_used"].as_f64().unwrap_or(0.0);
                            entry.memory_total = gpu_info["memory_total"].as_f64().unwrap_or(24576.0);
                            map.insert(node_key, entry);
                        }
                    }
                }
            }
        }
    }

    map
}
