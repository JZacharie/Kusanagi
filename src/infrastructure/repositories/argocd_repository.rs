//! ArgoCD Repository Implementation
//!
//! Implementation of the ArgoCdRepository port.

use async_trait::async_trait;
use kube::{
    api::{Api, ListParams, Patch, PatchParams},
    Client,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{info, error};

use crate::domain::ports::argocd_port::*;

/// ArgoCD Application CRD structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub api_version: String,
    pub kind: String,
    pub metadata: ObjectMeta,
    pub spec: ApplicationSpec,
    #[serde(default)]
    pub status: Option<ApplicationStatusData>,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            api_version: "argoproj.io/v1alpha1".to_string(),
            kind: "Application".to_string(),
            metadata: ObjectMeta::default(),
            spec: ApplicationSpec::default(),
            status: None,
        }
    }
}

impl kube::Resource for Application {
    type DynamicType = ();
    type Scope = kube::core::NamespaceResourceScope;

    fn group(_: &()) -> std::borrow::Cow<'_, str> {
        "argoproj.io".into()
    }

    fn version(_: &()) -> std::borrow::Cow<'_, str> {
        "v1alpha1".into()
    }

    fn plural(_: &()) -> std::borrow::Cow<'_, str> {
        "applications".into()
    }

    fn kind(_: &()) -> std::borrow::Cow<'_, str> {
        "Application".into()
    }

    fn api_version(_: &()) -> std::borrow::Cow<'_, str> {
        "argoproj.io/v1alpha1".into()
    }
    
    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }
    
    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApplicationSpec {
    #[serde(default)]
    pub source: Source,
    #[serde(default)]
    pub destination: Destination,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_policy: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Source {
    #[serde(rename = "repoURL", default)]
    pub repo_url: String,
    #[serde(default)]
    pub path: String,
    #[serde(rename = "targetRevision", skip_serializing_if = "Option::is_none")]
    pub target_revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Destination {
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApplicationStatusData {
    #[serde(default)]
    pub sync: Option<SyncStatusData>,
    #[serde(default)]
    pub health: Option<HealthStatusData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<RevisionHistoryData>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncStatusData {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthStatusData {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RevisionHistoryData {
    pub revision: String,
    #[serde(rename = "deployedAt")]
    pub deployed_at: String,
    #[serde(rename = "startedAt")]
    pub started_at: String,
}

/// ArgoCD repository implementation
pub struct ArgoCdRepositoryImpl {
    client: Client,
}

impl ArgoCdRepositoryImpl {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn new_arc(client: Client) -> Arc<Self> {
        Arc::new(Self::new(client))
    }
}

#[async_trait]
impl ArgoCdRepository for ArgoCdRepositoryImpl {
    async fn get_application_status(&self, name: &str) -> Result<ApplicationStatus, String> {
        let apps: Api<Application> = Api::namespaced(self.client.clone(), "argocd");
        
        let app = apps.get(name).await
            .map_err(|e| format!("Failed to get application: {}", e))?;

        let status = app.status.clone().unwrap_or_default();
        
        Ok(ApplicationStatus {
            name: name.to_string(),
            sync_status: parse_sync_status(status.sync.as_ref()),
            health_status: parse_health_status(status.health.as_ref()),
            revision: status
                .sync
                .and_then(|s| s.revision)
                .unwrap_or_default(),
        })
    }

    async fn sync_application(&self, name: &str) -> Result<(), String> {
        info!("Syncing ArgoCD application: {}", name);
        
        let apps: Api<Application> = Api::namespaced(self.client.clone(), "argocd");
        
        let patch = json!({
            "operation": {
                "sync": {
                    "prune": true,
                    "dryRun": false,
                    "strategy": {
                        "hook": {
                            "force": false
                        }
                    }
                }
            }
        });

        apps.patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map_err(|e| {
                error!("Failed to sync application {}: {}", name, e);
                format!("Failed to sync: {}", e)
            })?;

        info!("Successfully triggered sync for application: {}", name);
        Ok(())
    }

    async fn list_applications(&self) -> Result<Vec<ApplicationInfo>, String> {
        let apps: Api<Application> = Api::namespaced(self.client.clone(), "argocd");
        
        let app_list = apps.list(&ListParams::default()).await
            .map_err(|e| format!("Failed to list applications: {}", e))?;

        let mut result = Vec::new();
        
        for app in app_list.items {
            let metadata = &app.metadata;
            let spec = &app.spec;
            
            result.push(ApplicationInfo {
                name: metadata.name.clone().unwrap_or_default(),
                namespace: metadata.namespace.clone().unwrap_or_else(|| "argocd".to_string()),
                project: spec.project.clone().unwrap_or_else(|| "default".to_string()),
                repo_url: spec.source.repo_url.clone(),
                path: spec.source.path.clone(),
                target_revision: spec.source.target_revision.clone().unwrap_or_else(|| "HEAD".to_string()),
                destination_namespace: spec.destination.namespace.clone().unwrap_or_default(),
            });
        }

        Ok(result)
    }

    async fn get_application_details(&self, name: &str) -> Result<ApplicationDetails, String> {
        let apps: Api<Application> = Api::namespaced(self.client.clone(), "argocd");
        
        let app = apps.get(name).await
            .map_err(|e| format!("Failed to get application: {}", e))?;

        let status = app.status.clone().unwrap_or_default();
        
        // Parse resources
        let resources: Vec<ResourceStatus> = status
            .resources
            .unwrap_or_default()
            .iter()
            .map(|r| ResourceStatus {
                group: r.get("group").and_then(|g| g.as_str()).unwrap_or_default().to_string(),
                version: r.get("version").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                kind: r.get("kind").and_then(|k| k.as_str()).unwrap_or_default().to_string(),
                namespace: r.get("namespace").and_then(|n| n.as_str()).unwrap_or_default().to_string(),
                name: r.get("name").and_then(|n| n.as_str()).unwrap_or_default().to_string(),
                status: r.get("status").and_then(|s| s.as_str()).unwrap_or_default().to_string(),
                health: r.get("health").and_then(|h| h.get("status")).and_then(|s| s.as_str()).unwrap_or_default().to_string(),
            })
            .collect();

        // Parse history
        let history: Vec<RevisionHistory> = status
            .history
            .unwrap_or_default()
            .iter()
            .map(|h| RevisionHistory {
                revision: h.revision.clone(),
                deployed_at: h.deployed_at.clone(),
                deploy_started_at: h.started_at.clone(),
            })
            .collect();

        Ok(ApplicationDetails {
            name: name.to_string(),
            status: self.get_application_status(name).await?,
            resources,
            history,
        })
    }
}

fn parse_sync_status(sync: Option<&SyncStatusData>) -> SyncStatus {
    match sync.map(|s| s.status.as_str()) {
        Some("Synced") => SyncStatus::Synced,
        Some("OutOfSync") => SyncStatus::OutOfSync,
        Some("Unknown") => SyncStatus::Unknown,
        _ => SyncStatus::Unknown,
    }
}

fn parse_health_status(health: Option<&HealthStatusData>) -> HealthStatus {
    match health.map(|h| h.status.as_str()) {
        Some("Healthy") => HealthStatus::Healthy,
        Some("Degraded") => HealthStatus::Degraded,
        Some("Progressing") => HealthStatus::Progressing,
        Some("Suspended") => HealthStatus::Suspended,
        Some("Missing") => HealthStatus::Unknown,
        _ => HealthStatus::Unknown,
    }
}
