use async_trait::async_trait;
use crate::error::Result;

/// Port for system operations and health checks
#[async_trait]
pub trait SystemRepository: Send + Sync {
    async fn get_system_status(&self) -> Result<serde_json::Value>;
    async fn trigger_rollout(&self, deployment: &str) -> Result<()>;
    async fn get_system_logs(&self, lines: Option<u32>) -> Result<Vec<String>>;
    async fn check_health(&self) -> Result<serde_json::Value>;
}

/// Port for database operations
#[async_trait]
pub trait DatabaseRepository: Send + Sync {
    async fn check_health(&self) -> Result<serde_json::Value>;
    async fn get_stats(&self) -> Result<serde_json::Value>;
    async fn execute_query(&self, query: &str) -> Result<serde_json::Value>;
}
