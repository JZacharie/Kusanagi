//! Kubernetes Repository Port
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait KubernetesRepository: Send + Sync {
    /// Get status of all pods
    async fn get_pods_status(&self) -> Result<Value>;

    /// Get status of all nodes
    async fn get_nodes_status(&self) -> Result<Value>;

    /// Get cluster overview
    async fn get_cluster_overview(&self) -> Result<Value>;
}
