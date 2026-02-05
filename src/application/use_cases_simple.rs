use std::sync::Arc;
use crate::error::Result;
use crate::domain::entities_simple::ClusterOverview;
use crate::infrastructure::repositories::k8s_repository_simple::KubernetesRepository;

#[derive(Clone)]
pub struct GetClusterOverviewUseCase {
    k8s_repo: Arc<dyn KubernetesRepository + Send + Sync>,
}

impl GetClusterOverviewUseCase {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository + Send + Sync>) -> Self {
        Self { k8s_repo }
    }
    
    pub async fn execute(&self) -> Result<ClusterOverview> {
        self.k8s_repo.get_cluster_overview().await
    }
}
