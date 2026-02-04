use async_trait::async_trait;
use crate::domain::ports::{NodeMetricsRepository, NodeDiskMetrics, ClusterDiskSummary};
use crate::error::{Result, KusanagiError};
use crate::legacy;

/// Implementation of NodeMetricsRepository using legacy modules and Prometheus
pub struct LegacyNodeMetricsRepository;

#[async_trait]
impl NodeMetricsRepository for LegacyNodeMetricsRepository {
    async fn get_node_disk_metrics(&self, node_name: &str) -> Result<NodeDiskMetrics> {
        // Use Prometheus to get disk metrics for specific node
        let prometheus_query = format!(
            r#"(1 - (node_filesystem_avail_bytes{{instance=~"{}.*",fstype!="tmpfs"}} / node_filesystem_size_bytes{{instance=~"{}.*",fstype!="tmpfs"}})) * 100"#,
            node_name, node_name
        );
        
        // For now, delegate to legacy prometheus module
        match legacy::prometheus::query_instant(&prometheus_query).await {
            Ok(result) => {
                // Parse Prometheus result and convert to NodeDiskMetrics
                let usage_percent = result.get("data")
                    .and_then(|d| d.get("result"))
                    .and_then(|r| r.get(0))
                    .and_then(|res| res.get("value"))
                    .and_then(|v| v.get(1))
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0);

                Ok(NodeDiskMetrics {
                    node_name: node_name.to_string(),
                    disk_usage_percent: usage_percent,
                    disk_used_bytes: 0, // Would need additional queries
                    disk_total_bytes: 0, // Would need additional queries
                    disk_available_bytes: 0, // Would need additional queries
                    filesystem: "ext4".to_string(), // Would get from actual data
                    mount_point: "/".to_string(), // Would get from actual data
                })
            }
            Err(e) => Err(KusanagiError::prometheus(format!("Failed to get node disk metrics: {}", e)))
        }
    }

    async fn list_nodes_with_disk_metrics(&self) -> Result<Vec<NodeDiskMetrics>> {
        // Get all nodes first
        let nodes = legacy::nodes::get_nodes_status().await
            .map_err(|e| KusanagiError::k8s(format!("Failed to get nodes: {}", e)))?;

        let mut metrics = Vec::new();
        
        // For each node, get disk metrics
        for node in nodes.nodes {
            match self.get_node_disk_metrics(&node.name).await {
                Ok(disk_metrics) => metrics.push(disk_metrics),
                Err(_) => {
                    // If we can't get metrics for a node, create a default entry
                    metrics.push(NodeDiskMetrics {
                        node_name: node.name,
                        disk_usage_percent: 0.0,
                        disk_used_bytes: 0,
                        disk_total_bytes: 0,
                        disk_available_bytes: 0,
                        filesystem: "unknown".to_string(),
                        mount_point: "/".to_string(),
                    });
                }
            }
        }

        Ok(metrics)
    }

    async fn get_cluster_disk_summary(&self) -> Result<ClusterDiskSummary> {
        let node_metrics = self.list_nodes_with_disk_metrics().await?;
        
        if node_metrics.is_empty() {
            return Ok(ClusterDiskSummary {
                total_nodes: 0,
                average_disk_usage: 0.0,
                highest_usage_node: None,
                highest_usage_percent: 0.0,
                total_disk_bytes: 0,
                total_used_bytes: 0,
                nodes: vec![],
            });
        }

        let total_nodes = node_metrics.len();
        let average_disk_usage = node_metrics.iter()
            .map(|n| n.disk_usage_percent)
            .sum::<f64>() / total_nodes as f64;

        let highest_usage_node = node_metrics.iter()
            .max_by(|a, b| a.disk_usage_percent.partial_cmp(&b.disk_usage_percent).unwrap())
            .map(|n| n.node_name.clone());

        let highest_usage_percent = node_metrics.iter()
            .map(|n| n.disk_usage_percent)
            .fold(0.0, f64::max);

        let total_disk_bytes = node_metrics.iter()
            .map(|n| n.disk_total_bytes)
            .sum();

        let total_used_bytes = node_metrics.iter()
            .map(|n| n.disk_used_bytes)
            .sum();

        Ok(ClusterDiskSummary {
            total_nodes,
            average_disk_usage,
            highest_usage_node,
            highest_usage_percent,
            total_disk_bytes,
            total_used_bytes,
            nodes: node_metrics,
        })
    }
}
