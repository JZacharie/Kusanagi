//! Cluster-specific use cases

use crate::application::dtos::*;
use crate::application::mappers::*;
use crate::domain::entities::*;
use crate::domain::ports::*;
use crate::error::Result;
use std::sync::Arc;

/// Use case: Get cluster health summary
pub struct GetClusterHealthUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
    metrics_repo: Arc<dyn MetricsRepository>,
}

impl GetClusterHealthUseCase {
    pub fn new(
        k8s_repo: Arc<dyn KubernetesRepository>,
        metrics_repo: Arc<dyn MetricsRepository>,
    ) -> Self {
        Self {
            k8s_repo,
            metrics_repo,
        }
    }

    pub async fn execute(&self) -> Result<ClusterHealthDto> {
        let overview = self.k8s_repo.get_cluster_overview().await?;
        let metrics = self.metrics_repo.get_cluster_metrics().await?;
        
        Ok(ClusterHealthDto {
            status: format!("{:?}", overview.status),
            node_count: overview.node_count,
            pod_count: overview.pod_count,
            cpu_percent: metrics.cpu_percent,
            memory_percent: metrics.memory_percent,
            issues: vec![], // Would be populated from health checks
        })
    }
}

/// DTO for cluster health
#[derive(Debug, Clone, serde::Serialize)]
pub struct ClusterHealthDto {
    pub status: String,
    pub node_count: usize,
    pub pod_count: usize,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub issues: Vec<String>,
}

/// Use case: Get empty namespaces
pub struct GetEmptyNamespacesUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetEmptyNamespacesUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self) -> Result<Vec<String>> {
        let namespaces = self.k8s_repo.list_namespaces().await?;
        
        let empty: Vec<String> = namespaces
            .into_iter()
            .filter(|ns| ns.pod_count == 0)
            .map(|ns| ns.name)
            .collect();
        
        Ok(empty)
    }
}

/// Use case: Get resource quota status
pub struct GetResourceQuotaUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetResourceQuotaUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self) -> Result<Vec<ResourceQuotaDto>> {
        let namespaces = self.k8s_repo.list_namespaces().await?;
        
        let quotas: Vec<ResourceQuotaDto> = namespaces
            .into_iter()
            .filter_map(|ns| {
                ns.resource_quota.map(|rq| ResourceQuotaDto {
                    namespace: ns.name,
                    hard: rq.hard,
                    used: rq.used,
                })
            })
            .collect();
        
        Ok(quotas)
    }
}

/// DTO for resource quota
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceQuotaDto {
    pub namespace: String,
    pub hard: std::collections::HashMap<String, String>,
    pub used: std::collections::HashMap<String, String>,
}
