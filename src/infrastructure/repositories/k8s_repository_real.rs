use crate::error::Result;
use crate::domain::entities_simple::ClusterOverview;
use async_trait::async_trait;
use kube::{Api, Client};
use k8s_openapi::api::core::v1::{Node, Pod, Namespace};

#[async_trait]
pub trait KubernetesRepository {
    async fn get_cluster_overview(&self) -> Result<ClusterOverview>;
}

pub struct K8sRepository {
    client: Option<Client>,
    is_mock: bool,
}

impl K8sRepository {
    pub async fn new() -> Result<Self> {
        let is_k8s = std::env::var("KUBERNETES_SERVICE_HOST").is_ok();
        
        if is_k8s {
            match Client::try_default().await {
                Ok(client) => Ok(Self { 
                    client: Some(client), 
                    is_mock: false 
                }),
                Err(_) => {
                    println!("⚠️  Failed to connect to K8s API, falling back to mock mode");
                    Ok(Self { 
                        client: None, 
                        is_mock: true 
                    })
                }
            }
        } else {
            Ok(Self { 
                client: None, 
                is_mock: true 
            })
        }
    }
}

#[async_trait]
impl KubernetesRepository for K8sRepository {
    async fn get_cluster_overview(&self) -> Result<ClusterOverview> {
        if self.is_mock || self.client.is_none() {
            return Ok(ClusterOverview {
                cluster_name: "local-mock".to_string(),
                node_count: 1,
                pod_count: 5,
                namespace_count: 3,
                healthy_nodes: 1,
                running_pods: 5,
                status: "Healthy (Mock Data)".to_string(),
            });
        }

        let client = self.client.as_ref().unwrap();
        
        // Get nodes
        let nodes: Api<Node> = Api::all(client.clone());
        let node_list = nodes.list(&Default::default()).await
            .map_err(|e| crate::error::KusanagiError::k8s(format!("Failed to list nodes: {}", e)))?;
        
        let node_count = node_list.items.len() as i32;
        let healthy_nodes = node_list.items.iter()
            .filter(|node| {
                node.status.as_ref()
                    .and_then(|s| s.conditions.as_ref())
                    .map(|conditions| {
                        conditions.iter().any(|c| c.type_ == "Ready" && c.status == "True")
                    })
                    .unwrap_or(false)
            })
            .count() as i32;

        // Get pods
        let pods: Api<Pod> = Api::all(client.clone());
        let pod_list = pods.list(&Default::default()).await
            .map_err(|e| crate::error::KusanagiError::k8s(format!("Failed to list pods: {}", e)))?;
        
        let pod_count = pod_list.items.len() as i32;
        let running_pods = pod_list.items.iter()
            .filter(|pod| {
                pod.status.as_ref()
                    .and_then(|s| s.phase.as_ref())
                    .map(|phase| phase == "Running")
                    .unwrap_or(false)
            })
            .count() as i32;

        // Get namespaces
        let namespaces: Api<Namespace> = Api::all(client.clone());
        let namespace_list = namespaces.list(&Default::default()).await
            .map_err(|e| crate::error::KusanagiError::k8s(format!("Failed to list namespaces: {}", e)))?;
        
        let namespace_count = namespace_list.items.len() as i32;

        let status = if healthy_nodes == node_count && running_pods > 0 {
            "Healthy".to_string()
        } else if healthy_nodes > 0 {
            "Degraded".to_string()
        } else {
            "Unhealthy".to_string()
        };

        Ok(ClusterOverview {
            cluster_name: "kubernetes".to_string(),
            node_count,
            pod_count,
            namespace_count,
            healthy_nodes,
            running_pods,
            status,
        })
    }
}
