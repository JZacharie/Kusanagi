//! Domain ports (interfaces)
//!
//! Ports define the interfaces through which the domain layer interacts with
//! the outside world. They are the boundaries of the hexagon.
//!
//! # Types of Ports
//!
//! - **Driven ports**: Interfaces that the domain requires from external systems
//!   (e.g., Repository, Cache, External APIs)
//! - **Driving ports**: Interfaces that external systems use to interact with the domain
//!   (e.g., Use case interfaces)

use async_trait::async_trait;
use crate::domain::entities::*;
use crate::error::Result;
use std::collections::HashMap;

/// Port for Kubernetes operations
///
/// This is a driven port that defines what the domain needs from a Kubernetes client.
#[async_trait]
pub trait KubernetesRepository: Send + Sync {
    /// Get cluster overview
    async fn get_cluster_overview(&self) -> Result<ClusterOverview>;

    /// Get all nodes
    async fn list_nodes(&self) -> Result<Vec<Node>>;

    /// Get a specific node
    async fn get_node(&self, name: &str) -> Result<Node>;

    /// Get all pods
    async fn list_pods(&self, namespace: Option<&str>) -> Result<Vec<Pod>>;

    /// Get a specific pod
    async fn get_pod(&self, namespace: &str, name: &str) -> Result<Pod>;

    /// Get pod logs
    async fn get_pod_logs(&self, namespace: &str, name: &str, container: Option<&str>, tail: i64) -> Result<String>;

    /// Delete a pod
    async fn delete_pod(&self, namespace: &str, name: &str) -> Result<()>;
    
    /// Force delete a pod (remove finalizers)
    async fn force_delete_pod(&self, namespace: &str, name: &str) -> Result<()>;
    
    /// Get pods status overview
    async fn get_pods_status(&self) -> Result<crate::domain::entities::PodsStatus>;
    
    /// Delete all pods in error state
    async fn delete_error_pods(&self) -> Result<(usize, usize)>;
    
    /// Scale a deployment
    async fn scale_deployment(&self, namespace: &str, name: &str, replicas: i32) -> Result<()>;
    
    /// Scale a statefulset
    async fn scale_statefulset(&self, namespace: &str, name: &str, replicas: i32) -> Result<()>;

    /// Get all events
    async fn list_events(&self, namespace: Option<&str>, event_type: Option<&str>) -> Result<Vec<ClusterEvent>>;

    /// Get all services
    async fn list_services(&self, namespace: Option<&str>) -> Result<Vec<Service>>;

    /// Get all ingresses
    async fn list_ingresses(&self, namespace: Option<&str>) -> Result<Vec<Ingress>>;

    /// Get all namespaces
    async fn list_namespaces(&self) -> Result<Vec<Namespace>>;

    /// Get storage information
    async fn get_storage_info(&self) -> Result<StorageInfo>;
}

/// Port for metrics collection
///
/// This is a driven port for Prometheus metrics.
#[async_trait]
pub trait MetricsRepository: Send + Sync {
    /// Query a metric
    async fn query(&self, query: &str) -> Result<f64>;

    /// Query raw metrics
    async fn query_raw(&self, query: &str) -> Result<serde_json::Value>;

    /// Query metrics over a time range
    async fn query_range(&self, query: &str, start: i64, end: i64, step: &str) -> Result<serde_json::Value>;

    /// Get cluster metrics
    async fn get_cluster_metrics(&self) -> Result<ClusterResources>;

    /// Get pod resource usage
    async fn get_pod_resource_usage(&self) -> Result<HashMap<(String, String), (f64, i64)>>;
}

/// Port for caching
///
/// This is a driven port for cache operations.
#[async_trait]
pub trait CachePort: Send + Sync {
    type Key: Clone + Send + Sync;
    type Value: Clone + Send + Sync;

    /// Get a value from cache
    async fn get(&self, key: &Self::Key) -> Option<Self::Value>;

    /// Set a value in cache with TTL
    async fn set(&self, key: Self::Key, value: Self::Value, ttl_secs: u64);

    /// Remove a value from cache
    async fn remove(&self, key: &Self::Key);

    /// Clear all values
    async fn clear(&self);
}

/// Port for external integrations
///
/// This is a driven port for external APIs like Slack, Home Assistant, etc.
#[async_trait]
pub trait IntegrationPort: Send + Sync {
    /// Get the integration name
    fn name(&self) -> &str;

    /// Check if the integration is healthy
    async fn health_check(&self) -> Result<bool>;

    /// Send a notification/message
    async fn send_notification(&self, message: &str) -> Result<()>;
}

/// Port for ArgoCD repository operations
pub mod argocd_port;
pub use argocd_port::{ArgoCdRepository, ApplicationStatus, ApplicationInfo, SyncStatus, HealthStatus, ApplicationDetails, ResourceStatus, RevisionHistory};

/// Port for Prometheus operations
pub mod prometheus_port;
pub use prometheus_port::{PrometheusRepository, PrometheusMetrics};

/// Port for Backup operations
pub mod backup_port;
pub use backup_port::BackupRepository;

/// Port for Security operations
pub mod security_port;
pub use security_port::{SecurityRepository, AiEnrichmentService, VulnerabilityScanner};

