//! Use cases (application services)
//!
//! Use cases orchestrate domain services to accomplish specific tasks.
//! They represent the application's use cases or user stories.

use crate::application::dtos::*;
use crate::application::mappers::*;
use crate::cache::{Cache, InMemoryCache};
use crate::config;
use crate::domain::entities::*;
use crate::domain::ports::*;
use crate::domain::services::*;
use crate::error::{KusanagiError, Result};
use std::sync::Arc;
use std::time::Duration;

pub mod cluster_use_cases;
pub mod pod_use_cases;
pub mod event_use_cases;
pub mod argocd_use_cases;
pub mod nodes_use_cases;
pub mod pods_use_cases;
pub mod mcp_use_cases;
pub mod node_use_cases;
pub mod storage_use_cases;
pub mod service_use_cases;
pub mod ingress_use_cases;
pub mod prometheus_use_cases;
pub mod backup_use_cases;
pub mod security_use_cases;
pub mod alert_use_cases;
pub mod chat_use_cases;
pub mod node_metrics_use_cases;

pub use cluster_use_cases::*;
pub use pod_use_cases::*;
pub use event_use_cases::*;
pub use argocd_use_cases::*;
pub use node_use_cases::*;
pub use storage_use_cases::*;
pub use service_use_cases::*;
pub use ingress_use_cases::*;
pub use prometheus_use_cases::*;
pub use backup_use_cases::*;
pub use security_use_cases::*;
pub use alert_use_cases::*;
pub use chat_use_cases::*;
pub use node_metrics_use_cases::*;

/// Application service for cluster operations
pub struct ClusterApplicationService {
    cluster_service: ClusterService,
    cache: Arc<InMemoryCache<String, ClusterOverviewDto>>,
}

impl ClusterApplicationService {
    pub fn new(
        k8s_repo: Arc<dyn KubernetesRepository>,
        metrics_repo: Arc<dyn MetricsRepository>,
    ) -> Self {
        let cluster_service = ClusterService::new(k8s_repo, metrics_repo);
        let cache_ttl = config::get().cache.default_ttl_secs;
        let cache = Arc::new(InMemoryCache::from_config("cluster_overview", cache_ttl));
        
        Self {
            cluster_service,
            cache,
        }
    }

    /// Get cluster overview (with caching)
    pub async fn get_cluster_overview(&self) -> Result<ClusterOverviewDto> {
        // Try cache first
        if let Some(dto) = self.cache.get(&"overview".to_string()).await {
            return Ok(dto);
        }

        // Fetch from domain service
        let overview = self.cluster_service.get_cluster_status().await?;
        let resources = self.cluster_service.assess_capacity().await?;
        
        // Create DTO
        let dto = ClusterOverviewDto {
            name: "cluster".to_string(), // Would come from actual cluster info
            version: "v1.28".to_string(),
            status: ClusterMapper::status_to_string(overview),
            node_count: 0, // Would be populated from actual data
            pod_count: 0,
            namespace_count: 0,
            cpu_percent: match resources.cpu_status {
                crate::domain::services::ResourceStatus::Healthy => 50.0,
                crate::domain::services::ResourceStatus::Warning => 75.0,
                crate::domain::services::ResourceStatus::Critical => 95.0,
            },
            memory_percent: match resources.memory_status {
                crate::domain::services::ResourceStatus::Healthy => 50.0,
                crate::domain::services::ResourceStatus::Warning => 80.0,
                crate::domain::services::ResourceStatus::Critical => 95.0,
            },
            alerts_firing: 0,
            alerts_pending: 0,
        };

        // Cache the result
        let ttl = Duration::from_secs(config::get().cache.default_ttl_secs);
        self.cache.set("overview".to_string(), dto.clone(), ttl).await;

        Ok(dto)
    }

    /// Get capacity assessment
    pub async fn get_capacity_assessment(&self) -> Result<CapacityAssessmentDto> {
        let assessment = self.cluster_service.assess_capacity().await?;
        
        Ok(CapacityAssessmentDto {
            cpu_status: format!("{:?}", assessment.cpu_status),
            memory_status: format!("{:?}", assessment.memory_status),
            recommendation: assessment.recommendation,
        })
    }
}

/// DTO for capacity assessment
#[derive(Debug, Clone, serde::Serialize)]
pub struct CapacityAssessmentDto {
    pub cpu_status: String,
    pub memory_status: String,
    pub recommendation: Option<String>,
}

/// Application service for node operations
pub struct NodeApplicationService {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl NodeApplicationService {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    /// List all nodes
    pub async fn list_nodes(&self) -> Result<Vec<NodeDto>> {
        let nodes = self.k8s_repo.list_nodes().await?;
        Ok(NodeMapper::to_dto_list(nodes))
    }

