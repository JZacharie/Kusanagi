pub mod application;
pub mod cache;
pub mod config;
pub mod domain;
pub mod error;
pub mod event_bus;
pub mod features;
pub mod infrastructure;
pub mod interfaces;
pub mod jobs;
pub mod legacy;
pub mod metrics;
pub mod middleware;
pub mod resilience;
pub mod response;
pub mod validation;

use std::sync::Arc;

/// Shared application state
pub struct AppState {
    pub client: kube::Client,
    pub k8s_repo: Arc<dyn domain::ports::KubernetesRepository>,
    pub metrics_repo: Arc<dyn domain::ports::MetricsRepository>,
}
