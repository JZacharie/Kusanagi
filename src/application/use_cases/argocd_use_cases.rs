//! ArgoCD Use Cases
//!
//! Application layer use cases for ArgoCD operations.

use crate::domain::ports::argocd_port::*;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// Get ArgoCD applications use case
pub struct GetArgoCdApplicationsUseCase {
    repository: Arc<dyn ArgoCdRepository>,
}

impl GetArgoCdApplicationsUseCase {
    pub fn new(repository: Arc<dyn ArgoCdRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Vec<ApplicationInfo>> {
        self.repository.list_applications().await
            .map_err(|_e| KusanagiError::internal("Failed to list applications".to_string()))
    }
}

/// Get application status use case
pub struct GetApplicationStatusUseCase {
    repository: Arc<dyn ArgoCdRepository>,
}

impl GetApplicationStatusUseCase {
    pub fn new(repository: Arc<dyn ArgoCdRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, name: &str) -> Result<ApplicationStatus> {
        self.repository.get_application_status(name).await
            .map_err(|_e| KusanagiError::not_found("Application", name))
    }
}

/// Sync application use case
pub struct SyncApplicationUseCase {
    repository: Arc<dyn ArgoCdRepository>,
}

impl SyncApplicationUseCase {
    pub fn new(repository: Arc<dyn ArgoCdRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, name: &str) -> Result<()> {
        self.repository.sync_application(name).await
            .map_err(|e| KusanagiError::internal(format!("Failed to sync application: {}", e)))
    }
}

/// Get application details use case
pub struct GetApplicationDetailsUseCase {
    repository: Arc<dyn ArgoCdRepository>,
}

impl GetApplicationDetailsUseCase {
    pub fn new(repository: Arc<dyn ArgoCdRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, name: &str) -> Result<ApplicationDetails> {
        self.repository.get_application_details(name).await
            .map_err(|_e| KusanagiError::not_found("Application", name))
    }
}

/// ArgoCD service - aggregates all ArgoCD use cases
pub struct ArgoCdService {
    pub list_applications: GetArgoCdApplicationsUseCase,
    pub get_status: GetApplicationStatusUseCase,
    pub sync: SyncApplicationUseCase,
    pub get_details: GetApplicationDetailsUseCase,
}

impl ArgoCdService {
    pub fn new(repository: Arc<dyn ArgoCdRepository>) -> Self {
        Self {
            list_applications: GetArgoCdApplicationsUseCase::new(repository.clone()),
            get_status: GetApplicationStatusUseCase::new(repository.clone()),
            sync: SyncApplicationUseCase::new(repository.clone()),
            get_details: GetApplicationDetailsUseCase::new(repository),
        }
    }
}