    /// Get node details
    pub async fn get_node(&self, name: &str) -> Result<NodeDto> {
        let node = self.k8s_repo.get_node(name).await?;
        Ok(NodeMapper::to_dto(node))
    }
}

/// Application service for namespace operations
pub struct NamespaceApplicationService {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl NamespaceApplicationService {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    /// List all namespaces
    pub async fn list_namespaces(&self) -> Result<Vec<NamespaceDto>> {
        let namespaces = self.k8s_repo.list_namespaces().await?;
        Ok(NamespaceMapper::to_dto_list(namespaces))
    }
}

/// Application service for service operations
pub struct ServiceApplicationService {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl ServiceApplicationService {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    /// List services
    pub async fn list_services(&self, namespace: Option<&str>) -> Result<Vec<ServiceDto>> {
        let services = self.k8s_repo.list_services(namespace).await?;
        Ok(ServiceMapper::to_dto_list(services))
    }
}

/// Application service for ingress operations
pub struct IngressApplicationService {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl IngressApplicationService {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    /// List ingresses
    pub async fn list_ingresses(&self, namespace: Option<&str>) -> Result<Vec<Ingress>> {
        self.k8s_repo.list_ingresses(namespace).await
    }
}

/// Application service for storage operations
pub struct StorageApplicationService {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl StorageApplicationService {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    /// Get storage information
    pub async fn get_storage_info(&self) -> Result<StorageDto> {
        let info = self.k8s_repo.get_storage_info().await?;
        Ok(StorageMapper::to_dto(info))
    }
}

/// Application service for metrics operations
pub struct MetricsApplicationService {
    metrics_repo: Arc<dyn MetricsRepository>,
    cache: Arc<InMemoryCache<String, MetricsDto>>,
}

impl MetricsApplicationService {
    pub fn new(metrics_repo: Arc<dyn MetricsRepository>) -> Self {
        let cache_ttl = config::get().cache.prometheus_ttl_secs;
        let cache = Arc::new(InMemoryCache::from_config("metrics", cache_ttl));
        
        Self {
            metrics_repo,
            cache,
        }
    }

    /// Get cluster metrics (with caching)
    pub async fn get_cluster_metrics(&self) -> Result<MetricsDto> {
        // Try cache first
        if let Some(dto) = self.cache.get(&"cluster".to_string()).await {
            return Ok(dto);
        }

        // Fetch from repository
        let resources = self.metrics_repo.get_cluster_metrics().await?;
        
        let dto = MetricsDto {
            cpu_usage_percent: resources.cpu_percent,
            memory_usage_percent: resources.memory_percent,
            pod_count: resources.pod_capacity,
            node_count: 0, // Would come from cluster info
            container_count: 0,
            alerts_firing: 0,
            alerts_pending: 0,
        };

        // Cache the result
        let ttl = Duration::from_secs(config::get().cache.prometheus_ttl_secs);
        self.cache.set("cluster".to_string(), dto.clone(), ttl).await;

        Ok(dto)
    }

    /// Query a specific metric
    pub async fn query_metric(&self, query: &str) -> Result<f64> {
        self.metrics_repo.query(query).await
    }
}

/// Application service for alert operations (legacy compatibility)
pub struct AlertApplicationService {
    alert_repo: Arc<dyn crate::domain::ports::AlertRepository>,
    cache: Arc<InMemoryCache<String, Vec<AlertDto>>>,
}

impl AlertApplicationService {
    pub fn new(alert_repo: Arc<dyn crate::domain::ports::AlertRepository>) -> Self {
        let cache_ttl = config::get().cache.default_ttl_secs;
        let cache = Arc::new(InMemoryCache::from_config("alerts", cache_ttl));
        
        Self {
            alert_repo,
            cache,
        }
    }

