use async_trait::async_trait;
use crate::domain::ports::prometheus_port::{PrometheusRepository, PrometheusMetrics};
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacyPrometheusRepository;

#[async_trait]
impl PrometheusRepository for LegacyPrometheusRepository {
    async fn query(&self, query: &str) -> std::result::Result<f64, String> {
        legacy::prometheus::query_instant(query).await
            .map(|result| {
                result.get("data")
                    .and_then(|d| d.get("result"))
                    .and_then(|r| r.get(0))
                    .and_then(|res| res.get("value"))
                    .and_then(|v| v.get(1))
                    .and_then(|v| v.as_str())
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(0.0)
            })
            .map_err(|e| e.to_string())
    }

    async fn query_raw(&self, query: &str) -> std::result::Result<serde_json::Value, String> {
        legacy::prometheus::query_instant(query).await
            .map_err(|e| e.to_string())
    }

    async fn query_range(&self, query: &str, start: i64, end: i64, step: &str) -> std::result::Result<serde_json::Value, String> {
        legacy::prometheus::query_range(query, start, end, step).await
            .map_err(|e| e.to_string())
    }

    async fn get_cluster_metrics(&self) -> std::result::Result<PrometheusMetrics, String> {
        legacy::prometheus::get_cluster_metrics().await
            .map_err(|e| e.to_string())
    }
}
