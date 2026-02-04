use crate::domain::ports::{PrometheusRepository, PrometheusMetrics};
use crate::error::Result;
use std::sync::Arc;

pub struct GetPrometheusMetricsUseCase {
    prometheus_repo: Arc<dyn PrometheusRepository>,
}

impl GetPrometheusMetricsUseCase {
    pub fn new(prometheus_repo: Arc<dyn PrometheusRepository>) -> Self {
        Self { prometheus_repo }
    }

    pub async fn execute(&self) -> Result<PrometheusMetrics> {
        self.prometheus_repo.get_cluster_metrics().await
            .map_err(|e| crate::error::KusanagiError::prometheus(e))
    }
}

pub struct QueryPrometheusUseCase {
    prometheus_repo: Arc<dyn PrometheusRepository>,
}

impl QueryPrometheusUseCase {
    pub fn new(prometheus_repo: Arc<dyn PrometheusRepository>) -> Self {
        Self { prometheus_repo }
    }

    pub async fn execute_raw(&self, query: &str) -> Result<serde_json::Value> {
        self.prometheus_repo.query_raw(query).await
            .map_err(|e| crate::error::KusanagiError::prometheus(e))
    }

    pub async fn execute_range(&self, query: &str, start: i64, end: i64, step: &str) -> Result<serde_json::Value> {
        self.prometheus_repo.query_range(query, start, end, step).await
            .map_err(|e| crate::error::KusanagiError::prometheus(e))
    }
}
