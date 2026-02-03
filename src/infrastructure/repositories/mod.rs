//! Repository implementations
//!
//! These are concrete implementations of the driven ports defined in the domain layer.

use crate::domain::entities::*;
use crate::domain::ports::*;
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
use kube::api::ListParams;
use kube::{Api, Client};
use std::sync::Arc;

mod argocd_repository;
pub use argocd_repository::ArgoCdRepositoryImpl;

/// Kubernetes repository implementation using kube-rs
pub struct K8sRepository {
    client: Client,
}

impl K8sRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn from_arc(client: Arc<Client>) -> Self {
        // This is a workaround - in practice, you'd want to store Arc<Client>
        Self {
            client: Arc::try_unwrap(client).unwrap_or_else(|arc| (*arc).clone()),
        }
    }
}

#[async_trait]
impl KubernetesRepository for K8sRepository {
    async fn get_cluster_overview(&self) -> Result<ClusterOverview> {
        // Implementation using kube-rs
        let nodes: Api<k8s_openapi::api::core::v1::Node> = Api::all(self.client.clone());
        let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::all(self.client.clone());
        let namespaces: Api<k8s_openapi::api::core::v1::Namespace> = Api::all(self.client.clone());

        let node_list = nodes.list(&ListParams::default()).await?;
        let pod_list = pods.list(&ListParams::default()).await?;
        let ns_list = namespaces.list(&ListParams::default()).await?;

        let ready_nodes = node_list
            .items
            .iter()
            .filter(|n| {
                n.status.as_ref().map_or(false, |s| {
                    s.conditions.as_ref().map_or(false, |conds| {
                        conds.iter().any(|c| {
                            c.type_ == "Ready" && c.status == "True"
                        })
                    })
                })
            })
            .count();

        Ok(ClusterOverview {
            name: "kubernetes".to_string(),
            version: "v1.28".to_string(), // Would get from actual cluster info
            node_count: node_list.items.len(),
            pod_count: pod_list.items.len(),
            namespace_count: ns_list.items.len(),
            status: if ready_nodes == node_list.items.len() {
                ClusterStatus::Healthy
            } else {
                ClusterStatus::Degraded
            },
            resources: ClusterResources::default(),
            health: ClusterHealth {
                nodes_not_ready: (node_list.items.len() - ready_nodes) as i32,
                pods_crashing: 0, // Would calculate from actual pod statuses
                ..Default::default()
            },
        })
    }

