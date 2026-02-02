//! Ingress Use Cases
//!
//! Application layer use cases for ingress operations.

use crate::domain::entities::Ingress;
use crate::domain::ports::KubernetesRepository;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// List ingresses use case
pub struct ListIngressesUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl ListIngressesUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<Vec<Ingress>> {
        self.repository.list_ingresses(namespace).await
    }
}

/// Get ingress details use case
pub struct GetIngressDetailsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetIngressDetailsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: &str, name: &str) -> Result<Ingress> {
        let ingresses = self.repository.list_ingresses(Some(namespace)).await?;
        ingresses
            .into_iter()
            .find(|i| i.name == name)
            .ok_or_else(|| KusanagiError::not_found("Ingress", name))
    }
}

/// Ingress statistics DTO
#[derive(Debug, Clone, serde::Serialize)]
pub struct IngressStats {
    pub total_count: usize,
    pub total_hosts: usize,
    pub total_paths: usize,
    pub with_tls: usize,
}

/// Get ingress statistics use case
pub struct GetIngressStatsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetIngressStatsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<IngressStats> {
        let ingresses = self.repository.list_ingresses(namespace).await?;
        
        let total_count = ingresses.len();
        let total_hosts: usize = ingresses.iter().map(|i| i.hosts.len()).sum();
        let total_paths: usize = ingresses.iter().map(|i| i.paths.len()).sum();
        let with_tls = ingresses.iter().filter(|i| !i.tls.is_empty()).count();
        
        Ok(IngressStats {
            total_count,
            total_hosts,
            total_paths,
            with_tls,
        })
    }
}

/// Ingress service - aggregates all ingress use cases
pub struct IngressUseCaseService {
    pub list: ListIngressesUseCase,
    pub get_details: GetIngressDetailsUseCase,
    pub get_stats: GetIngressStatsUseCase,
}

impl IngressUseCaseService {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self {
            list: ListIngressesUseCase::new(repository.clone()),
            get_details: GetIngressDetailsUseCase::new(repository.clone()),
            get_stats: GetIngressStatsUseCase::new(repository),
        }
    }
}
