use crate::domain::ports::KubernetesRepository;
use crate::error::Result;
use std::sync::Arc;

pub struct GetNodesUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetNodesUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self) -> Result<Vec<crate::domain::entities::Node>> {
        self.k8s_repo.list_nodes().await
    }
}

pub struct GetNodeDetailsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetNodeDetailsUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    pub async fn execute(&self, name: &str) -> Result<crate::domain::entities::Node> {
        self.k8s_repo.get_node(name).await
    }
}
