//! Cluster Use Cases
//!
//! Application layer use cases for cluster operations.

use crate::domain::entities::{ClusterOverview, Namespace};
use crate::domain::ports::KubernetesRepository;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// Get cluster overview use case
pub struct GetClusterOverviewUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetClusterOverviewUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<ClusterOverview> {
        self.repository.get_cluster_overview().await
    }
}

/// Get empty namespaces use case
pub struct GetEmptyNamespacesUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetEmptyNamespacesUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Vec<String>> {
        let namespaces = self.repository.list_namespaces().await?;
        
        // For now, return all namespaces as potentially empty
        // In a real implementation, we would check for workloads in each namespace
        let empty: Vec<String> = namespaces
            .into_iter()
            .filter(|ns| ns.pod_count == 0)
            .map(|ns| ns.name)
            .collect();
        
        Ok(empty)
    }
}

/// Get cluster statistics use case
pub struct GetClusterStatsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetClusterStatsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<ClusterStats> {
        let overview = self.repository.get_cluster_overview().await?;
        let namespaces = self.repository.list_namespaces().await?;
        let nodes = self.repository.list_nodes().await?;
        
        Ok(ClusterStats {
            namespace_count: namespaces.len(),
            node_count: nodes.len(),
            pod_count: overview.pod_count,
            healthy: overview.status == crate::domain::entities::ClusterStatus::Healthy,
        })
    }
}

/// Cluster statistics DTO
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClusterStats {
    pub namespace_count: usize,
    pub node_count: usize,
    pub pod_count: usize,
    pub healthy: bool,
}

/// Cluster use case service - aggregates all cluster use cases
pub struct ClusterUseCaseService {
    pub get_overview: GetClusterOverviewUseCase,
    pub get_empty_namespaces: GetEmptyNamespacesUseCase,
    pub get_stats: GetClusterStatsUseCase,
}

impl ClusterUseCaseService {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self {
            get_overview: GetClusterOverviewUseCase::new(repository.clone()),
            get_empty_namespaces: GetEmptyNamespacesUseCase::new(repository.clone()),
            get_stats: GetClusterStatsUseCase::new(repository),
        }
    }
}
