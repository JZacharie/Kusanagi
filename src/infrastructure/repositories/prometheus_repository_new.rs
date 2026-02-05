use async_trait::async_trait;
use crate::domain::ports::prometheus_port::{PrometheusRepository, PrometheusMetrics};
use crate::error::{Result, KusanagiError};
// use crate::legacy; // Disabled for core version

pub struct LegacyPrometheusRepository;

#[async_trait]
impl PrometheusRepository for LegacyPrometheusRepository {
    async fn query(&self, _query: &str) -> std::result::Result<f64, String> {
        Ok(0.0)
    }

    async fn query_raw(&self, _query: &str) -> std::result::Result<serde_json::Value, String> {
        Ok(serde_json::json!({}))
    }

    async fn query_range(&self, _query: &str, _start: i64, _end: i64, _step: &str) -> std::result::Result<serde_json::Value, String> {
        Ok(serde_json::json!({}))
    }

    async fn get_cluster_metrics(&self) -> std::result::Result<PrometheusMetrics, String> {
        Ok(PrometheusMetrics {
            cpu_usage_percent: 0.0,
            memory_usage_percent: 0.0,
            memory_usage_bytes: 0,
            pod_count: 0,
            node_count: 0,
            container_count: 0,
            alerts_firing: 0,
            alerts_pending: 0,
        })
    }
}
