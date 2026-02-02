//! Prometheus Use Cases
//!
//! Application layer use cases for Prometheus metrics operations.

use crate::domain::ports::{PrometheusRepository, PrometheusMetrics};
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// Get cluster metrics use case
pub struct GetClusterMetricsUseCase {
    repository: Arc<dyn PrometheusRepository>,
}

impl GetClusterMetricsUseCase {
    pub fn new(repository: Arc<dyn PrometheusRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<PrometheusMetrics> {
        self.repository.get_cluster_metrics().await
            .map_err(|e| KusanagiError::internal(format!("Failed to get cluster metrics: {}", e)))
    }
}

/// Query Prometheus metric use case
pub struct QueryMetricUseCase {
    repository: Arc<dyn PrometheusRepository>,
}

impl QueryMetricUseCase {
    pub fn new(repository: Arc<dyn PrometheusRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, query: &str) -> Result<f64> {
        self.repository.query(query).await
            .map_err(|e| KusanagiError::internal(format!("Query failed: {}", e)))
    }
}

/// Query raw Prometheus data use case
pub struct QueryRawUseCase {
    repository: Arc<dyn PrometheusRepository>,
}

impl QueryRawUseCase {
    pub fn new(repository: Arc<dyn PrometheusRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, query: &str) -> Result<serde_json::Value> {
        self.repository.query_raw(query).await
            .map_err(|e| KusanagiError::internal(format!("Query failed: {}", e)))
    }
}

/// Query range data use case
pub struct QueryRangeUseCase {
    repository: Arc<dyn PrometheusRepository>,
}

impl QueryRangeUseCase {
    pub fn new(repository: Arc<dyn PrometheusRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, query: &str, start: i64, end: i64, step: &str) -> Result<serde_json::Value> {
        self.repository.query_range(query, start, end, step).await
            .map_err(|e| KusanagiError::internal(format!("Query range failed: {}", e)))
    }
}

/// Prometheus service - aggregates all Prometheus use cases
pub struct PrometheusUseCaseService {
    pub get_cluster_metrics: GetClusterMetricsUseCase,
    pub query: QueryMetricUseCase,
    pub query_raw: QueryRawUseCase,
    pub query_range: QueryRangeUseCase,
}

impl PrometheusUseCaseService {
    pub fn new(repository: Arc<dyn PrometheusRepository>) -> Self {
        Self {
            get_cluster_metrics: GetClusterMetricsUseCase::new(repository.clone()),
            query: QueryMetricUseCase::new(repository.clone()),
            query_raw: QueryRawUseCase::new(repository.clone()),
            query_range: QueryRangeUseCase::new(repository),
        }
    }
}
