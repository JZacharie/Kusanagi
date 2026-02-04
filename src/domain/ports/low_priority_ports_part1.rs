//! Low priority ports - Domain contracts for remaining 18 modules

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// Services Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub namespace: String,
    pub service_type: String,
    pub cluster_ip: String,
    pub ports: Vec<ServicePort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: String,
    pub port: u16,
    pub target_port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDetails {
    pub service: Service,
    pub endpoints: Vec<String>,
    pub selector: std::collections::HashMap<String, String>,
}

#[async_trait]
pub trait ServicesRepository: Send + Sync {
    async fn list_services(&self, namespace: Option<&str>) -> Result<Vec<Service>>;
    async fn get_service_details(&self, namespace: &str, name: &str) -> Result<ServiceDetails>;
}

// Ingress Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingress {
    pub name: String,
    pub namespace: String,
    pub hosts: Vec<String>,
    pub tls_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressRule {
    pub host: String,
    pub paths: Vec<IngressPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressPath {
    pub path: String,
    pub service_name: String,
    pub service_port: u16,
}

#[async_trait]
pub trait IngressRepository: Send + Sync {
    async fn list_ingresses(&self, namespace: Option<&str>) -> Result<Vec<Ingress>>;
    async fn get_ingress_rules(&self, namespace: &str, name: &str) -> Result<Vec<IngressRule>>;
}

// Alertmanager Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub name: String,
    pub severity: String,
    pub status: String,
    pub message: String,
    pub timestamp: u64,
}

#[async_trait]
pub trait AlertmanagerRepository: Send + Sync {
    async fn get_alerts(&self) -> Result<Vec<Alert>>;
    async fn silence_alert(&self, alert_id: &str, duration: u64) -> Result<()>;
}

// Quota Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub name: String,
    pub namespace: String,
    pub limits: std::collections::HashMap<String, String>,
    pub used: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaUsage {
    pub cpu_used: f64,
    pub memory_used: f64,
    pub cpu_limit: f64,
    pub memory_limit: f64,
}

#[async_trait]
pub trait QuotaRepository: Send + Sync {
    async fn get_resource_quotas(&self, namespace: &str) -> Result<Vec<ResourceQuota>>;
    async fn get_quota_usage(&self, namespace: &str) -> Result<QuotaUsage>;
}

// Setup Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub name: String,
    pub version: String,
    pub node_count: u32,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatus {
    pub initialized: bool,
    pub components: Vec<ComponentStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub name: String,
    pub status: String,
    pub version: String,
}

#[async_trait]
pub trait SetupRepository: Send + Sync {
    async fn initialize_cluster(&self, config: &ClusterConfig) -> Result<()>;
    async fn get_setup_status(&self) -> Result<SetupStatus>;
}

// WebSocket Domain
#[async_trait]
pub trait WebSocketRepository: Send + Sync {
    async fn broadcast_message(&self, message: &str) -> Result<()>;
    async fn get_active_connections(&self) -> Result<u32>;
}

// Chat Storage Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub user_id: String,
    pub messages: Vec<crate::domain::entities::ChatMessage>,
    pub timestamp: u64,
}

#[async_trait]
pub trait ChatStorageRepository: Send + Sync {
    async fn store_conversation(&self, conversation: &Conversation) -> Result<()>;
    async fn get_conversation_history(&self, user_id: &str) -> Result<Vec<Conversation>>;
}

// Export Domain
#[async_trait]
pub trait ExportRepository: Send + Sync {
    async fn export_cluster_config(&self, format: &str) -> Result<String>;
    async fn export_metrics(&self, start_time: u64, end_time: u64) -> Result<String>;
}

// Apps Domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub name: String,
    pub version: String,
    pub status: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub image: String,
    pub replicas: u32,
    pub namespace: String,
}

#[async_trait]
pub trait AppsRepository: Send + Sync {
    async fn list_applications(&self) -> Result<Vec<Application>>;
    async fn deploy_application(&self, app_config: &ApplicationConfig) -> Result<()>;
}