    async fn list_nodes(&self) -> Result<Vec<Node>> {
        let nodes: Api<k8s_openapi::api::core::v1::Node> = Api::all(self.client.clone());
        let node_list = nodes.list(&ListParams::default()).await?;

        let result: Vec<Node> = node_list
            .items
            .into_iter()
            .map(|n| {
                let status = n.status.as_ref();
                let conditions = status.and_then(|s| s.conditions.as_ref());
                let node_status = conditions.map_or(NodeStatus::Unknown, |conds| {
                    if conds.iter().any(|c| c.type_ == "Ready" && c.status == "True") {
                        NodeStatus::Ready
                    } else {
                        NodeStatus::NotReady
                    }
                });

                let allocatable = status.and_then(|s| s.allocatable.clone()).unwrap_or_default();
                let capacity = status.and_then(|s| s.capacity.clone()).unwrap_or_default();
                let node_info = status.and_then(|s| s.node_info.clone());

                // Extract disk info
                let ephemeral_storage_capacity = capacity.get("ephemeral-storage").map(|q| q.0.clone());
                let ephemeral_storage_allocatable = allocatable.get("ephemeral-storage").map(|q| q.0.clone());
                
                Node {
                    name: n.metadata.name.unwrap_or_default(),
                    status: node_status,
                    role: NodeRole::Worker, // Would determine from labels
                    resources: NodeResources {
                        cpu_capacity: capacity.get("cpu").map_or("0".to_string(), |q| q.0.clone()),
                        cpu_allocatable: allocatable.get("cpu").map_or("0".to_string(), |q| q.0.clone()),
                        memory_capacity: capacity.get("memory").map_or("0".to_string(), |q| q.0.clone()),
                        memory_allocatable: allocatable.get("memory").map_or("0".to_string(), |q| q.0.clone()),
                        pod_capacity: capacity.get("pods").and_then(|q| q.0.parse().ok()).unwrap_or(0),
                        pod_count: 0, // Would need to count from pods
                        disk_capacity: ephemeral_storage_capacity.clone(),
                        disk_allocatable: ephemeral_storage_allocatable.clone(),
                        disk_usage_percent: None, // Will be populated from Prometheus metrics
                        ephemeral_storage_capacity,
                        ephemeral_storage_allocatable,
                    },
                    info: NodeInfo {
                        architecture: node_info.as_ref().map_or("unknown".to_string(), |i| i.architecture.clone()),
                        os: node_info.as_ref().map_or("unknown".to_string(), |i| i.os_image.clone()),
                        kernel_version: node_info.as_ref().map_or("unknown".to_string(), |i| i.kernel_version.clone()),
                        kubelet_version: node_info.as_ref().map_or("unknown".to_string(), |i| i.kubelet_version.clone()),
                        container_runtime: node_info.as_ref().map_or("unknown".to_string(), |i| i.container_runtime_version.clone()),
                    },
                    conditions: conditions.map_or(vec![], |conds| {
                        conds.iter().map(|c| NodeCondition {
                            condition_type: c.type_.clone(),
                            status: c.status.clone(),
                            last_transition: c.last_transition_time.as_ref().map(|t| t.0),
                            reason: c.reason.clone().unwrap_or_default(),
                            message: c.message.clone().unwrap_or_default(),
                        }).collect()
                    }),
                }
            })
            .collect();

        Ok(result)
    }

    async fn get_node(&self, name: &str) -> Result<Node> {
        let nodes: Api<k8s_openapi::api::core::v1::Node> = Api::all(self.client.clone());
        let _node = nodes.get(name).await?;

        // Convert to domain entity (simplified)
        Ok(Node {
            name: name.to_string(),
            status: NodeStatus::Ready,
            role: NodeRole::Worker,
            resources: NodeResources::default(),
            info: NodeInfo::default(),
            conditions: vec![],
        })
    }

    async fn list_pods(&self, namespace: Option<&str>) -> Result<Vec<Pod>> {
        let pods = if let Some(ns) = namespace {
            let api: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(self.client.clone(), ns);
            api.list(&ListParams::default()).await?
        } else {
            let api: Api<k8s_openapi::api::core::v1::Pod> = Api::all(self.client.clone());
            api.list(&ListParams::default()).await?
        };

        let result: Vec<Pod> = pods
            .items
            .into_iter()
            .map(|p| {
                let status = p.status.as_ref();
                let container_statuses = status.and_then(|s| s.container_statuses.clone()).unwrap_or_default();
                let spec = p.spec.as_ref();
                let containers = spec.map(|s| s.containers.clone()).unwrap_or_default();

                Pod {
                    name: p.metadata.name.unwrap_or_default(),
                    namespace: p.metadata.namespace.unwrap_or_default(),
                    status: match status.and_then(|s| s.phase.clone()) {
                        Some(phase) => match phase.as_str() {
                            "Running" => PodStatus::Running,
                            "Pending" => PodStatus::Pending,
                            "Succeeded" => PodStatus::Succeeded,
                            "Failed" => PodStatus::Failed,
                            _ => PodStatus::Unknown,
                        },
                        None => PodStatus::Unknown,
                    },
                    containers: containers.iter().map(|c| Container {
                        name: c.name.clone(),
                        image: c.image.clone().unwrap_or_default(),
                        ready: container_statuses.iter().any(|cs| cs.name == c.name && cs.ready),
                        restart_count: container_statuses.iter().find(|cs| cs.name == c.name).map_or(0, |cs| cs.restart_count),
                        state: ContainerState::Running, // Simplified
                    }).collect(),
                    node_name: spec.and_then(|s| s.node_name.clone()),
                    restart_count: container_statuses.iter().map(|cs| cs.restart_count).sum(),
                    age: p.metadata.creation_timestamp.as_ref().map(|t| format!("{:?}", chrono::Utc::now() - t.0)),
                    age_seconds: p.metadata.creation_timestamp.as_ref().map_or(0, |t| (chrono::Utc::now() - t.0).num_seconds()),
                    labels: p.metadata.labels.unwrap_or_default().into_iter().collect(),
                    reason: status.and_then(|s| s.reason.clone()),
                    message: status.and_then(|s| s.message.clone()),
                    cpu_usage: None,
                    memory_usage: None,
                    cpu_limit: None,
                    memory_limit: None,
                    cpu_request: None,
                    memory_request: None,
                }
            })
            .collect();

        Ok(result)
    }

