use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterOverview {
    pub cluster_name: String,
    pub node_count: i32,
    pub pod_count: i32,
    pub namespace_count: i32,
    pub healthy_nodes: i32,
    pub running_pods: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
    pub roles: Vec<String>,
    pub age: String,
    pub version: String,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub node: Option<String>,
    pub age: String,
    pub restarts: i32,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub name: String,
    pub namespace: Option<String>,
    pub reason: String,
    pub message: String,
    pub type_: String,
    pub age: String,
    pub object: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusMetrics {
    pub cluster_cpu_usage: f64,
    pub cluster_memory_usage: f64,
    pub node_metrics: Vec<NodeMetric>,
    pub pod_metrics: Vec<PodMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetric {
    pub node_name: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodMetric {
    pub pod_name: String,
    pub namespace: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
}

impl Default for ClusterOverview {
    fn default() -> Self {
        Self {
            cluster_name: "unknown".to_string(),
            node_count: 0,
            pod_count: 0,
            namespace_count: 0,
            healthy_nodes: 0,
            running_pods: 0,
            status: "Unknown".to_string(),
        }
    }
}
