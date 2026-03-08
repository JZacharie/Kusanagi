use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{api::ListParams, Api, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::AdvancedCache;

/// ArgoCD Application resource (CRD)
/// Simplified struct matching the ArgoCD Application CRD
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArgoCDApplication {
    #[serde(rename = "apiVersion", default)]
    pub api_version: String,
    #[serde(rename = "kind", default)]
    pub kind: String,
    pub metadata: ObjectMeta,
    pub status: Option<ArgoCDApplicationStatus>,
}

impl k8s_openapi::Resource for ArgoCDApplication {
    const GROUP: &'static str = "argoproj.io";
    const VERSION: &'static str = "v1alpha1";
    const KIND: &'static str = "Application";
    const API_VERSION: &'static str = "argoproj.io/v1alpha1";
    const URL_PATH_SEGMENT: &'static str = "applications";
    type Scope = k8s_openapi::NamespaceResourceScope;
}

impl k8s_openapi::Metadata for ArgoCDApplication {
    type Ty = ObjectMeta;

    fn metadata(&self) -> &Self::Ty {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut Self::Ty {
        &mut self.metadata
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArgoCDApplicationStatus {
    pub health: Option<HealthStatus>,
    pub sync: Option<SyncStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HealthStatus {
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyncStatus {
    pub status: Option<String>,
    pub revision: Option<String>,
}

pub async fn get_argocd_status(cache: &AdvancedCache<String>) -> Result<Value, String> {
    let cache_key = "argocd_status";

    // Try cache first
    if let Some(cached_json) = cache.get(cache_key).await {
        if let Ok(json) = serde_json::from_str::<Value>(&cached_json) {
            return Ok(json);
        }
    }

    let result = fetch_argocd_status_from_cluster().await;

    if let Ok(ref json) = result {
        if let Ok(s) = serde_json::to_string(json) {
            cache
                .set(
                    cache_key.to_string(),
                    s,
                    Some(std::time::Duration::from_secs(300)),
                )
                .await;
        }
    }

    result
}

async fn fetch_argocd_status_from_cluster() -> Result<Value, String> {
    // Create kube client
    let client = match Client::try_default().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("⚠️ Failed to create Kubernetes client: {}", e);
            return Err("Kubernetes client not available".to_string());
        }
    };

    // Try to get ArgoCD applications from namespace argocd
    let apps_api: Api<ArgoCDApplication> = Api::namespaced(client.clone(), "argocd");

    match tokio::time::timeout(
        std::time::Duration::from_secs(20),
        apps_api.list(&ListParams::default()),
    )
    .await
    {
        Ok(Ok(app_list)) => {
            tracing::info!("✅ Found {} ArgoCD applications", app_list.items.len());
            return parse_argocd_applications(&app_list.items);
        }
        Ok(Err(e)) => {
            tracing::warn!("⚠️ Failed to list ArgoCD applications: {}", e);
        }
        Err(_) => {
            tracing::warn!("⚠️ Timeout listing ArgoCD applications");
        }
    }

    // Check if ArgoCD is installed by checking for pods in argocd namespace
    let pods_api: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(client, "argocd");

    if let Ok(Ok(pod_list)) = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        pods_api.list(&ListParams::default()),
    )
    .await
    {
        let total_pods = pod_list.items.len();
        if total_pods > 0 {
            let running_pods = pod_list
                .items
                .iter()
                .filter(|p| p.status.as_ref().and_then(|s| s.phase.as_deref()) == Some("Running"))
                .count();

            tracing::warn!(
                "⚠️ ArgoCD installed ({}/{} pods running) but applications API not accessible",
                running_pods,
                total_pods
            );
            return Ok(json!({
                "total": 0,
                "healthy": 0,
                "unhealthy": 0,
                "synced": 0,
                "out_of_sync": 0,
                "progressing": 0,
                "upgrades_available": 0,
                "apps_with_issues": [],
                "apps_with_upgrades": [],
                "message": format!("ArgoCD installed ({}/{} pods) but applications not accessible - check permissions", running_pods, total_pods)
            }));
        }
    }

    tracing::warn!("⚠️ ArgoCD not detected in cluster");
    Err("ArgoCD not detected or not accessible".to_string())
}

fn parse_argocd_applications(items: &[ArgoCDApplication]) -> Result<Value, String> {
    let mut healthy = 0;
    let mut degraded = 0;
    let mut progressing = 0;
    let mut suspended = 0;
    let mut missing = 0;
    let mut unknown = 0;
    let mut synced = 0;
    let mut out_of_sync = 0;

    let mut apps_with_issues = Vec::new();
    let mut apps_with_upgrades = Vec::new();
    let total = items.len();

    for item in items {
        let name = item.metadata.name.as_deref().unwrap_or("unknown");
        let namespace = item.metadata.namespace.as_deref().unwrap_or("argocd");

        let health_status = item
            .status
            .as_ref()
            .and_then(|s| s.health.as_ref())
            .and_then(|h| h.status.as_deref())
            .unwrap_or("Unknown");

        let sync_status = item
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .and_then(|sy| sy.status.as_deref())
            .unwrap_or("Unknown");

        let revision = item
            .status
            .as_ref()
            .and_then(|s| s.sync.as_ref())
            .and_then(|sy| sy.revision.as_deref())
            .unwrap_or("");

        match health_status {
            "Healthy" => healthy += 1,
            "Degraded" => degraded += 1,
            "Progressing" => progressing += 1,
            "Suspended" => suspended += 1,
            "Missing" => missing += 1,
            _ => unknown += 1,
        }

        match sync_status {
            "Synced" => synced += 1,
            "OutOfSync" => out_of_sync += 1,
            _ => {}
        }

        let app_obj = json!({
            "name": name,
            "namespace": namespace,
            "health_status": health_status,
            "sync_status": sync_status,
            "current_revision": revision,
            "argocd_url": format!("https://argocd.p.zacharie.org/applications/{}", name),
            "message": "",
            "can_sync": sync_status == "OutOfSync"
        });

        if health_status != "Healthy" || sync_status == "OutOfSync" {
            apps_with_issues.push(app_obj.clone());
        }

        if sync_status == "OutOfSync" && health_status == "Healthy" {
            apps_with_upgrades.push(app_obj);
        }
    }

    Ok(json!({
        "total": total,
        "healthy": healthy,
        "degraded": degraded,
        "progressing": progressing,
        "suspended": suspended,
        "missing": missing,
        "unknown": unknown,
        "synced": synced,
        "out_of_sync": out_of_sync,
        "upgrades_available": apps_with_upgrades.len(),
        "apps_with_issues": apps_with_issues,
        "apps_with_upgrades": apps_with_upgrades
    }))
}

pub async fn sync_app(app_name: &str) -> Result<String, String> {
    tracing::info!("🔄 Triggering sync for ArgoCD app: {}", app_name);

    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let apps_api: Api<ArgoCDApplication> = Api::namespaced(client, "argocd");

    // Patch the application to trigger a sync
    let patch = serde_json::json!({
        "operation": {
            "sync": {
                "prune": true
            }
        }
    });

    let patch_params = kube::api::PatchParams::apply("kusanagi");

    match apps_api
        .patch(app_name, &patch_params, &kube::api::Patch::Merge(&patch))
        .await
    {
        Ok(_) => Ok(format!("Sync triggered for {}", app_name)),
        Err(e) => Err(format!("Sync failed: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_argocd_applications() {
        let items = vec![
            ArgoCDApplication {
                api_version: "argoproj.io/v1alpha1".to_string(),
                kind: "Application".to_string(),
                metadata: ObjectMeta {
                    name: Some("app-healthy".to_string()),
                    namespace: Some("argocd".to_string()),
                    ..Default::default()
                },
                status: Some(ArgoCDApplicationStatus {
                    health: Some(HealthStatus {
                        status: Some("Healthy".to_string()),
                    }),
                    sync: Some(SyncStatus {
                        status: Some("Synced".to_string()),
                        revision: Some("rev1".to_string()),
                    }),
                }),
            },
            ArgoCDApplication {
                api_version: "argoproj.io/v1alpha1".to_string(),
                kind: "Application".to_string(),
                metadata: ObjectMeta {
                    name: Some("app-degraded".to_string()),
                    namespace: Some("argocd".to_string()),
                    ..Default::default()
                },
                status: Some(ArgoCDApplicationStatus {
                    health: Some(HealthStatus {
                        status: Some("Degraded".to_string()),
                    }),
                    sync: Some(SyncStatus {
                        status: Some("OutOfSync".to_string()),
                        revision: Some("rev2".to_string()),
                    }),
                }),
            },
        ];

        let result = parse_argocd_applications(&items).expect("Failed to parse");

        assert_eq!(result["total"], 2);
        assert_eq!(result["healthy"], 1);
        assert_eq!(result["degraded"], 1);
        assert_eq!(result["synced"], 1);
        assert_eq!(result["out_of_sync"], 1);
        assert_eq!(result["apps_with_issues"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_parse_empty_list() {
        let items: Vec<ArgoCDApplication> = vec![];
        let result = parse_argocd_applications(&items).expect("Failed to parse");
        assert_eq!(result["total"], 0);
    }
}
