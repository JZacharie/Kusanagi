// Domain Ports - Hexagonal Architecture
use super::entities::{ClusterInfo, NodeInfo};
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ClusterRepository: Send + Sync {
    async fn get_cluster_info(&self) -> Result<ClusterInfo>;
    async fn get_nodes(&self) -> Result<Vec<NodeInfo>>;
}
