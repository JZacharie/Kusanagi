//! Pod Repository Port
//!
//! Interface for pod data access - implemented by infrastructure layer.

use crate::domain::entities::{Pod, PodsStatus};
use async_trait::async_trait;

/// Pod repository interface
#[async_trait]
pub trait PodRepository: Send + Sync {
    /// Get all pods status
    async fn get_pods_status(&self) -> Result<PodsStatus, String>;
    
    /// Get pods in a specific namespace
    async fn get_pods_by_namespace(&self, namespace: &str) -> Result<Vec<Pod>, String>;
    
    /// Get a specific pod
    async fn get_pod(&self, namespace: &str, name: &str) -> Result<Option<Pod>, String>;
    
    /// Get pod logs
    async fn get_pod_logs(
        &self,
        namespace: &str,
        name: &str,
        container: Option<String>,
        tail_lines: i64,
    ) -> Result<String, String>;
    
    /// Force delete a pod
    async fn force_delete_pod(&self, namespace: &str, name: &str) -> Result<(), String>;
    
    /// Delete pods with error status
    async fn delete_error_pods(&self) -> Result<(usize, usize), String>;
    
    /// Scale a deployment
    async fn scale_deployment(&self, namespace: &str, name: &str, replicas: i32) -> Result<(), String>;
    
    /// Scale a statefulset
    async fn scale_statefulset(&self, namespace: &str, name: &str, replicas: i32) -> Result<(), String>;
}

/// Factory for creating pod repositories
pub trait PodRepositoryFactory: Send + Sync {
    fn create_repository(&self) -> Box<dyn PodRepository>;
}