    async fn get_pod(&self, namespace: &str, name: &str) -> Result<Pod> {
        let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(self.client.clone(), namespace);
        let _pod = pods.get(name).await?;

        // Convert to domain entity (simplified)
        Ok(Pod {
            name: name.to_string(),
            namespace: namespace.to_string(),
            status: PodStatus::Running,
            containers: vec![],
            node_name: None,
            restart_count: 0,
            age: None,
            age_seconds: 0,
            labels: Default::default(),
            reason: None,
            message: None,
            cpu_usage: None,
            memory_usage: None,
            cpu_limit: None,
            memory_limit: None,
            cpu_request: None,
            memory_request: None,
        })
    }

    async fn get_pod_logs(&self, namespace: &str, name: &str, container: Option<&str>, tail: i64) -> Result<String> {
        let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(self.client.clone(), namespace);
        
        let params = kube::api::LogParams {
            container: container.map(|c| c.to_string()),
            tail_lines: Some(tail),
            ..Default::default()
        };

        let logs = pods.logs(name, &params).await?;
        Ok(logs)
    }

    async fn delete_pod(&self, namespace: &str, name: &str) -> Result<()> {
        let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(self.client.clone(), namespace);
        pods.delete(name, &kube::api::DeleteParams::default()).await?;
        Ok(())
    }

    async fn list_events(&self, namespace: Option<&str>, event_type: Option<&str>) -> Result<Vec<ClusterEvent>> {
        let events = if let Some(ns) = namespace {
            let api: Api<k8s_openapi::api::core::v1::Event> = Api::namespaced(self.client.clone(), ns);
            api.list(&ListParams::default()).await?
        } else {
            let api: Api<k8s_openapi::api::core::v1::Event> = Api::all(self.client.clone());
            api.list(&ListParams::default()).await?
        };

        let result: Vec<ClusterEvent> = events
            .items
            .into_iter()
            .filter(|e| {
                event_type.as_ref().map_or(true, |et| {
                    e.type_.as_ref().map_or(false, |t| t.eq_ignore_ascii_case(et))
                })
            })
            .map(|e| ClusterEvent {
                name: e.metadata.name.unwrap_or_default(),
                namespace: e.metadata.namespace.unwrap_or_default(),
                event_type: match e.type_.as_deref() {
                    Some("Warning") => EventType::Warning,
                    _ => EventType::Normal,
                },
                reason: e.reason.clone().unwrap_or_default(),
                message: e.message.clone().unwrap_or_default(),
                involved_object: InvolvedObject {
                    kind: e.involved_object.kind.clone().unwrap_or_default(),
                    name: e.involved_object.name.clone().unwrap_or_default(),
                },
                count: e.count.unwrap_or(1),
                first_timestamp: e.first_timestamp.as_ref().map(|t| t.0),
                last_timestamp: e.last_timestamp.as_ref().map(|t| t.0),
                age: e.last_timestamp.as_ref().map(|t| format!("{:?}", chrono::Utc::now() - t.0)),
            })
            .collect();

        Ok(result)
    }

