// Use Cases - Business Logic
use std::sync::Arc;
use crate::domain::ports::ClusterRepository;
use crate::domain::entities::{ClusterInfo, NodeInfo};
use crate::error::Result;

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
