use crate::domain::ports::KubernetesRepository;
use crate::error::Result;
use std::sync::Arc;

pub struct GetPodsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetPodsUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<Vec<crate::domain::entities::Pod>> {
        self.k8s_repo.list_pods(namespace).await
    }
}

pub struct ScalePodUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl ScalePodUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn scale_deployment(&self, namespace: &str, name: &str, replicas: i32) -> Result<()> {
        self.k8s_repo.scale_deployment(namespace, name, replicas).await
    }

    pub async fn scale_statefulset(&self, namespace: &str, name: &str, replicas: i32) -> Result<()> {
        self.k8s_repo.scale_statefulset(namespace, name, replicas).await
    }
}

pub struct DeletePodUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl DeletePodUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn delete(&self, namespace: &str, name: &str) -> Result<()> {
        self.k8s_repo.delete_pod(namespace, name).await
    }

    pub async fn force_delete(&self, namespace: &str, name: &str) -> Result<()> {
        self.k8s_repo.force_delete_pod(namespace, name).await
    }
}
