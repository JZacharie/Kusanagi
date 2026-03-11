use crate::domain::entities::{ClusterInfo, NodeInfo};
use crate::domain::ports::ClusterRepository;
use crate::domain::services::kubernetes_service;
use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct KubernetesClusterRepository {
    client: Arc<reqwest::Client>,
    cache: Arc<crate::AdvancedCache<String>>,
}

impl KubernetesClusterRepository {
    pub fn new(client: Arc<reqwest::Client>, cache: Arc<crate::AdvancedCache<String>>) -> Self {
        Self { client, cache }
    }
}

#[async_trait]
impl ClusterRepository for KubernetesClusterRepository {
    async fn get_cluster_info(&self) -> Result<ClusterInfo> {
        // We can get version and node count from get_nodes_status or get_cluster_overview
        // For now, let's use get_nodes_status as it gives us node count
        let nodes_data = kubernetes_service::get_nodes_status(&self.client, &self.cache, false)
            .await
            .map_err(|e| {
                crate::error::KusanagiError::ExternalService(format!(
                    "Failed to fetch node status: {}",
                    e
                ))
            })?;

        let nodes = nodes_data["nodes"].as_array().ok_or_else(|| {
            crate::error::KusanagiError::ExternalService("Invalid nodes data format".to_string())
        })?;

        let node_count = nodes.len() as u32;

        // Version is harder to get from just nodes, but let's try to find it or default
        // In a real k8s client we could get it from the server version
        // For now, let's placeholder it or extract from first node
        let version = if let Some(first) = nodes.first() {
            first["kubelet_version"]
                .as_str()
                .unwrap_or("unknown")
                .to_string()
        } else {
            "unknown".to_string()
        };

        Ok(ClusterInfo {
            name: "kusanagi-cluster".to_string(), // Could be env var
            version,
            status: "Active".to_string(), // Simple status
            nodes: node_count,
        })
    }

    async fn get_nodes(&self) -> Result<Vec<NodeInfo>> {
        let nodes_data = kubernetes_service::get_nodes_status(&self.client, &self.cache, false)
            .await
            .map_err(|e| {
                crate::error::KusanagiError::ExternalService(format!(
                    "Failed to fetch node status: {}",
                    e
                ))
            })?;

        let nodes_array = nodes_data["nodes"].as_array().ok_or_else(|| {
            crate::error::KusanagiError::ExternalService("Invalid nodes data format".to_string())
        })?;

        let mut result = Vec::new();
        for node in nodes_array {
            let name = node["name"].as_str().unwrap_or("unknown").to_string();
            let status = node["status"].as_str().unwrap_or("Unknown").to_string();
            let role = node["role"].as_str().unwrap_or("worker").to_string();

            result.push(NodeInfo { name, status, role });
        }

        Ok(result)
    }
}
