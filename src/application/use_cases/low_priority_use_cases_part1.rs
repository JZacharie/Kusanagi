//! Low priority modules migration - Phase 3
//! 18 modules restants avec patterns établis

use crate::domain::ports::*;
use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

// Services Use Cases
pub struct ServicesUseCases {
    services_repo: Arc<dyn ServicesRepository>,
}

impl ServicesUseCases {
    pub fn new(services_repo: Arc<dyn ServicesRepository>) -> Self {
        Self { services_repo }
    }

    pub async fn list_services(&self, namespace: Option<&str>) -> Result<Vec<Service>> {
        self.services_repo.list_services(namespace).await
    }

    pub async fn get_service_details(&self, namespace: &str, name: &str) -> Result<ServiceDetails> {
        self.services_repo.get_service_details(namespace, name).await
    }
}

// Ingress Use Cases
pub struct IngressUseCases {
    ingress_repo: Arc<dyn IngressRepository>,
}

impl IngressUseCases {
    pub fn new(ingress_repo: Arc<dyn IngressRepository>) -> Self {
        Self { ingress_repo }
    }

    pub async fn list_ingresses(&self, namespace: Option<&str>) -> Result<Vec<Ingress>> {
        self.ingress_repo.list_ingresses(namespace).await
    }

    pub async fn get_ingress_rules(&self, namespace: &str, name: &str) -> Result<Vec<IngressRule>> {
        self.ingress_repo.get_ingress_rules(namespace, name).await
    }
}

// Alertmanager Use Cases
pub struct AlertmanagerUseCases {
    alertmanager_repo: Arc<dyn AlertmanagerRepository>,
}

impl AlertmanagerUseCases {
    pub fn new(alertmanager_repo: Arc<dyn AlertmanagerRepository>) -> Self {
        Self { alertmanager_repo }
    }

    pub async fn get_alerts(&self) -> Result<Vec<Alert>> {
        self.alertmanager_repo.get_alerts().await
    }

    pub async fn silence_alert(&self, alert_id: &str, duration: u64) -> Result<()> {
        self.alertmanager_repo.silence_alert(alert_id, duration).await
    }
}

// Quota Use Cases
pub struct QuotaUseCases {
    quota_repo: Arc<dyn QuotaRepository>,
}

impl QuotaUseCases {
    pub fn new(quota_repo: Arc<dyn QuotaRepository>) -> Self {
        Self { quota_repo }
    }

    pub async fn get_resource_quotas(&self, namespace: &str) -> Result<Vec<ResourceQuota>> {
        self.quota_repo.get_resource_quotas(namespace).await
    }

    pub async fn get_quota_usage(&self, namespace: &str) -> Result<QuotaUsage> {
        self.quota_repo.get_quota_usage(namespace).await
    }
}

// Setup Use Cases
pub struct SetupUseCases {
    setup_repo: Arc<dyn SetupRepository>,
}

impl SetupUseCases {
    pub fn new(setup_repo: Arc<dyn SetupRepository>) -> Self {
        Self { setup_repo }
    }

    pub async fn initialize_cluster(&self, config: &ClusterConfig) -> Result<()> {
        self.setup_repo.initialize_cluster(config).await
    }

    pub async fn get_setup_status(&self) -> Result<SetupStatus> {
        self.setup_repo.get_setup_status().await
    }
}

// WebSocket Use Cases
pub struct WebSocketUseCases {
    websocket_repo: Arc<dyn WebSocketRepository>,
}

impl WebSocketUseCases {
    pub fn new(websocket_repo: Arc<dyn WebSocketRepository>) -> Self {
        Self { websocket_repo }
    }

    pub async fn broadcast_message(&self, message: &str) -> Result<()> {
        self.websocket_repo.broadcast_message(message).await
    }

    pub async fn get_active_connections(&self) -> Result<u32> {
        self.websocket_repo.get_active_connections().await
    }
}

// Chat Storage Use Cases
pub struct ChatStorageUseCases {
    chat_storage_repo: Arc<dyn ChatStorageRepository>,
}

impl ChatStorageUseCases {
    pub fn new(chat_storage_repo: Arc<dyn ChatStorageRepository>) -> Self {
        Self { chat_storage_repo }
    }

    pub async fn store_conversation(&self, conversation: &Conversation) -> Result<()> {
        self.chat_storage_repo.store_conversation(conversation).await
    }

    pub async fn get_conversation_history(&self, user_id: &str) -> Result<Vec<Conversation>> {
        self.chat_storage_repo.get_conversation_history(user_id).await
    }
}

// Export Use Cases
pub struct ExportUseCases {
    export_repo: Arc<dyn ExportRepository>,
}

impl ExportUseCases {
    pub fn new(export_repo: Arc<dyn ExportRepository>) -> Self {
        Self { export_repo }
    }

    pub async fn export_cluster_config(&self, format: &str) -> Result<String> {
        self.export_repo.export_cluster_config(format).await
    }

    pub async fn export_metrics(&self, start_time: u64, end_time: u64) -> Result<String> {
        self.export_repo.export_metrics(start_time, end_time).await
    }
}

// Apps Use Cases
pub struct AppsUseCases {
    apps_repo: Arc<dyn AppsRepository>,
}

impl AppsUseCases {
    pub fn new(apps_repo: Arc<dyn AppsRepository>) -> Self {
        Self { apps_repo }
    }

    pub async fn list_applications(&self) -> Result<Vec<Application>> {
        self.apps_repo.list_applications().await
    }

    pub async fn deploy_application(&self, app_config: &ApplicationConfig) -> Result<()> {
        self.apps_repo.deploy_application(app_config).await
    }
}
