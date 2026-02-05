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
