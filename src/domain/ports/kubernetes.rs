//! Kubernetes Repository Port
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait KubernetesRepository: Send + Sync {
    /// Get status of all pods
    async fn get_pods_status(&self, force_refresh: bool) -> Result<Value>;

    /// Get status of all nodes
    async fn get_nodes_status(&self, force_refresh: bool) -> Result<Value>;

    /// Get cluster overview
    async fn get_cluster_overview(&self, force_refresh: bool) -> Result<Value>;

    /// Get all services
    async fn get_services(&self) -> Result<Value>;

    /// Get all ingresses
    async fn get_ingress(&self) -> Result<Value>;

    /// Get storage/PVC information
    async fn get_storage(&self) -> Result<Value>;

    /// Get recent Kubernetes events
    async fn get_events(&self) -> Result<Value>;

    /// Force delete a specific pod
    async fn force_delete_pod(&self, namespace: &str, name: &str) -> Result<Value>;

    /// Delete all pods in error state
    async fn delete_error_pods(&self) -> Result<Value>;

    /// Get logs for a specific pod
    async fn get_pod_logs(&self, namespace: &str, name: &str) -> Result<String>;

    /// Get resource usage metrics per namespace
    async fn get_namespace_metrics(&self, window: Option<String>) -> Result<Value>;

    /// Get failed Kubernetes jobs from Prometheus
    async fn get_failed_jobs(&self) -> Result<Value>;

    /// Get cluster-wide resource metrics (capacity, allocatable, usage)
    async fn get_cluster_resource_metrics(&self) -> Result<Value>;
}