    async fn list_services(&self, namespace: Option<&str>) -> Result<Vec<Service>> {
        let services = if let Some(ns) = namespace {
            let api: Api<k8s_openapi::api::core::v1::Service> = Api::namespaced(self.client.clone(), ns);
            api.list(&ListParams::default()).await?
        } else {
            let api: Api<k8s_openapi::api::core::v1::Service> = Api::all(self.client.clone());
            api.list(&ListParams::default()).await?
        };

        let result: Vec<Service> = services
            .items
            .into_iter()
            .map(|s| {
                let spec = s.spec.as_ref();
                Service {
                    name: s.metadata.name.unwrap_or_default(),
                    namespace: s.metadata.namespace.unwrap_or_default(),
                    service_type: spec.and_then(|sp| sp.type_.clone()).unwrap_or_default(),
                    cluster_ip: spec.and_then(|sp| sp.cluster_ip.clone()).unwrap_or_default(),
                    external_ips: spec.and_then(|sp| sp.external_ips.clone()).unwrap_or_default(),
                    ports: spec.and_then(|sp| sp.ports.clone()).map_or(vec![], |ports| {
                        ports.iter().map(|p| ServicePort {
                            name: p.name.clone().unwrap_or_default(),
                            port: p.port,
                            target_port: p.target_port.as_ref().map_or("".to_string(), |t| format!("{:?}", t)),
                            protocol: p.protocol.clone().unwrap_or_else(|| "TCP".to_string()),
                        }).collect()
                    }),
                    selector: spec.and_then(|sp| sp.selector.clone()).map(|s| s.into_iter().collect()).unwrap_or_default(),
                    age: s.metadata.creation_timestamp.as_ref().map_or("".to_string(), |t| format!("{:?}", chrono::Utc::now() - t.0)),
                }
            })
            .collect();

        Ok(result)
    }

    async fn list_ingresses(&self, namespace: Option<&str>) -> Result<Vec<Ingress>> {
        let ingresses = if let Some(ns) = namespace {
            let api: Api<k8s_openapi::api::networking::v1::Ingress> = Api::namespaced(self.client.clone(), ns);
            api.list(&ListParams::default()).await?
        } else {
            let api: Api<k8s_openapi::api::networking::v1::Ingress> = Api::all(self.client.clone());
            api.list(&ListParams::default()).await?
        };

        let result: Vec<Ingress> = ingresses
            .items
            .into_iter()
            .map(|i| {
                let spec = i.spec.as_ref();
                Ingress {
                    name: i.metadata.name.unwrap_or_default(),
                    namespace: i.metadata.namespace.unwrap_or_default(),
                    hosts: spec.and_then(|s| s.rules.clone()).map_or(vec![], |rules| {
                        rules.iter().filter_map(|r| r.host.clone()).collect()
                    }),
                    paths: vec![], // Simplified
                    tls: spec.and_then(|s| s.tls.clone()).map_or(vec![], |tls| {
                        tls.iter().filter_map(|t| t.secret_name.clone()).collect()
                    }),
                    age: i.metadata.creation_timestamp.as_ref().map_or("".to_string(), |t| format!("{:?}", chrono::Utc::now() - t.0)),
                }
            })
            .collect();

        Ok(result)
    }

    async fn list_namespaces(&self) -> Result<Vec<Namespace>> {
        let namespaces: Api<k8s_openapi::api::core::v1::Namespace> = Api::all(self.client.clone());
        let ns_list = namespaces.list(&ListParams::default()).await?;

        let result: Vec<Namespace> = ns_list
            .items
            .into_iter()
            .map(|ns| Namespace {
                name: ns.metadata.name.unwrap_or_default(),
                status: ns.status.as_ref().and_then(|s| s.phase.clone()).unwrap_or_default(),
                pod_count: 0, // Would need to count
                resource_quota: None, // Would need to fetch
                age: ns.metadata.creation_timestamp.as_ref().map_or("".to_string(), |t| format!("{:?}", chrono::Utc::now() - t.0)),
            })
            .collect();

        Ok(result)
    }

