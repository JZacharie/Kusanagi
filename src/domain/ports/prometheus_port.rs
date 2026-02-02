//! Prometheus Port
//!
//! Port defining the interface for Prometheus metrics operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Prometheus metrics data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrometheusMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub memory_usage_bytes: i64,
    pub pod_count: i32,
    pub node_count: i32,
    pub container_count: i32,
    pub alerts_firing: i32,
    pub alerts_pending: i32,
}

/// Port for Prometheus operations
#[async_trait]
pub trait PrometheusRepository: Send + Sync {
    /// Query a Prometheus metric
    async fn query(&self, query: &str) -> Result<f64, String>;
    
    /// Query raw Prometheus data
    async fn query_raw(&self, query: &str) -> Result<serde_json::Value, String>;
    
    /// Query range data
    async fn query_range(&self, query: &str, start: i64, end: i64, step: &str) -> Result<serde_json::Value, String>;
    
    /// Get cluster metrics
    async fn get_cluster_metrics(&self) -> Result<PrometheusMetrics, String>;
}
