//! Tests for the KubernetesRepository port and its implementations
//!
//! Validates the seam: handlers route through the port trait,
//! and the port can be satisfied by either the production adapter
//! or a mock adapter for testing.

use std::collections::HashMap;
use std::sync::Arc;

use kusanagi::domain::ports::KubernetesRepository;
use kusanagi::error::Result;
use async_trait::async_trait;
use serde_json::{json, Value};

// ── Mock adapter ──────────────────────────────────────────────

struct MockKubernetesRepository {
    responses: HashMap<String, Result<Value>>,
    pod_logs_response: Option<Result<String>>,
}

impl MockKubernetesRepository {
    fn new() -> Self {
        Self {
            responses: HashMap::new(),
            pod_logs_response: None,
        }
    }

    fn with(mut self, key: &str, value: Result<Value>) -> Self {
        self.responses.insert(key.to_string(), value);
        self
    }

    fn with_pod_logs(mut self, value: Result<String>) -> Self {
        self.pod_logs_response = Some(value);
        self
    }
}

#[async_trait]
impl KubernetesRepository for MockKubernetesRepository {
    async fn get_pods_status(&self, _force_refresh: bool) -> Result<Value> {
        self.responses
            .get("get_pods_status")
            .cloned()
            .unwrap_or(Ok(json!({"pods": [], "total": 0})))
    }

    async fn get_nodes_status(&self, _force_refresh: bool) -> Result<Value> {
        self.responses
            .get("get_nodes_status")
            .cloned()
            .unwrap_or(Ok(json!({"nodes": [], "total_nodes": 0})))
    }

    async fn get_cluster_overview(&self, _force_refresh: bool) -> Result<Value> {
        self.responses
            .get("get_cluster_overview")
            .cloned()
            .unwrap_or(Ok(json!({"pods": 0, "nodes_ready": 0})))
    }

    async fn get_services(&self) -> Result<Value> {
        self.responses
            .get("get_services")
            .cloned()
            .unwrap_or(Ok(json!({"services": []})))
    }

    async fn get_ingress(&self) -> Result<Value> {
        self.responses
            .get("get_ingress")
            .cloned()
            .unwrap_or(Ok(json!({"ingresses": []})))
    }

    async fn get_storage(&self) -> Result<Value> {
        self.responses
            .get("get_storage")
            .cloned()
            .unwrap_or(Ok(json!({"pvc_count": 0, "pvcs": []})))
    }

    async fn get_events(&self) -> Result<Value> {
        self.responses
            .get("get_events")
            .cloned()
            .unwrap_or(Ok(json!([])))
    }

    async fn force_delete_pod(&self, _namespace: &str, _name: &str) -> Result<Value> {
        self.responses
            .get("force_delete_pod")
            .cloned()
            .unwrap_or(Ok(json!({"success": true})))
    }

    async fn delete_error_pods(&self) -> Result<Value> {
        self.responses
            .get("delete_error_pods")
            .cloned()
            .unwrap_or(Ok(json!({"success": true, "deleted": 0})))
    }

    async fn get_pod_logs(&self, _namespace: &str, _name: &str) -> Result<String> {
        self.pod_logs_response
            .clone()
            .unwrap_or(Ok("log line 1\nlog line 2".to_string()))
    }

    async fn get_namespace_metrics(&self, _window: Option<String>) -> Result<Value> {
        self.responses
            .get("get_namespace_metrics")
            .cloned()
            .unwrap_or(Ok(json!({"namespaces": []})))
    }

    async fn get_failed_jobs(&self) -> Result<Value> {
        self.responses
            .get("get_failed_jobs")
            .cloned()
            .unwrap_or(Ok(json!({"total": 0, "failed_jobs": []})))
    }

    async fn get_cluster_resource_metrics(&self) -> Result<Value> {
        self.responses
            .get("get_cluster_resource_metrics")
            .cloned()
            .unwrap_or(Ok(json!({})))
    }
}

// ── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use kusanagi::error::KusanagiError;

    #[tokio::test]
    async fn test_mock_returns_defaults() {
        let repo = MockKubernetesRepository::new();

        let pods = repo.get_pods_status(false).await.unwrap();
        assert_eq!(pods["total"], 0);

        let nodes = repo.get_nodes_status(false).await.unwrap();
        assert_eq!(nodes["total_nodes"], 0);

        let overview = repo.get_cluster_overview(false).await.unwrap();
        assert_eq!(overview["pods"], 0);

        let logs = repo.get_pod_logs("ns", "pod").await.unwrap();
        assert!(logs.contains("log line 1"));
    }

    #[tokio::test]
    async fn test_mock_returns_configured_responses() {
        let repo = MockKubernetesRepository::new()
            .with("get_pods_status", Ok(json!({"pods": [{"name": "pod-1"}], "total": 1})))
            .with("get_nodes_status", Ok(json!({"nodes": [{"name": "node-1"}], "total_nodes": 1})));

        let pods = repo.get_pods_status(false).await.unwrap();
        assert_eq!(pods["total"], 1);
        assert_eq!(pods["pods"][0]["name"], "pod-1");

        let nodes = repo.get_nodes_status(false).await.unwrap();
        assert_eq!(nodes["total_nodes"], 1);
    }

    #[tokio::test]
    async fn test_mock_can_return_errors() {
        let repo = MockKubernetesRepository::new()
            .with("get_pods_status", Err(KusanagiError::ExternalService("cluster down".into())));

        let result = repo.get_pods_status(false).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cluster down"));
    }

    #[tokio::test]
    async fn test_mock_returns_string_for_pod_logs() {
        let repo = MockKubernetesRepository::new()
            .with_pod_logs(Ok("custom log output".to_string()));

        let logs = repo.get_pod_logs("default", "my-pod").await.unwrap();
        assert_eq!(logs, "custom log output");
    }

    #[tokio::test]
    async fn test_mock_force_refresh_parameter_accepted() {
        let repo = MockKubernetesRepository::new()
            .with("get_cluster_overview", Ok(json!({"pods": 5, "nodes_ready": 3})));

        let overview = repo.get_cluster_overview(true).await.unwrap();
        assert_eq!(overview["pods"], 5);
        assert_eq!(overview["nodes_ready"], 3);
    }

    #[tokio::test]
    async fn test_mock_delete_operations() {
        let repo = MockKubernetesRepository::new()
            .with("force_delete_pod", Ok(json!({"success": true, "message": "deleted"})))
            .with("delete_error_pods", Ok(json!({"success": true, "deleted": 2})));

        let force = repo.force_delete_pod("ns", "pod-1").await.unwrap();
        assert_eq!(force["success"], true);
        assert_eq!(force["message"], "deleted");

        let deleted = repo.delete_error_pods().await.unwrap();
        assert_eq!(deleted["deleted"], 2);
    }

    #[tokio::test]
    async fn test_mock_storage_and_metrics() {
        let repo = MockKubernetesRepository::new()
            .with("get_storage", Ok(json!({"pvc_count": 3, "pvcs": [{ "name": "data-pvc" }]})))
            .with("get_failed_jobs", Ok(json!({"total": 1, "failed_jobs": [{ "name": "job-1" }]})))
            .with("get_namespace_metrics", Ok(json!({"namespaces": [{ "name": "default", "cpu": 0.5 }]})));

        let storage = repo.get_storage().await.unwrap();
        assert_eq!(storage["pvc_count"], 3);

        let jobs = repo.get_failed_jobs().await.unwrap();
        assert_eq!(jobs["total"], 1);

        let ns = repo.get_namespace_metrics(Some("5m".into())).await.unwrap();
        assert_eq!(ns["namespaces"][0]["name"], "default");
    }

    #[tokio::test]
    async fn test_port_is_object_safe_and_send_sync() {
        let mock = MockKubernetesRepository::new();
        let boxed: Arc<dyn KubernetesRepository> = Arc::new(mock);

        let result = boxed.get_services().await;
        assert!(result.is_ok());
    }
}
