use chrono::{DateTime, Utc};
use kube::{
    api::{Api, ListParams, Patch, PatchParams},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

/// ArgoCD Application structure (simplified)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    pub metadata: ApplicationMetadata,
    pub spec: ApplicationSpec,
    #[serde(default)]
    pub status: Option<ApplicationStatus>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationMetadata {
    pub name: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSpec {
    pub project: Option<String>,
    pub source: Option<ApplicationSource>,
    pub destination: Option<ApplicationDestination>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationSource {
    pub repo_url: Option<String>,
    pub path: Option<String>,
    pub target_revision: Option<String>,
    pub chart: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationDestination {
    pub server: Option<String>,
    pub namespace: Option<String>,
}

/// ArgoCD Application Status
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationStatus {
    pub sync: Option<SyncStatus>,
    pub health: Option<HealthStatus>,
    pub operation_state: Option<OperationState>,
    pub reconciled_at: Option<String>,
    pub resources: Option<Vec<ResourceStatus>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub status: Option<String>,
    pub revision: Option<String>,
    pub compared_to: Option<ComparedTo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparedTo {
    pub source: Option<ApplicationSource>,
    pub destination: Option<ApplicationDestination>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub status: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationState {
    pub phase: Option<String>,
    pub message: Option<String>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub sync_result: Option<SyncResult>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncResult {
    pub revision: Option<String>,
    pub source: Option<ApplicationSource>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceStatus {
    pub group: Option<String>,
    pub version: Option<String>,
    pub kind: Option<String>,
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub status: Option<String>,
    pub health: Option<HealthStatus>,
    pub requires_pruning: Option<bool>,
}

/// Application List response
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationList {
    pub items: Vec<Application>,
}

/// Issue category
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    RealIssue,
    UpgradeAvailable,
    Progressing,
}

/// Response structure for the API
#[derive(Clone, Debug, Serialize)]
pub struct ArgoStatusResponse {
    pub total: usize,
    pub healthy: usize,
    pub unhealthy: usize,
    pub synced: usize,
    pub out_of_sync: usize,
    pub unknown: usize,
    pub progressing: usize,
    pub upgrades_available: usize,
    pub apps_with_issues: Vec<AppIssue>,
    pub apps_with_upgrades: Vec<AppIssue>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AppIssue {
    pub name: String,
    pub namespace: String,
    pub health_status: String,
    pub sync_status: String,
    pub message: Option<String>,
    pub error_since: Option<String>,
    pub error_duration: Option<String>,
    pub category: IssueCategory,
    pub target_revision: Option<String>,
    pub current_revision: Option<String>,
    pub is_helm_chart: bool,
    pub can_sync: bool,
    pub argocd_url: String,
}

/// Sync request
#[derive(Clone, Debug, Deserialize)]
pub struct SyncRequest {
    pub app_name: String,
}

/// Sync response
#[derive(Clone, Debug, Serialize)]
pub struct SyncResponse {
    pub success: bool,
    pub message: String,
}

/// Get ArgoCD applications status
pub async fn get_argocd_status() -> Result<ArgoStatusResponse, String> {
    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create Kubernetes client: {}", e))?;

    // Use dynamic API to get ArgoCD Applications
    let apps_api: Api<kube::core::DynamicObject> = Api::namespaced_with(
        client,
        "argocd",
        &kube::discovery::ApiResource {
            group: "argoproj.io".to_string(),
            version: "v1alpha1".to_string(),
            api_version: "argoproj.io/v1alpha1".to_string(),
            kind: "Application".to_string(),
            plural: "applications".to_string(),
        },
    );

    let app_list = apps_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list ArgoCD applications: {}", e))?;

    let mut response = ArgoStatusResponse {
        total: app_list.items.len(),
        healthy: 0,
        unhealthy: 0,
        synced: 0,
        out_of_sync: 0,
        unknown: 0,
        progressing: 0,
        upgrades_available: 0,
        apps_with_issues: Vec::new(),
        apps_with_upgrades: Vec::new(),
    };

    let now = Utc::now();

    for app in app_list.items {
        let name = app.metadata.name.clone().unwrap_or_default();

        // Extract status from dynamic object data
        let status: ApplicationStatus = app
            .data
            .get("status")
            .and_then(|s| serde_json::from_value(s.clone()).ok())
            .unwrap_or_default();

        let spec: ApplicationSpec = app
            .data
            .get("spec")
            .and_then(|s| serde_json::from_value(s.clone()).ok())
            .unwrap_or(ApplicationSpec {
                project: None,
                source: None,
                destination: None,
            });

        let dest_namespace = spec
            .destination
            .as_ref()
            .and_then(|d| d.namespace.clone())
            .unwrap_or_default();

        let health_status = status
            .health
            .as_ref()
            .and_then(|h| h.status.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let sync_status = status
            .sync
            .as_ref()
            .and_then(|s| s.status.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        // Check if this is a Helm chart
        let is_helm_chart = spec.source.as_ref().and_then(|s| s.chart.clone()).is_some();
        
        // Get target revision (could be a version like "1.2.3" or "*" or "HEAD")
        let target_revision = spec.source.as_ref().and_then(|s| s.target_revision.clone());
        
        // Get current synced revision
        let current_revision = status.sync.as_ref().and_then(|s| s.revision.clone());

        // Count health statuses
        match health_status.as_str() {
            "Healthy" => response.healthy += 1,
            "Progressing" => response.progressing += 1,
            "Unknown" => response.unknown += 1,
            _ => response.unhealthy += 1,
        }

        // Count sync statuses
        match sync_status.as_str() {
            "Synced" => response.synced += 1,
            "OutOfSync" => response.out_of_sync += 1,
            _ => {}
        }

        // Check if app has issues
        let has_issue = health_status != "Healthy"
            || sync_status == "OutOfSync"
            || sync_status == "Unknown";

        if has_issue {
            let message = status
                .health
                .as_ref()
                .and_then(|h| h.message.clone())
                .or_else(|| {
                    status
                        .operation_state
                        .as_ref()
                        .and_then(|o| o.message.clone())
                });

            // Determine the category of the issue
            let category = categorize_issue(
                &health_status,
                &sync_status,
                &message,
                is_helm_chart,
                &target_revision,
            );

            // Try to determine when the error started
            let (error_since, error_duration) = calculate_error_duration(&status, &now);

            // Build ArgoCD URL
            let argocd_url = format!(
                "https://argocd.p.zacharie.org/applications/argocd/{}",
                name
            );

            let app_issue = AppIssue {
                name: name.clone(),
                namespace: dest_namespace,
                health_status: health_status.clone(),
                sync_status: sync_status.clone(),
                message,
                error_since,
                error_duration,
                category: category.clone(),
                target_revision,
                current_revision,
                is_helm_chart,
                can_sync: health_status == "Healthy" || health_status == "Progressing",
                argocd_url,
            };

            match category {
                IssueCategory::UpgradeAvailable => {
                    response.upgrades_available += 1;
                    response.apps_with_upgrades.push(app_issue);
                }
                _ => {
                    response.apps_with_issues.push(app_issue);
                }
            }
        }
    }

    info!(
        "ArgoCD status: {} total, {} healthy, {} issues, {} upgrades",
        response.total,
        response.healthy,
        response.apps_with_issues.len(),
        response.apps_with_upgrades.len()
    );

    Ok(response)
}

/// Categorize the type of issue
fn categorize_issue(
    health_status: &str,
    sync_status: &str,
    message: &Option<String>,
    is_helm_chart: bool,
    target_revision: &Option<String>,
) -> IssueCategory {
    // If app is progressing, it's just in progress
    if health_status == "Progressing" {
        return IssueCategory::Progressing;
    }

    // If app is healthy but out of sync, check if it's likely an upgrade
    if health_status == "Healthy" && sync_status == "OutOfSync" {
        // Check if target revision suggests auto-upgrade (*, latest, etc.)
        if let Some(ref rev) = target_revision {
            if rev == "*" || rev.to_lowercase() == "latest" || rev.to_lowercase() == "head" {
                return IssueCategory::UpgradeAvailable;
            }
        }
        
        // Check message for upgrade-related keywords
        if let Some(ref msg) = message {
            let msg_lower = msg.to_lowercase();
            if msg_lower.contains("successfully synced")
                || msg_lower.contains("no more tasks")
                || msg_lower.contains("all tasks run")
            {
                // App synced successfully before, likely just needs re-sync for new version
                if is_helm_chart {
                    return IssueCategory::UpgradeAvailable;
                }
            }
        }

        // For Helm charts with targetRevision set to any version, consider it an upgrade
        if is_helm_chart {
            return IssueCategory::UpgradeAvailable;
        }
    }

    // Everything else is a real issue
    IssueCategory::RealIssue
}

/// Trigger sync for an ArgoCD application
pub async fn sync_application(app_name: &str) -> Result<SyncResponse, String> {
    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create Kubernetes client: {}", e))?;

    let apps_api: Api<kube::core::DynamicObject> = Api::namespaced_with(
        client,
        "argocd",
        &kube::discovery::ApiResource {
            group: "argoproj.io".to_string(),
            version: "v1alpha1".to_string(),
            api_version: "argoproj.io/v1alpha1".to_string(),
            kind: "Application".to_string(),
            plural: "applications".to_string(),
        },
    );

    // Add sync operation annotation to trigger sync
    let patch = json!({
        "metadata": {
            "annotations": {
                "argocd.argoproj.io/refresh": "hard"
            }
        },
        "operation": {
            "initiatedBy": {
                "username": "kusanagi"
            },
            "sync": {
                "prune": false,
                "revision": ""
            }
        }
    });

    let patch_params = PatchParams::apply("kusanagi").force();
    
    apps_api
        .patch(app_name, &patch_params, &Patch::Merge(&patch))
        .await
        .map_err(|e| format!("Failed to sync application {}: {}", app_name, e))?;

    info!("Triggered sync for application: {}", app_name);

    Ok(SyncResponse {
        success: true,
        message: format!("Sync triggered for {}", app_name),
    })
}

fn calculate_error_duration(
    status: &ApplicationStatus,
    now: &DateTime<Utc>,
) -> (Option<String>, Option<String>) {
    if let Some(ref op_state) = status.operation_state {
        if let Some(ref started) = op_state.started_at {
            if let Ok(started_time) = DateTime::parse_from_rfc3339(started) {
                let duration = now.signed_duration_since(started_time.with_timezone(&Utc));
                let duration_str = format_duration(duration);
                return (Some(started.clone()), Some(duration_str));
            }
            return (Some(started.clone()), None);
        }
    }

    if let Some(ref reconciled) = status.reconciled_at {
        if let Ok(reconciled_time) = DateTime::parse_from_rfc3339(reconciled) {
            let duration = now.signed_duration_since(reconciled_time.with_timezone(&Utc));
            let duration_str = format_duration(duration);
            return (Some(reconciled.clone()), Some(duration_str));
        }
        return (Some(reconciled.clone()), None);
    }

    (None, None)
}

/// Format a duration in human-readable format
fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 0 {
        return "just now".to_string();
    }

    let days = total_seconds / 86400;
    let hours = (total_seconds % 86400) / 3600;
    let minutes = (total_seconds % 3600) / 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", total_seconds)
    }
}