/// Port for Alert operations
pub mod alert_port;
pub use alert_port::AlertRepository;

/// Port for Chat operations
pub mod chat_port;
pub use chat_port::{ChatService, ChatHistoryRepository, AiProvider};

/// Port for Chat Repository operations
pub mod chat_repository;
pub use chat_repository::{ChatRepository, AiService, ChatMessage};

/// Port for Integration operations
pub mod integration_port;
pub use integration_port::IntegrationRepository;

/// Port for System operations
pub mod system_port;
pub use system_port::{SystemRepository, DatabaseRepository};

/// Port for Node Metrics operations
pub mod node_metrics_port;
pub use node_metrics_port::{NodeMetricsRepository, NodeDiskMetrics, ClusterDiskSummary};

/// Port for MCP operations
pub mod mcp_port;
pub use mcp_port::{McpRepository, K8sResourceSummary, CiliumPolicySummary, TrivyVulnerabilitySummary, SteampipeResult};

/// Port for Cilium operations
pub mod cilium_port;
pub use cilium_port::{CiliumRepository, NetworkFlow, NetworkPolicy, BandwidthMetrics, NetworkAnomaly};

/// Port for Proxmox operations
pub mod proxmox_port;
pub use proxmox_port::{ProxmoxRepository, ClusterStatus, ProxmoxVM, ProxmoxContainer};

/// Port for Newsfeed operations
pub mod newsfeed_port;
pub use newsfeed_port::{NewsfeedRepository, NewsItem};

/// Port for Cilium network operations
#[async_trait]
pub trait CiliumPort: Send + Sync {
    /// Get network flows
    async fn get_flows(&self, namespace: Option<&str>, limit: usize) -> Result<Vec<NetworkFlow>>;

    /// Get network policies
    async fn get_policies(&self, namespace: Option<&str>) -> Result<Vec<NetworkPolicy>>;
}

/// Network flow information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkFlow {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source_ip: String,
    pub destination_ip: String,
    pub source_port: u16,
    pub destination_port: u16,
    pub protocol: String,
    pub verdict: String,
}

/// Network policy information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicy {
    pub name: String,
    pub namespace: String,
    pub action: String,
    pub rules: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementation for testing
    struct MockKubernetesRepository;

    #[async_trait]
    impl KubernetesRepository for MockKubernetesRepository {
        async fn get_cluster_overview(&self) -> Result<ClusterOverview> {
            Ok(ClusterOverview::default())
        }

        async fn list_nodes(&self) -> Result<Vec<Node>> {
            Ok(vec![])
        }

        async fn get_node(&self, _name: &str) -> Result<Node> {
            Err(crate::error::KusanagiError::not_found("Node", "test"))
        }

        async fn list_pods(&self, _namespace: Option<&str>) -> Result<Vec<Pod>> {
            Ok(vec![])
        }

        async fn get_pod(&self, _namespace: &str, _name: &str) -> Result<Pod> {
            Err(crate::error::KusanagiError::not_found("Pod", "test"))
        }

        async fn get_pod_logs(&self, _namespace: &str, _name: &str, _container: Option<&str>, _tail: i64) -> Result<String> {
            Ok("log output".to_string())
        }

        async fn delete_pod(&self, _namespace: &str, _name: &str) -> Result<()> {
            Ok(())
        }

        async fn force_delete_pod(&self, _namespace: &str, _name: &str) -> Result<()> {
            Ok(())
        }

        async fn get_pods_status(&self) -> Result< PodsStatus> {
            Ok(PodsStatus {
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

        async fn scale_deployment(&self, _ns: &str, _name: &str, _replicas: i32) -> Result<()> {
            Ok(())
        }

        async fn scale_statefulset(&self, _ns: &str, _name: &str, _replicas: i32) -> Result<()> {
            Ok(())
        }

        async fn list_events(&self, _namespace: Option<&str>, _event_type: Option<&str>) -> Result<Vec<ClusterEvent>> {
            Ok(vec![])
        }

        async fn list_services(&self, _namespace: Option<&str>) -> Result<Vec<Service>> {
            Ok(vec![])
        }

        async fn list_ingresses(&self, _namespace: Option<&str>) -> Result<Vec<Ingress>> {
            Ok(vec![])
        }

        async fn list_namespaces(&self) -> Result<Vec<Namespace>> {
            Ok(vec![])
        }

        async fn get_storage_info(&self) -> Result<StorageInfo> {
            Ok(StorageInfo::default())
        }
    }

    #[tokio::test]
    async fn test_mock_kubernetes_repository() {
        let repo = MockKubernetesRepository;
        
        let overview = repo.get_cluster_overview().await.unwrap();
        assert_eq!(overview.node_count, 0);
        
        let nodes = repo.list_nodes().await.unwrap();
        assert!(nodes.is_empty());
        
        let result = repo.get_node("nonexistent").await;
        assert!(result.is_err());
    }
}

// Low Priority Ports (Phase 3)
pub mod low_priority_ports_part1;
pub mod low_priority_ports_part2;

pub use low_priority_ports_part1::*;
pub use low_priority_ports_part2::*;
