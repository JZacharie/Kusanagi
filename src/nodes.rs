use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::{Node, Pod};
use kube::{
    api::{Api, ListParams},
    Client,
};
use serde::Serialize;
use tracing::{error, info};
use std::collections::HashMap;
use serde_json::Value;

/// Node status response
#[derive(Clone, Debug, Serialize)]
pub struct NodesStatusResponse {
    pub total_nodes: usize,
    pub ready_nodes: usize,
    pub not_ready_nodes: usize,
    pub nodes: Vec<NodeInfo>,
}

/// Individual node information
#[derive(Clone, Debug, Serialize)]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
    pub architecture: String,
    pub os: String,
    pub kernel_version: String,
    pub kubelet_version: String,
    pub container_runtime: String,
    pub cpu_capacity: String,
    pub cpu_allocatable: String,
    pub memory_capacity: String,
    pub memory_allocatable: String,
    pub pod_count: usize,
    pub pod_capacity: String,
    pub pods_in_error: usize,
    pub error_pod_names: Vec<String>,
    pub uptime: Option<String>,
    pub uptime_seconds: Option<i64>,
    pub conditions: Vec<NodeCondition>,
    pub labels: std::collections::BTreeMap<String, String>,
    pub cpu_usage_percent: Option<f64>,
    pub memory_usage_percent: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NodeCondition {
    pub condition_type: String,
    pub status: String,
    pub message: Option<String>,
}

