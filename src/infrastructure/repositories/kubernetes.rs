//! Kubernetes Repository Implementation
use crate::domain::ports::KubernetesRepository;
use crate::domain::services::kubernetes_service;
use crate::error::{KusanagiError, Result};
use crate::AdvancedCache;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

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
    async fn get_pods_status(&self, force_refresh: bool) -> Result<Value> {
        kubernetes_service::get_pods_status(&self.cache, force_refresh)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn get_nodes_status(&self, force_refresh: bool) -> Result<Value> {
        kubernetes_service::get_nodes_status(&self.http_client, &self.cache, force_refresh)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn get_cluster_overview(&self, force_refresh: bool) -> Result<Value> {
        kubernetes_service::get_cluster_overview(
            &self.http_client,
            &self.kube_client,
            &self.cache,
            force_refresh,
        )
        .await
        .map_err(KusanagiError::ExternalService)
    }

    async fn get_services(&self) -> Result<Value> {
        kubernetes_service::get_services(&self.kube_client, &self.cache)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn get_ingress(&self) -> Result<Value> {
        kubernetes_service::get_ingress(&self.kube_client, &self.cache)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn get_storage(&self) -> Result<Value> {
        kubernetes_service::get_storage(&self.http_client)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn get_events(&self) -> Result<Value> {
        kubernetes_service::get_events()
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn force_delete_pod(&self, namespace: &str, name: &str) -> Result<Value> {
        kubernetes_service::force_delete_pod(&self.kube_client, namespace, name)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn delete_error_pods(&self) -> Result<Value> {
        kubernetes_service::delete_error_pods(&self.kube_client)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn get_pod_logs(&self, namespace: &str, name: &str) -> Result<String> {
        kubernetes_service::get_pod_logs(namespace, name)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn get_namespace_metrics(&self, window: Option<String>) -> Result<Value> {
        kubernetes_service::get_namespace_metrics(&self.http_client, window)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn get_failed_jobs(&self) -> Result<Value> {
        kubernetes_service::get_failed_jobs(&self.http_client)
            .await
            .map_err(KusanagiError::ExternalService)
    }

    async fn get_cluster_resource_metrics(&self) -> Result<Value> {
        kubernetes_service::get_cluster_resource_metrics(&self.http_client)
            .await
            .map_err(KusanagiError::ExternalService)
    }
}
