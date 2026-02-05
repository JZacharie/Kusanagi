use crate::error::Result;
use crate::domain::entities::ClusterOverview;
use async_trait::async_trait;

#[async_trait]
pub trait KubernetesRepository {
    async fn get_cluster_overview(&self) -> Result<ClusterOverview>;
}

pub struct K8sRepository {
    is_mock: bool,
}

impl K8sRepository {
    pub async fn new() -> Result<Self> {
        let is_mock = std::env::var("KUBERNETES_SERVICE_HOST").is_err();
        Ok(Self { is_mock })
    }
}

#[async_trait]
impl KubernetesRepository for K8sRepository {
    async fn get_cluster_overview(&self) -> Result<ClusterOverview> {
        if self.is_mock {
            // Return mock data for local development
            Ok(ClusterOverview {
                cluster_name: "local-mock".to_string(),
                node_count: 1,
                pod_count: 5,
                namespace_count: 3,
                healthy_nodes: 1,
                running_pods: 5,
                status: "Healthy".to_string(),
            })
        } else {
            // TODO: Implement real Kubernetes API calls
            Ok(ClusterOverview {
                cluster_name: "kubernetes".to_string(),
                node_count: 0,
                pod_count: 0,
                namespace_count: 0,
                healthy_nodes: 0,
                running_pods: 0,
                status: "Unknown".to_string(),
            })
        }
    }
}
