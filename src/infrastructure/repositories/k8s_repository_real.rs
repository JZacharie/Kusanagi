use crate::error::Result;
use crate::domain::entities_simple::{ClusterOverview, NodeInfo, PodInfo, EventInfo};
use async_trait::async_trait;
use kube::{Api, Client};
use k8s_openapi::api::core::v1::{Node, Pod, Namespace, Event};
use chrono::{DateTime, Utc};

#[async_trait]
pub trait KubernetesRepository {
    async fn get_cluster_overview(&self) -> Result<ClusterOverview>;
    async fn get_nodes(&self) -> Result<Vec<NodeInfo>>;
    async fn get_pods(&self, namespace: Option<String>) -> Result<Vec<PodInfo>>;
    async fn get_events(&self, namespace: Option<String>) -> Result<Vec<EventInfo>>;
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

    fn format_age(created: Option<&DateTime<Utc>>) -> String {
        match created {
            Some(time) => {
                let duration = Utc::now().signed_duration_since(*time);
                if duration.num_days() > 0 {
                    format!("{}d", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{}h", duration.num_hours())
                } else {
                    format!("{}m", duration.num_minutes())
                }
            }
            None => "unknown".to_string(),
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

    async fn get_nodes(&self) -> Result<Vec<NodeInfo>> {
        if self.is_mock || self.client.is_none() {
            return Ok(vec![
                NodeInfo {
                    name: "node-1".to_string(),
                    status: "Ready".to_string(),
                    roles: vec!["control-plane".to_string()],
                    age: "5d".to_string(),
                    version: "v1.28.0".to_string(),
                    cpu_usage: Some(42.1),
                    memory_usage: Some(65.3),
                },
                NodeInfo {
                    name: "node-2".to_string(),
                    status: "Ready".to_string(),
                    roles: vec!["worker".to_string()],
                    age: "5d".to_string(),
                    version: "v1.28.0".to_string(),
                    cpu_usage: Some(38.7),
                    memory_usage: Some(58.9),
                },
            ]);
        }

        let client = self.client.as_ref().unwrap();
        let nodes: Api<Node> = Api::all(client.clone());
        let node_list = nodes.list(&Default::default()).await
            .map_err(|e| crate::error::KusanagiError::k8s(format!("Failed to list nodes: {}", e)))?;

        let mut node_infos = Vec::new();
        for node in node_list.items {
            let name = node.metadata.name.unwrap_or_else(|| "unknown".to_string());
            
            let status = node.status.as_ref()
                .and_then(|s| s.conditions.as_ref())
                .and_then(|conditions| {
                    conditions.iter()
                        .find(|c| c.type_ == "Ready")
                        .map(|c| if c.status == "True" { "Ready" } else { "NotReady" })
                })
                .unwrap_or("Unknown")
                .to_string();

            let roles = node.metadata.labels.as_ref()
                .map(|labels| {
                    labels.keys()
                        .filter(|k| k.starts_with("node-role.kubernetes.io/"))
                        .map(|k| k.strip_prefix("node-role.kubernetes.io/").unwrap_or(k).to_string())
                        .collect()
                })
                .unwrap_or_else(|| vec!["worker".to_string()]);

            let version = node.status.as_ref()
                .and_then(|s| s.node_info.as_ref())
                .map(|info| info.kubelet_version.clone())
                .unwrap_or_else(|| "unknown".to_string());

            let age = Self::format_age(node.metadata.creation_timestamp.as_ref());

            node_infos.push(NodeInfo {
                name,
                status,
                roles,
                age,
                version,
                cpu_usage: None, // Will be filled by Prometheus
                memory_usage: None,
            });
        }

        Ok(node_infos)
    }

    async fn get_pods(&self, namespace: Option<String>) -> Result<Vec<PodInfo>> {
        if self.is_mock || self.client.is_none() {
            return Ok(vec![
                PodInfo {
                    name: "app-1-7d8f9b5c6d-xyz12".to_string(),
                    namespace: "default".to_string(),
                    status: "Running".to_string(),
                    node: Some("node-1".to_string()),
                    age: "2h".to_string(),
                    restarts: 0,
                    cpu_usage: Some(15.2),
                    memory_usage: Some(234.5),
                },
                PodInfo {
                    name: "coredns-558bd4d5db-abc34".to_string(),
                    namespace: "kube-system".to_string(),
                    status: "Running".to_string(),
                    node: Some("node-1".to_string()),
                    age: "5d".to_string(),
                    restarts: 1,
                    cpu_usage: Some(2.1),
                    memory_usage: Some(45.8),
                },
            ]);
        }

        let client = self.client.as_ref().unwrap();
        let pods: Api<Pod> = match namespace {
            Some(ns) => Api::namespaced(client.clone(), &ns),
            None => Api::all(client.clone()),
        };

        let pod_list = pods.list(&Default::default()).await
            .map_err(|e| crate::error::KusanagiError::k8s(format!("Failed to list pods: {}", e)))?;

        let mut pod_infos = Vec::new();
        for pod in pod_list.items {
            let name = pod.metadata.name.unwrap_or_else(|| "unknown".to_string());
            let namespace = pod.metadata.namespace.unwrap_or_else(|| "default".to_string());
            
            let status = pod.status.as_ref()
                .and_then(|s| s.phase.as_ref())
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string());

            let node = pod.spec.as_ref()
                .and_then(|s| s.node_name.clone());

            let age = Self::format_age(pod.metadata.creation_timestamp.as_ref());

            let restarts = pod.status.as_ref()
                .and_then(|s| s.container_statuses.as_ref())
                .map(|containers| {
                    containers.iter().map(|c| c.restart_count).sum()
                })
                .unwrap_or(0);

            pod_infos.push(PodInfo {
                name,
                namespace,
                status,
                node,
                age,
                restarts,
                cpu_usage: None, // Will be filled by Prometheus
                memory_usage: None,
            });
        }

        Ok(pod_infos)
    }

    async fn get_events(&self, namespace: Option<String>) -> Result<Vec<EventInfo>> {
        if self.is_mock || self.client.is_none() {
            return Ok(vec![
                EventInfo {
                    name: "pod-created".to_string(),
                    namespace: Some("default".to_string()),
                    reason: "Created".to_string(),
                    message: "Pod app-1-7d8f9b5c6d-xyz12 created".to_string(),
                    type_: "Normal".to_string(),
                    age: "2h".to_string(),
                    object: "Pod/app-1-7d8f9b5c6d-xyz12".to_string(),
                },
                EventInfo {
                    name: "node-ready".to_string(),
                    namespace: None,
                    reason: "NodeReady".to_string(),
                    message: "Node node-1 status is now: NodeReady".to_string(),
                    type_: "Normal".to_string(),
                    age: "5d".to_string(),
                    object: "Node/node-1".to_string(),
                },
            ]);
        }

        let client = self.client.as_ref().unwrap();
        let events: Api<Event> = match namespace {
            Some(ns) => Api::namespaced(client.clone(), &ns),
            None => Api::all(client.clone()),
        };

        let event_list = events.list(&Default::default()).await
            .map_err(|e| crate::error::KusanagiError::k8s(format!("Failed to list events: {}", e)))?;

        let mut event_infos = Vec::new();
        for event in event_list.items.into_iter().take(50) { // Limit to 50 events
            let name = event.metadata.name.unwrap_or_else(|| "unknown".to_string());
            let namespace = event.metadata.namespace;
            let reason = event.reason.unwrap_or_else(|| "Unknown".to_string());
            let message = event.message.unwrap_or_else(|| "No message".to_string());
            let type_ = event.type_.unwrap_or_else(|| "Normal".to_string());
            let age = Self::format_age(event.first_timestamp.as_ref().or(event.event_time.as_ref()));
            
            let object = event.involved_object.as_ref()
                .map(|obj| format!("{}/{}", obj.kind.as_deref().unwrap_or("Unknown"), obj.name.as_deref().unwrap_or("unknown")))
                .unwrap_or_else(|| "Unknown".to_string());

            event_infos.push(EventInfo {
                name,
                namespace,
                reason,
                message,
                type_,
                age,
                object,
            });
        }

        Ok(event_infos)
    }
}
