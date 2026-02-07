// Use Cases - Business Logic
use crate::domain::entities::{ClusterInfo, NodeInfo};
use crate::domain::ports::ClusterRepository;
use crate::error::Result;
use std::sync::Arc;

pub struct ClusterUseCase {
    repository: Arc<dyn ClusterRepository>,
}

impl ClusterUseCase {
    pub fn new(repository: Arc<dyn ClusterRepository>) -> Self {
        Self { repository }
    }

    pub async fn get_cluster_status(&self) -> Result<ClusterInfo> {
        self.repository.get_cluster_info().await
    }

    pub async fn list_nodes(&self) -> Result<Vec<NodeInfo>> {
        self.repository.get_nodes().await
    }
}
