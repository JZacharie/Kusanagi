//! ArgoCD Repository Port
//!
//! Port defining the interface for ArgoCD operations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// ArgoCD sync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    #[serde(rename = "synced")]
    Synced,
    #[serde(rename = "out_of_sync")]
    OutOfSync,
    #[serde(rename = "unknown")]
    Unknown,
}

/// ArgoCD health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "progressing")]
    Progressing,
    #[serde(rename = "suspended")]
    Suspended,
    #[serde(rename = "unknown")]
    Unknown,
}

/// Application status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationStatus {
    pub name: String,
    pub sync_status: SyncStatus,
    pub health_status: HealthStatus,
    pub revision: String,
}

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationInfo {
    pub name: String,
    pub namespace: String,
    pub project: String,
    pub repo_url: String,
    pub path: String,
    pub target_revision: String,
    pub destination_namespace: String,
}

/// Resource status within an application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatus {
    pub group: String,
    pub version: String,
    pub kind: String,
    pub namespace: String,
    pub name: String,
    pub status: String,
    pub health: String,
}

/// Revision history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionHistory {
    pub revision: String,
    pub deployed_at: String,
    pub deploy_started_at: String,
}

/// Application details with resources and history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationDetails {
    pub name: String,
    pub status: ApplicationStatus,
    pub resources: Vec<ResourceStatus>,
    pub history: Vec<RevisionHistory>,
}

/// Port for ArgoCD repository operations
#[async_trait]
pub trait ArgoCdRepository: Send + Sync {
    /// Get application status
    async fn get_application_status(&self, name: &str) -> Result<ApplicationStatus, String>;

    /// Sync an application
    async fn sync_application(&self, name: &str) -> Result<(), String>;

    /// List all applications
    async fn list_applications(&self) -> Result<Vec<ApplicationInfo>, String>;

    /// Get application details
    async fn get_application_details(&self, name: &str) -> Result<ApplicationDetails, String>;
}