    async fn get_storage_info(&self) -> Result<StorageInfo> {
        let pvs: Api<k8s_openapi::api::core::v1::PersistentVolume> = Api::all(self.client.clone());
        let pv_list = pvs.list(&ListParams::default()).await?;

        let mut total_capacity = 0i64;
        let mut available = 0usize;
        let mut bound = 0usize;
        let mut released = 0usize;

        for pv in &pv_list.items {
            match pv.status.as_ref().and_then(|s| s.phase.as_deref()) {
                Some("Available") => available += 1,
                Some("Bound") => bound += 1,
                Some("Released") => released += 1,
                _ => {}
            }

            if let Some(cap) = pv.spec.as_ref().and_then(|s| s.capacity.as_ref()).and_then(|c| c.get("storage")) {
                // Parse capacity (simplified)
                let cap_str = &cap.0;
                if cap_str.ends_with("Gi") {
                    if let Ok(num) = cap_str.trim_end_matches("Gi").parse::<i64>() {
                        total_capacity += num * 1024 * 1024 * 1024;
                    }
                }
            }
        }

        Ok(StorageInfo {
            total_pvs: pv_list.items.len(),
            available_pvs: available,
            bound_pvs: bound,
            released_pvs: released,
            total_capacity: format!("{}Gi", total_capacity / (1024 * 1024 * 1024)),
            used_capacity: format!("{}Gi", bound as i64 * 10), // Simplified
        })
    }
    
    async fn force_delete_pod(&self, namespace: &str, name: &str) -> Result<()> {
        let pods: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(self.client.clone(), namespace);
        
        // Remove finalizers
        let patch = serde_json::json!({
            "metadata": {
                "finalizers": null
            }
        });
        
        let _ = pods.patch(name, &kube::api::PatchParams::default(), &kube::api::Patch::Merge(&patch)).await;
        
        // Delete with grace period 0
        let delete_params = kube::api::DeleteParams {
            grace_period_seconds: Some(0),
            ..Default::default()
        };
        
        pods.delete(name, &delete_params).await?;
        Ok(())
    }
    
    async fn get_pods_status(&self) -> Result<crate::domain::entities::PodsStatus> {
        // Simplified implementation
        Ok(crate::domain::entities::PodsStatus {
            total_pods: 0,
            running_pods: 0,
            pending_pods: 0,
            succeeded_pods: 0,
            failed_pods: 0,
            error_pods: 0,
            pods_in_error: vec![],
            fetch_duration_ms: 0,
        })
    }
    
    async fn delete_error_pods(&self) -> Result<(usize, usize)> {
        Ok((0, 0))
    }
    
    async fn scale_deployment(&self, namespace: &str, name: &str, replicas: i32) -> Result<()> {
        let api: Api<k8s_openapi::api::apps::v1::Deployment> = Api::namespaced(self.client.clone(), namespace);
        let patch = serde_json::json!({
            "spec": {
                "replicas": replicas
            }
        });
        
        api.patch(name, &kube::api::PatchParams::default(), &kube::api::Patch::Merge(&patch)).await?;
        Ok(())
    }
    
    async fn scale_statefulset(&self, namespace: &str, name: &str, replicas: i32) -> Result<()> {
        let api: Api<k8s_openapi::api::apps::v1::StatefulSet> = Api::namespaced(self.client.clone(), namespace);
        let patch = serde_json::json!({
            "spec": {
                "replicas": replicas
            }
        });
        
        api.patch(name, &kube::api::PatchParams::default(), &kube::api::Patch::Merge(&patch)).await?;
        Ok(())
    }
}

/// Prometheus repository implementation
pub struct PrometheusRepository {
    client: reqwest::Client,
    base_url: String,
}

impl PrometheusRepository {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.into(),
        }
    }

    pub fn from_config() -> Self {
        Self::new(crate::config::get().prometheus.url.clone())
    }
}

