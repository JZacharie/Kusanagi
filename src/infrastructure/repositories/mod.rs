// Repository Implementations
use async_trait::async_trait;
use crate::domain::ports::ClusterRepository;
use crate::domain::entities::{ClusterInfo, NodeInfo};
use crate::error::Result;

pub struct MockClusterRepository;

#[async_trait]
impl ClusterRepository for MockClusterRepository {
    async fn get_cluster_info(&self) -> Result<ClusterInfo> {
        Ok(ClusterInfo {
            name: "kusanagi-cluster".to_string(),
            version: "v1.28.0".to_string(),
            status: "healthy".to_string(),
            nodes: 3,
        })
    }
    
    async fn get_nodes(&self) -> Result<Vec<NodeInfo>> {
        Ok(vec![
            NodeInfo {
                name: "master-01".to_string(),
                status: "Ready".to_string(),
                role: "control-plane".to_string(),
            },
            NodeInfo {
                name: "worker-01".to_string(),
                status: "Ready".to_string(),
                role: "worker".to_string(),
            },
        ])
    }
}
