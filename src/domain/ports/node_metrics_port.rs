use async_trait::async_trait;
use crate::error::Result;

/// Port for node metrics operations
#[async_trait]
pub trait NodeMetricsRepository: Send + Sync {
    /// Get disk usage metrics for a specific node
    async fn get_node_disk_metrics(&self, node_name: &str) -> Result<NodeDiskMetrics>;
    
    /// Get disk usage metrics for all nodes
    async fn list_nodes_with_disk_metrics(&self) -> Result<Vec<NodeDiskMetrics>>;
    
    /// Get cluster-wide disk summary
    async fn get_cluster_disk_summary(&self) -> Result<ClusterDiskSummary>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeDiskMetrics {
    pub node_name: String,
    pub disk_usage_percent: f64,
    pub disk_used_bytes: i64,
    pub disk_total_bytes: i64,
    pub disk_available_bytes: i64,
    pub filesystem: String,
    pub mount_point: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClusterDiskSummary {
    pub total_nodes: usize,
    pub average_disk_usage: f64,
    pub highest_usage_node: Option<String>,
    pub highest_usage_percent: f64,
    pub total_disk_bytes: i64,
    pub total_used_bytes: i64,
    pub nodes: Vec<NodeDiskMetrics>,
}
