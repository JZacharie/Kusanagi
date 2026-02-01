//! Pod-specific use cases

use crate::application::dtos::*;
use crate::application::mappers::*;
use crate::domain::entities::*;
use crate::domain::ports::*;
use crate::error::Result;
use std::sync::Arc;

/// Use case: List pods with filtering
pub struct ListPodsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl ListPodsUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<Vec<PodDto>> {
        let pods = self.k8s_repo.list_pods(namespace).await?;
        Ok(PodMapper::to_dto_list(pods))
    }
}

/// Use case: Get pod details with diagnostics
pub struct GetPodDetailsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetPodDetailsUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self, namespace: &str, name: &str) -> Result<PodDetailsDto> {
        let pod = self.k8s_repo.get_pod(namespace, name).await?;
        let logs = self.k8s_repo.get_pod_logs(namespace, name, None, 100).await.ok();
        
        let age = pod.age.clone().unwrap_or_default();
        let containers = PodMapper::to_dto(pod.clone()).containers;
        
        Ok(PodDetailsDto {
            name: pod.name,
            namespace: pod.namespace,
            status: format!("{:?}", pod.status),
            node_name: pod.node_name,
            restart_count: pod.restart_count,
            age,
            containers,
            logs,
            events: vec![], // Would fetch from event repo
        })
    }
}

/// DTO for pod details
#[derive(Debug, Clone, serde::Serialize)]
pub struct PodDetailsDto {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub node_name: Option<String>,
    pub restart_count: i32,
    pub age: String,
    pub containers: Vec<ContainerDto>,
    pub logs: Option<String>,
    pub events: Vec<String>,
}

/// Use case: Restart pod (delete and let it recreate)
pub struct RestartPodUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl RestartPodUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self, namespace: &str, name: &str) -> Result<RestartResultDto> {
        // Verify pod exists
        let _pod = self.k8s_repo.get_pod(namespace, name).await?;
        
        // Delete the pod
        self.k8s_repo.delete_pod(namespace, name).await?;
        
        Ok(RestartResultDto {
            success: true,
            message: format!("Pod {}/{} scheduled for restart", namespace, name),
        })
    }
}

/// DTO for restart result
#[derive(Debug, Clone, serde::Serialize)]
pub struct RestartResultDto {
    pub success: bool,
    pub message: String,
}

/// Use case: Get pods with errors
pub struct GetErrorPodsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetErrorPodsUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<Vec<ErrorPodDto>> {
        let pods = self.k8s_repo.list_pods(namespace).await?;
        
        let error_pods: Vec<ErrorPodDto> = pods
            .into_iter()
            .filter(|p| p.status.is_error())
            .map(|p| ErrorPodDto {
                name: p.name,
                namespace: p.namespace,
                status: format!("{:?}", p.status),
                restart_count: p.restart_count,
            })
            .collect();
        
        Ok(error_pods)
    }
}

/// DTO for error pod
#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorPodDto {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub restart_count: i32,
}

/// Use case: Get pod resource usage
pub struct GetPodResourceUsageUseCase {
    metrics_repo: Arc<dyn MetricsRepository>,
}

impl GetPodResourceUsageUseCase {
    pub fn new(metrics_repo: Arc<dyn MetricsRepository>) -> Self {
        Self { metrics_repo }
    }

    pub async fn execute(&self) -> Result<Vec<PodResourceUsageDto>> {
        let usage = self.metrics_repo.get_pod_resource_usage().await?;
        
        let dtos: Vec<PodResourceUsageDto> = usage
            .into_iter()
            .map(|((namespace, name), (cpu, memory))| PodResourceUsageDto {
                namespace,
                name,
                cpu_cores: cpu,
                memory_bytes: memory,
            })
            .collect();
        
        Ok(dtos)
    }
}

/// DTO for pod resource usage
#[derive(Debug, Clone, serde::Serialize)]
pub struct PodResourceUsageDto {
    pub namespace: String,
    pub name: String,
    pub cpu_cores: f64,
    pub memory_bytes: i64,
}
