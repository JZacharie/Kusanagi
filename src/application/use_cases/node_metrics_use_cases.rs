//! Node Metrics Use Cases
//!
//! Application layer use cases for node metrics with Prometheus integration.

use crate::domain::entities::{Node, NodeResources};
use crate::domain::ports::{KubernetesRepository, MetricsRepository};
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// Get nodes with disk metrics use case
pub struct GetNodesWithDiskMetricsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
    metrics_repo: Arc<dyn MetricsRepository>,
}

impl GetNodesWithDiskMetricsUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>, metrics_repo: Arc<dyn MetricsRepository>) -> Self {
        Self { k8s_repo, metrics_repo }
    }

    pub async fn execute(&self) -> Result<Vec<Node>> {
        // Get nodes from Kubernetes
        let mut nodes = self.k8s_repo.list_nodes().await?;
        
        // Get disk usage metrics from Prometheus for each node
        for node in &mut nodes {
            let node_name = &node.name;
            
            // Query Prometheus for disk usage percentage
            // Query: (node_filesystem_size_bytes - node_filesystem_free_bytes) / node_filesystem_size_bytes * 100
            let query = format!(
                "100 - ((node_filesystem_avail_bytes{{mountpoint=\"/\",instance=~\"{}:.*\"}} / node_filesystem_size_bytes{{mountpoint=\"/\",instance=~\"{}:.*\"}}) * 100)",
                node_name, node_name
            );
            
            match self.metrics_repo.query(&query).await {
                Ok(usage_percent) => {
                    node.resources.disk_usage_percent = Some(usage_percent);
                }
                Err(_) => {
                    // Try alternative query without instance label
                    let query_alt = "100 - ((node_filesystem_avail_bytes{mountpoint=\"/\"} / node_filesystem_size_bytes{mountpoint=\"/\"}) * 100)";
                    if let Ok(usage_percent) = self.metrics_repo.query(query_alt).await {
                        node.resources.disk_usage_percent = Some(usage_percent);
                    }
                }
            }
        }
        
        Ok(nodes)
    }
}

/// Get node disk usage for a specific node
pub struct GetNodeDiskUsageUseCase {
    metrics_repo: Arc<dyn MetricsRepository>,
}

impl GetNodeDiskUsageUseCase {
    pub fn new(metrics_repo: Arc<dyn MetricsRepository>) -> Self {
        Self { metrics_repo }
    }

    pub async fn execute(&self, node_name: &str) -> Result<NodeDiskMetrics> {
        // Query disk usage
        let usage_query = format!(
            "100 - ((node_filesystem_avail_bytes{{mountpoint=\"/\",instance=~\"{}:.*\"}} / node_filesystem_size_bytes{{mountpoint=\"/\",instance=~\"{}:.*\"}}) * 100)",
            node_name, node_name
        );
        
        let usage_percent = self.metrics_repo.query(&usage_query).await
            .unwrap_or(0.0);
        
        // Query total size
        let size_query = format!(
            "node_filesystem_size_bytes{{mountpoint=\"/\",instance=~\"{}:.*\"}} / (1024*1024*1024)",
            node_name
        );
        
        let total_gb = self.metrics_repo.query(&size_query).await
            .unwrap_or(0.0);
        
        // Query used size
        let used_query = format!(
            "(node_filesystem_size_bytes{{mountpoint=\"/\",instance=~\"{}:.*\"}} - node_filesystem_avail_bytes{{mountpoint=\"/\",instance=~\"{}:.*\"}}) / (1024*1024*1024)",
            node_name, node_name
        );
        
        let used_gb = self.metrics_repo.query(&used_query).await
            .unwrap_or(0.0);
        
        Ok(NodeDiskMetrics {
            node_name: node_name.to_string(),
            usage_percent,
            total_gb: format!("{:.1} Gi", total_gb),
            used_gb: format!("{:.1} Gi", used_gb),
            free_gb: format!("{:.1} Gi", total_gb - used_gb),
        })
    }
}

/// Node disk metrics DTO
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeDiskMetrics {
    pub node_name: String,
    pub usage_percent: f64,
    pub total_gb: String,
    pub used_gb: String,
    pub free_gb: String,
}

/// Get cluster disk summary use case
pub struct GetClusterDiskSummaryUseCase {
    metrics_repo: Arc<dyn MetricsRepository>,
}

impl GetClusterDiskSummaryUseCase {
    pub fn new(metrics_repo: Arc<dyn MetricsRepository>) -> Self {
        Self { metrics_repo }
    }

    pub async fn execute(&self) -> Result<ClusterDiskSummary> {
        // Average disk usage across cluster
        let avg_usage_query = "avg(100 - ((node_filesystem_avail_bytes{mountpoint=\"/\"} / node_filesystem_size_bytes{mountpoint=\"/\"}) * 100))";
        
        let avg_usage = self.metrics_repo.query(avg_usage_query).await
            .unwrap_or(0.0);
        
        // Max disk usage
        let max_usage_query = "max(100 - ((node_filesystem_avail_bytes{mountpoint=\"/\"} / node_filesystem_size_bytes{mountpoint=\"/\"}) * 100))";
        
        let max_usage = self.metrics_repo.query(max_usage_query).await
            .unwrap_or(0.0);
        
        // Total cluster storage
        let total_storage_query = "sum(node_filesystem_size_bytes{mountpoint=\"/\"}) / (1024*1024*1024*1024)";
        
        let total_tb = self.metrics_repo.query(total_storage_query).await
            .unwrap_or(0.0);
        
        Ok(ClusterDiskSummary {
            average_usage_percent: avg_usage,
            max_usage_percent: max_usage,
            total_storage_tb: format!("{:.2} Ti", total_tb),
            status: if max_usage > 90.0 {
                "Critical"
            } else if max_usage > 75.0 {
                "Warning"
            } else {
                "Healthy"
            }.to_string(),
        })
    }
}

/// Cluster disk summary DTO
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClusterDiskSummary {
    pub average_usage_percent: f64,
    pub max_usage_percent: f64,
    pub total_storage_tb: String,
    pub status: String,
}
