//! Pod Use Cases
//!
//! Application layer use cases for pod operations.

use crate::domain::entities::{Pod, PodsStatus};
use crate::domain::ports::KubernetesRepository;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// List pods use case
pub struct ListPodsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl ListPodsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<Vec<Pod>> {
        self.repository.list_pods(namespace).await
    }
}

/// Get pod details use case
pub struct GetPodDetailsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetPodDetailsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: &str, name: &str) -> Result<Pod> {
        self.repository.get_pod(namespace, name).await
    }
}

/// Get pods status use case
pub struct GetPodsStatusUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetPodsStatusUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<PodsStatus> {
        self.repository.get_pods_status().await
            .map_err(|e| KusanagiError::internal(format!("Failed to get pods status: {}", e)))
    }
}

/// Get pod logs use case
pub struct GetPodLogsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl GetPodLogsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        namespace: &str,
        name: &str,
        container: Option<String>,
        tail_lines: i64,
    ) -> Result<String> {
        self.repository
            .get_pod_logs(namespace, name, container.as_deref(), tail_lines)
            .await
            .map_err(|e| KusanagiError::internal(format!("Failed to get pod logs: {}", e)))
    }
}

/// Force delete pod use case
pub struct ForceDeletePodUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl ForceDeletePodUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: &str, name: &str) -> Result<()> {
        // First check if pod exists
        let _ = self.repository.get_pod(namespace, name).await
            .map_err(|_| KusanagiError::not_found("Pod", name))?;

        self.repository.force_delete_pod(namespace, name)
            .await
            .map_err(|e| KusanagiError::internal(format!("Failed to force delete pod: {}", e)))
    }
}

/// Delete error pods use case
pub struct DeleteErrorPodsUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl DeleteErrorPodsUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<(usize, usize)> {
        self.repository.delete_error_pods()
            .await
            .map_err(|e| KusanagiError::internal(format!("Failed to delete error pods: {}", e)))
    }
}

/// Scale deployment use case
pub struct ScaleDeploymentUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl ScaleDeploymentUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: &str, name: &str, replicas: i32) -> Result<()> {
        if replicas < 0 || replicas > 100 {
            return Err(KusanagiError::validation("Replicas must be between 0 and 100"));
        }

        self.repository.scale_deployment(namespace, name, replicas)
            .await
            .map_err(|e| KusanagiError::internal(format!("Failed to scale deployment: {}", e)))
    }
}

/// Scale statefulset use case
pub struct ScaleStatefulSetUseCase {
    repository: Arc<dyn KubernetesRepository>,
}

impl ScaleStatefulSetUseCase {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, namespace: &str, name: &str, replicas: i32) -> Result<()> {
        if replicas < 0 || replicas > 100 {
            return Err(KusanagiError::validation("Replicas must be between 0 and 100"));
        }

        self.repository.scale_statefulset(namespace, name, replicas)
            .await
            .map_err(|e| KusanagiError::internal(format!("Failed to scale statefulset: {}", e)))
    }
}

/// Pod service - aggregates all pod use cases
pub struct PodService {
    pub get_status: GetPodsStatusUseCase,
    pub get_logs: GetPodLogsUseCase,
    pub force_delete: ForceDeletePodUseCase,
    pub delete_error_pods: DeleteErrorPodsUseCase,
    pub scale_deployment: ScaleDeploymentUseCase,
    pub scale_statefulset: ScaleStatefulSetUseCase,
}

impl PodService {
    pub fn new(repository: Arc<dyn KubernetesRepository>) -> Self {
        Self {
            get_status: GetPodsStatusUseCase::new(repository.clone()),
            get_logs: GetPodLogsUseCase::new(repository.clone()),
            force_delete: ForceDeletePodUseCase::new(repository.clone()),
            delete_error_pods: DeleteErrorPodsUseCase::new(repository.clone()),
            scale_deployment: ScaleDeploymentUseCase::new(repository.clone()),
            scale_statefulset: ScaleStatefulSetUseCase::new(repository),
        }
    }
}
