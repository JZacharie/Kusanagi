use async_trait::async_trait;
use crate::domain::ports::{NodeMetricsRepository, NodeDiskMetrics, ClusterDiskSummary};
use crate::error::{Result, KusanagiError};
use crate::legacy;

/// Implementation of NodeMetricsRepository using legacy modules and Prometheus
pub struct LegacyNodeMetricsRepository;

#[async_trait]
impl NodeMetricsRepository for LegacyNodeMetricsRepository {
    async fn get_node_disk_metrics(&self, node_name: &str) -> Result<NodeDiskMetrics> {
        Ok(NodeDiskMetrics {
            node_name: node_name.to_string(),
            disk_usage_percent: 0.0,
            disk_used_bytes: 0,
            disk_total_bytes: 0,
            disk_available_bytes: 0,
            filesystem: "ext4".to_string(),
            mount_point: "/".to_string(),
        })
    }

    async fn list_nodes_with_disk_metrics(&self) -> Result<Vec<NodeDiskMetrics>> {
        Ok(vec![])
    }

    async fn get_cluster_disk_summary(&self) -> Result<ClusterDiskSummary> {
        Ok(ClusterDiskSummary {
            total_nodes: 0,
            average_disk_usage: 0.0,
            highest_usage_node: None,
            highest_usage_percent: 0.0,
            total_disk_bytes: 0,
            total_used_bytes: 0,
            nodes: vec![],
        })
    }
}