    /// Get active alerts (with caching)
    pub async fn get_active_alerts(&self) -> Result<Vec<AlertDto>> {
        // Try cache first
        if let Some(dtos) = self.cache.get(&"active".to_string()).await {
            return Ok(dtos);
        }

        // Fetch from repository - use new alerts use case
        let alerts = self.alert_repo.get_active_alerts().await
            .map_err(|e| KusanagiError::internal(format!("Failed to get alerts: {}", e)))?;
        let dtos = AlertMapper::to_dto_list(alerts.critical); // TODO: Map all alerts

        // Cache the result
        let ttl = Duration::from_secs(config::get().cache.default_ttl_secs);
        self.cache.set("active".to_string(), dtos.clone(), ttl).await;

        Ok(dtos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::*;

    // Mock implementations for testing
    struct MockK8sRepo;
    
    #[async_trait::async_trait]
    impl KubernetesRepository for MockK8sRepo {
        async fn get_cluster_overview(&self) -> Result<ClusterOverview> {
            Ok(ClusterOverview::default())
        }
        async fn list_nodes(&self) -> Result<Vec<Node>> { Ok(vec![]) }
        async fn get_node(&self, _name: &str) -> Result<Node> {
            Err(crate::error::KusanagiError::not_found("Node", _name))
        }
        async fn list_pods(&self, _ns: Option<&str>) -> Result<Vec<Pod>> { Ok(vec![]) }
        async fn get_pod(&self, _ns: &str, _name: &str) -> Result<Pod> {
            Ok(Pod {
                name: _name.to_string(),
                namespace: _ns.to_string(),
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
        async fn get_pod_logs(&self, _ns: &str, _name: &str, _c: Option<&str>, _t: i64) -> Result<String> {
            Ok("".to_string())
        }
        async fn delete_pod(&self, _ns: &str, _name: &str) -> Result<()> { Ok(()) }
        async fn force_delete_pod(&self, _ns: &str, _name: &str) -> Result<()> { Ok(()) }
        async fn get_pods_status(&self) -> Result<PodsStatus> {
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
        async fn delete_error_pods(&self) -> Result<(usize, usize)> { Ok((0, 0)) }
        async fn scale_deployment(&self, _ns: &str, _name: &str, _replicas: i32) -> Result<()> { Ok(()) }
        async fn scale_statefulset(&self, _ns: &str, _name: &str, _replicas: i32) -> Result<()> { Ok(()) }
        async fn list_events(&self, _ns: Option<&str>, _t: Option<&str>) -> Result<Vec<ClusterEvent>> { Ok(vec![]) }
        async fn list_services(&self, _ns: Option<&str>) -> Result<Vec<Service>> { Ok(vec![]) }
        async fn list_ingresses(&self, _ns: Option<&str>) -> Result<Vec<Ingress>> { Ok(vec![]) }
        async fn list_namespaces(&self) -> Result<Vec<Namespace>> {
            Ok(vec![Namespace {
                name: "default".to_string(),
                status: "Active".to_string(),
                pod_count: 5,
                resource_quota: None,
                age: "1d".to_string(),
            }])
        }
        async fn get_storage_info(&self) -> Result<StorageInfo> { Ok(StorageInfo::default()) }
    }

    struct MockMetricsRepo;
    
    #[async_trait::async_trait]
    impl MetricsRepository for MockMetricsRepo {
        async fn query(&self, _q: &str) -> Result<f64> { Ok(42.0) }
        async fn query_raw(&self, _q: &str) -> Result<serde_json::Value> { Ok(serde_json::Value::Null) }
        async fn query_range(&self, _q: &str, _s: i64, _e: i64, _st: &str) -> Result<serde_json::Value> {
            Ok(serde_json::Value::Null)
        }
        async fn get_cluster_metrics(&self) -> Result<ClusterResources> {
            Ok(ClusterResources {
                cpu_percent: 50.0,
                memory_percent: 60.0,
                ..Default::default()
            })
        }
        async fn get_pod_resource_usage(&self) -> Result<std::collections::HashMap<(String, String), (f64, i64)>> {
            Ok(Default::default())
        }
    }

    struct MockAlertRepo;
    
    #[async_trait::async_trait]
    impl crate::domain::ports::AlertRepository for MockAlertRepo {
        async fn get_active_alerts(&self) -> std::result::Result<crate::domain::entities::AlertsResponse, String> {
            Ok(crate::domain::entities::AlertsResponse {
                critical: vec![],
                warning: vec![],
                info: vec![],
                total: 0,
                firing: 0,
                pending: 0,
            })
        }
        async fn get_cached_alerts(&self) -> std::result::Result<crate::domain::entities::AlertsResponse, String> {
            self.get_active_alerts().await
        }
        async fn get_alert(&self, _fingerprint: &str) -> std::result::Result<crate::domain::entities::Alert, String> {
            Err("Not found".to_string())
        }
        async fn silence_alert(&self, _fingerprint: &str, _duration_secs: u64) -> std::result::Result<(), String> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_cluster_application_service() {
        // Initialize config
        let _ = crate::config::init();
        
        let k8s = Arc::new(MockK8sRepo);
        let metrics = Arc::new(MockMetricsRepo);
        let service = ClusterApplicationService::new(k8s, metrics);
        
        let overview = service.get_cluster_overview().await.unwrap();
        assert!(!overview.status.is_empty());
    }

    #[tokio::test]
    async fn test_namespace_application_service() {
        let k8s = Arc::new(MockK8sRepo);
        let service = NamespaceApplicationService::new(k8s);
        
        let namespaces = service.list_namespaces().await.unwrap();
        assert_eq!(namespaces.len(), 1);
        assert_eq!(namespaces[0].name, "default");
    }

    #[tokio::test]
    async fn test_metrics_application_service() {
        // Initialize config
        let _ = crate::config::init();
        
        let metrics = Arc::new(MockMetricsRepo);
        let service = MetricsApplicationService::new(metrics);
        
        let result = service.get_cluster_metrics().await.unwrap();
        assert_eq!(result.cpu_usage_percent, 50.0);
        assert_eq!(result.memory_usage_percent, 60.0);
    }

    #[tokio::test]
    async fn test_alert_application_service() {
        // Initialize config
        let _ = crate::config::init();
        
        let alerts = Arc::new(MockAlertRepo);
        let service = AlertApplicationService::new(alerts);
        
        let result = service.get_active_alerts().await.unwrap();
        assert!(result.is_empty());
    }
}