/// Get all nodes status with resource information
pub async fn get_nodes_status() -> Result<NodesStatusResponse, String> {
    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create Kubernetes client: {}", e))?;

    let nodes_api: Api<Node> = Api::all(client.clone());
    let pods_api: Api<Pod> = Api::all(client);

    info!("Starting get_nodes_status...");

    let nodes = nodes_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list nodes: {}", e))?;


    let pods = match pods_api.list(&ListParams::default()).await {
        Ok(p) => Some(p),
        Err(e) => {
            error!("Failed to list pods (continuing without pod info): {}", e);
            None
        }
    };

    // Fetch metrics from Prometheus (Parallel & with timeout)
    info!("Fetching Prometheus metrics...");
    let (cpu_metrics, mem_metrics) = tokio::join!(
        fetch_prometheus_metric("avg(rate(node_cpu_seconds_total{mode!=\"idle\"}[5m])) by (instance) * 100"),
        fetch_prometheus_metric("(1 - node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes) * 100")
    );
    info!("Prometheus metrics fetched. CPU count: {}, Mem count: {}", cpu_metrics.len(), mem_metrics.len());

    let now = Utc::now();
    let mut response = NodesStatusResponse {
        total_nodes: nodes.items.len(),
        ready_nodes: 0,
        not_ready_nodes: 0,
        nodes: Vec::new(),
    };

    for node in nodes.items {
        let name = node.metadata.name.clone().unwrap_or_default();
        let labels = node.metadata.labels.clone().unwrap_or_default();
        
        let state = node.status.as_ref();
        
        // Get InternalIP for metrics mapping
        let internal_ip = state
            .and_then(|s| s.addresses.as_ref())
            .and_then(|addrs| {
                addrs.iter().find(|a| a.type_ == "InternalIP").map(|a| a.address.clone())
            });

        // Get node info
        let node_info = state.and_then(|s| s.node_info.as_ref());
        
        let architecture = node_info
            .map(|i| i.architecture.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        let os = node_info
            .map(|i| i.operating_system.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        let kernel_version = node_info
            .map(|i| i.kernel_version.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        let kubelet_version = node_info
            .map(|i| i.kubelet_version.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        let container_runtime = node_info
            .map(|i| i.container_runtime_version.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Get capacity
        let capacity = state.and_then(|s| s.capacity.as_ref());
        let allocatable = state.and_then(|s| s.allocatable.as_ref());

        let cpu_capacity = capacity
            .and_then(|c| c.get("cpu"))
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());

        let cpu_allocatable = allocatable
            .and_then(|a| a.get("cpu"))
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());

        let memory_capacity = capacity
            .and_then(|c| c.get("memory"))
            .map(|q| format_memory(&q.0))
            .unwrap_or_else(|| "0".to_string());

        let memory_allocatable = allocatable
            .and_then(|a| a.get("memory"))
            .map(|q| format_memory(&q.0))
            .unwrap_or_else(|| "0".to_string());

        let pod_capacity = capacity
            .and_then(|c| c.get("pods"))
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());

        // Count pods on this node
        let node_pods: Vec<&Pod> = pods.as_ref()
            .map(|p_list| {
                p_list.items
                    .iter()
                    .filter(|p| {
                        p.spec
                            .as_ref()
                            .and_then(|s| s.node_name.as_ref())
                            .map(|n| n == &name)
                            .unwrap_or(false)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let pod_count = node_pods.len();

        // Find pods in error state
        let error_pods: Vec<String> = node_pods
            .iter()
            .filter(|p| is_pod_in_error(p))
            .filter_map(|p| p.metadata.name.clone())
            .collect();
        
        let pods_in_error = error_pods.len();

        // Get node conditions
        let conditions: Vec<NodeCondition> = state
            .and_then(|s| s.conditions.as_ref())
            .map(|conds| {
                conds
                    .iter()
                    .map(|c| NodeCondition {
                        condition_type: c.type_.clone(),
                        status: c.status.clone(),
                        message: c.message.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Check if node is ready
        let is_ready = conditions
            .iter()
            .any(|c| c.condition_type == "Ready" && c.status == "True");

        let node_status = if is_ready {
            response.ready_nodes += 1;
            "Ready".to_string()
        } else {
            response.not_ready_nodes += 1;
            "NotReady".to_string()
        };

        // Calculate uptime from creation timestamp
        let (uptime, uptime_seconds) = node
            .metadata
            .creation_timestamp
            .as_ref()
            .and_then(|ts| {
                DateTime::parse_from_rfc3339(&ts.0.to_rfc3339()).ok().map(|dt| {
                    let duration = now.signed_duration_since(dt.with_timezone(&Utc));
                    let seconds = duration.num_seconds();
                    (Some(format_uptime(seconds)), Some(seconds))
                })
            })
            .unwrap_or((None, None));

        response.nodes.push(NodeInfo {
            name,
            status: node_status,
            architecture,
            os,
            kernel_version,
            kubelet_version,
            container_runtime,
            cpu_capacity,
            cpu_allocatable,
            memory_capacity,
            memory_allocatable,
            pod_count,
            pod_capacity,
            pods_in_error,
            error_pod_names: error_pods,
            uptime,
            uptime_seconds,
            conditions,
            labels,
            cpu_usage_percent: internal_ip.as_ref().and_then(|ip| cpu_metrics.get(ip).copied()),
            memory_usage_percent: internal_ip.as_ref().and_then(|ip| mem_metrics.get(ip).copied()),
        });
    }

    // Sort nodes by name
    response.nodes.sort_by(|a, b| a.name.cmp(&b.name));

    info!(
        "Nodes status: {} total, {} ready, {} not ready",
        response.total_nodes, response.ready_nodes, response.not_ready_nodes
    );

    Ok(response)
}

/// Check if a pod is in error state
fn is_pod_in_error(pod: &Pod) -> bool {
    let phase = pod
        .status
        .as_ref()
        .and_then(|s| s.phase.as_ref())
        .map(|p| p.as_str())
        .unwrap_or("");

    // Check phase
    if phase == "Failed" || phase == "Unknown" {
        return true;
    }

    // Check container statuses for CrashLoopBackOff, Error, etc.
    if let Some(status) = &pod.status {
        if let Some(container_statuses) = &status.container_statuses {
            for cs in container_statuses {
                if let Some(waiting) = &cs.state.as_ref().and_then(|s| s.waiting.as_ref()) {
                    let reason = waiting.reason.as_deref().unwrap_or("");
                    if reason == "CrashLoopBackOff"
                        || reason == "Error"
                        || reason == "ImagePullBackOff"
                        || reason == "ErrImagePull"
                        || reason == "CreateContainerError"
                    {
                        return true;
                    }
                }
                // Check restart count
                if cs.restart_count > 5 {
                    return true;
                }
            }
        }
    }

    false
}

/// Format memory from Ki to human readable
fn format_memory(ki_str: &str) -> String {
    // Remove Ki suffix and parse
    let value = ki_str
        .trim_end_matches("Ki")
        .trim_end_matches("Mi")
        .trim_end_matches("Gi")
        .parse::<f64>()
        .unwrap_or(0.0);

    if ki_str.ends_with("Gi") {
        format!("{:.1}Gi", value)
    } else if ki_str.ends_with("Mi") {
        format!("{:.0}Mi", value)
    } else if ki_str.ends_with("Ki") {
        let gi = value / 1024.0 / 1024.0;
        if gi >= 1.0 {
            format!("{:.1}Gi", gi)
        } else {
            let mi = value / 1024.0;
            format!("{:.0}Mi", mi)
        }
    } else {
        ki_str.to_string()
    }
}

/// Format uptime in human readable format
fn format_uptime(seconds: i64) -> String {
    if seconds < 0 {
        return "just started".to_string();
    }

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", seconds)
    }
}

/// Fetch a metric from Prometheus
async fn fetch_prometheus_metric(query: &str) -> HashMap<String, f64> {
    let prometheus_url = std::env::var("PROMETHEUS_URL")
        .unwrap_or_else(|_| "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string());
    
    let url = format!("{}/api/v1/query", prometheus_url);
    info!("Querying Prometheus at {}: {}", url, query);
    
    let mut client_builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3));

    // Optional Basic Auth
    if let (Ok(u), Ok(p)) = (std::env::var("PROMETHEUS_USERNAME"), std::env::var("PROMETHEUS_PASSWORD")) {
        client_builder = client_builder.danger_accept_invalid_certs(true); // Self-signed often used with auth
        // We handle auth in the request builder usually, but reqwest client builder doesn't store auth globally easily implies cookie store etc.
        // Actually it's easier to set it on the client or request.
        // Let's rely on standard builder logic.
    }

    let client = match client_builder.build() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to build Prometheus client: {}", e);
                return HashMap::new();
            }
        };

    let mut request = client.get(&url).query(&[("query", query)]);

    if let (Ok(u), Ok(p)) = (std::env::var("PROMETHEUS_USERNAME"), std::env::var("PROMETHEUS_PASSWORD")) {
        request = request.basic_auth(u, Some(p));
    }

    let response = match request.send().await {
        Ok(res) => res,
        Err(e) => {
            error!("Failed to query Prometheus: {}", e);
            return HashMap::new();
        }
    };

    if !response.status().is_success() {
        error!("Prometheus returned error status: {}", response.status());
        return HashMap::new();
    }

    let body: Value = match response.json().await {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to parse Prometheus response: {}", e);
            return HashMap::new();
        }
    };

    let mut metrics = HashMap::new();

    if let Some(result) = body.get("data").and_then(|d| d.get("result")).and_then(|r| r.as_array()) {
        for item in result {
            if let Some(metric) = item.get("metric") {
                if let Some(instance) = metric.get("instance").and_then(|i| i.as_str()) {
                    // Extract IP from instance (remove port if present)
                    let ip = instance.split(':').next().unwrap_or(instance).to_string();
                    
                    if let Some(value) = item.get("value").and_then(|v| v.as_array()) {
                        if value.len() >= 2 {
                            if let Some(val_str) = value[1].as_str() {
                                if let Ok(val) = val_str.parse::<f64>() {
                                    metrics.insert(ip, val);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    metrics
}
