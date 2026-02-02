//! Service Use Cases
//!
//! Application layer use cases for service operations.

use crate::domain::entities::Service;
use crate::domain::ports::KubernetesRepository;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// List services use case
pub struct ListServicesUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl ListServicesUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<Vec<Service>> {
        self.repository.list_services(namespace).await
    }
}

/// Get service details use case
pub struct GetServiceDetailsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetServiceDetailsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: &str, name: &str) -> Result<Service> {
        // List services in namespace and find the specific one
        let services = self.repository.list_services(Some(namespace)).await?;
        services
            .into_iter()
            .find(|s| s.name == name)
            .ok_or_else(|| KusanagiError::not_found("Service", name))
    }
}

/// Service statistics DTO
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceStats {
    pub total_count: usize,
    pub by_type: std::collections::HashMap<String, usize>,
}

/// Get service statistics use case
pub struct GetServiceStatsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetServiceStatsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<ServiceStats> {
        let services = self.repository.list_services(namespace).await?;
        
        let total_count = services.len();
        let mut by_type = std::collections::HashMap::new();
        
        for service in services {
            *by_type.entry(service.service_type).or_insert(0) += 1;
        }
        
        Ok(ServiceStats {
            total_count,
            by_type,
        })
    }
}

/// Service service - aggregates all service use cases
pub struct ServiceUseCaseService {
    pub list: ListServicesUseCase,
    pub get_details: GetServiceDetailsUseCase,
    pub get_stats: GetServiceStatsUseCase,
}

impl ServiceUseCaseService {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self {
            list: ListServicesUseCase::new(repository.clone()),
            get_details: GetServiceDetailsUseCase::new(repository.clone()),
            get_stats: GetServiceStatsUseCase::new(repository),
        }
    }
}
