//! Kubernetes Repository Implementation
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use crate::domain::ports::KubernetesRepository;
use crate::domain::services::kubernetes_service;
use crate::error::Result;
use crate::AdvancedCache;

pub struct KubernetesRepositoryImpl {
    http_client: Arc<reqwest::Client>,
    kube_client: Option<Arc<kube::Client>>,
    cache: Arc<AdvancedCache<String>>,
}

impl KubernetesRepositoryImpl {
    pub fn new(
        http_client: Arc<reqwest::Client>,
        kube_client: Option<Arc<kube::Client>>,
        cache: Arc<AdvancedCache<String>>,
    ) -> Self {
        Self {
            http_client,
            kube_client,
            cache,
        }
    }
}

#[async_trait]
impl KubernetesRepository for KubernetesRepositoryImpl {
    async fn get_pods_status(&self) -> Result<Value> {
        kubernetes_service::get_pods_status(&self.cache, false)
            .await
            .map_err(crate::error::KusanagiError::ExternalService)
    }

    async fn get_nodes_status(&self) -> Result<Value> {
        kubernetes_service::get_nodes_status(&self.http_client, &self.cache, false)
            .await
            .map_err(crate::error::KusanagiError::ExternalService)
    }

    async fn get_cluster_overview(&self) -> Result<Value> {
        kubernetes_service::get_cluster_overview(
            &self.http_client,
            &self.kube_client,
            &self.cache,
            false,
        )
        .await
        .map_err(crate::error::KusanagiError::ExternalService)
    }
}