#[async_trait]
impl MetricsRepository for PrometheusRepository {
    async fn query(&self, query: &str) -> Result<f64> {
        let url = format!("{}/api/v1/query", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[("query", query)])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KusanagiError::prometheus(format!(
                "Prometheus returned status: {}",
                response.status()
            )));
        }

        let result: serde_json::Value = response.json().await?;
        
        // Extract value from response
        let value = result
            .get("data")
            .and_then(|d| d.get("result"))
            .and_then(|r| r.get(0))
            .and_then(|res| res.get("value"))
            .and_then(|v| v.get(1))
            .and_then(|v| v.as_str())
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        Ok(value)
    }

    async fn query_raw(&self, query: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/query", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[("query", query)])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KusanagiError::prometheus(format!(
                "Prometheus returned status: {}",
                response.status()
            )));
        }

        Ok(response.json().await?)
    }

    async fn query_range(&self, query: &str, start: i64, end: i64, step: &str) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/query_range", self.base_url);
        
        let response = self.client
            .get(&url)
            .query(&[
                ("query", query),
                ("start", &start.to_string()),
                ("end", &end.to_string()),
                ("step", step),
            ])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KusanagiError::prometheus(format!(
                "Prometheus returned status: {}",
                response.status()
            )));
        }

        Ok(response.json().await?)
    }

    async fn get_cluster_metrics(&self) -> Result<ClusterResources> {
        // Query CPU usage
        let cpu_query = r#"100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)"#;
        let cpu_percent = self.query(cpu_query).await.unwrap_or(0.0);

        // Query memory usage
        let mem_query = r#"(1 - (sum(node_memory_MemAvailable_bytes) / sum(node_memory_MemTotal_bytes))) * 100"#;
        let memory_percent = self.query(mem_query).await.unwrap_or(0.0);

        Ok(ClusterResources {
            cpu_percent,
            memory_percent,
            storage_percent: 0.0,
            pod_capacity: 0,
            pod_used: 0,
        })
    }

    async fn get_pod_resource_usage(&self) -> Result<std::collections::HashMap<(String, String), (f64, i64)>> {
        let mut usage_map = std::collections::HashMap::new();

        // Query CPU usage by pod
        let cpu_query = r#"sum(rate(container_cpu_usage_seconds_total{container!="", image!=""}[5m])) by (namespace, pod)"#;
        if let Ok(result) = self.query_raw(cpu_query).await {
            if let Some(results) = result.get("data").and_then(|d| d.get("result")).and_then(|r| r.as_array()) {
                for r in results {
                    if let (Some(metric), Some(value)) = (r.get("metric"), r.get("value")) {
                        let namespace = metric.get("namespace").and_then(|s| s.as_str()).unwrap_or_default();
                        let pod = metric.get("pod").and_then(|s| s.as_str()).unwrap_or_default();
                        
                        if let Some(val_str) = value.get(1).and_then(|v| v.as_str()) {
                            if let Ok(val) = val_str.parse::<f64>() {
                                usage_map.insert((namespace.to_string(), pod.to_string()), (val, 0));
                            }
                        }
                    }
                }
            }
        }

        // Query memory usage by pod
        let mem_query = r#"sum(container_memory_usage_bytes{container!="", image!=""}) by (namespace, pod)"#;
        if let Ok(result) = self.query_raw(mem_query).await {
            if let Some(results) = result.get("data").and_then(|d| d.get("result")).and_then(|r| r.as_array()) {
                for r in results {
                    if let (Some(metric), Some(value)) = (r.get("metric"), r.get("value")) {
                        let namespace = metric.get("namespace").and_then(|s| s.as_str()).unwrap_or_default();
                        let pod = metric.get("pod").and_then(|s| s.as_str()).unwrap_or_default();
                        
                        if let Some(val_str) = value.get(1).and_then(|v| v.as_str()) {
                            if let Ok(val) = val_str.parse::<f64>() {
                                let mem_bytes = val as i64;
                                usage_map.entry((namespace.to_string(), pod.to_string()))
                                    .and_modify(|e| e.1 = mem_bytes)
                                    .or_insert((0.0, mem_bytes));
                            }
                        }
                    }
                }
            }
        }

        Ok(usage_map)
    }
}
