//! Node Use Cases
//!
//! Application layer use cases for node operations.

use crate::domain::entities::{Node, NodeStatus};
use crate::domain::ports::KubernetesRepository;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// Get all nodes use case
pub struct GetNodesUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetNodesUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Vec<Node>> {
        self.repository.list_nodes().await
    }
}

/// Get node details use case
pub struct GetNodeDetailsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetNodeDetailsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, name: &str) -> Result<Node> {
        self.repository.get_node(name).await
    }
}

/// Get nodes status summary use case
pub struct GetNodesStatusUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetNodesStatusUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<NodesStatus> {
        let nodes = self.repository.list_nodes().await?;
        
        let total = nodes.len();
        let ready = nodes.iter().filter(|n| n.status == NodeStatus::Ready).count();
        let not_ready = nodes.iter().filter(|n| n.status == NodeStatus::NotReady).count();
        
        Ok(NodesStatus {
            total_nodes: total,
            ready_nodes: ready,
            not_ready_nodes: not_ready,
        })
    }
}

/// Nodes status summary
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodesStatus {
    pub total_nodes: usize,
    pub ready_nodes: usize,
    pub not_ready_nodes: usize,
}

/// Check if node is ready use case
pub struct IsNodeReadyUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl IsNodeReadyUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, name: &str) -> Result<bool> {
        let node = self.repository.get_node(name).await?;
        Ok(node.status.is_ready())
    }
}

/// Get nodes by status use case
pub struct GetNodesByStatusUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetNodesByStatusUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, status: NodeStatus) -> Result<Vec<Node>> {
        let nodes = self.repository.list_nodes().await?;
        Ok(nodes.into_iter().filter(|n| n.status == status).collect())
    }
}

/// Node service - aggregates all node use cases
pub struct NodeService {
    pub get_all: GetNodesUseCase,
    pub get_details: GetNodeDetailsUseCase,
    pub get_status: GetNodesStatusUseCase,
    pub is_ready: IsNodeReadyUseCase,
    pub get_by_status: GetNodesByStatusUseCase,
}

impl NodeService {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self {
            get_all: GetNodesUseCase::new(repository.clone()),
            get_details: GetNodeDetailsUseCase::new(repository.clone()),
            get_status: GetNodesStatusUseCase::new(repository.clone()),
            is_ready: IsNodeReadyUseCase::new(repository.clone()),
            get_by_status: GetNodesByStatusUseCase::new(repository),
        }
    }
}
