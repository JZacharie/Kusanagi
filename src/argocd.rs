use chrono::{DateTime, Utc};
use kube::{
    api::{Api, ListParams},
    Client,
};
use serde::{Deserialize, Serialize};
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub status: Option<String>,
    pub revision: Option<String>,
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
}

/// Application List response
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationList {
    pub items: Vec<Application>,
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
    pub apps_with_issues: Vec<AppIssue>,
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
        apps_with_issues: Vec::new(),
    };

    let now = Utc::now();

    for app in app_list.items {
        let name = app.metadata.name.clone().unwrap_or_default();
        
        // Extract status from dynamic object data
        let status: ApplicationStatus = app.data.get("status")
            .and_then(|s| serde_json::from_value(s.clone()).ok())
            .unwrap_or_default();

        let dest_namespace = app.data.get("spec")
            .and_then(|s| s.get("destination"))
            .and_then(|d| d.get("namespace"))
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();

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

            // Try to determine when the error started
            let (error_since, error_duration) = calculate_error_duration(&status, &now);

            response.apps_with_issues.push(AppIssue {
                name,
                namespace: dest_namespace,
                health_status,
                sync_status,
                message,
                error_since,
                error_duration,
            });
        }
    }

    info!(
        "ArgoCD status: {} total, {} healthy, {} with issues",
        response.total,
        response.healthy,
        response.apps_with_issues.len()
    );

    Ok(response)
}

fn calculate_error_duration(status: &ApplicationStatus, now: &DateTime<Utc>) -> (Option<String>, Option<String>) {
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
