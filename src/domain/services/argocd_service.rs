use serde_json::{json, Value};
use tokio::process::Command;

pub async fn get_argocd_status() -> Result<Value, String> {
    tracing::info!("🔍 Fetching ArgoCD status (JSON mode)...");

    let kubectl_output = Command::new("kubectl")
        .args(["get", "applications", "-n", "argocd", "-o", "json"])
        .output()
        .await;

    let applications_error = if let Ok(result) = kubectl_output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            tracing::debug!("✅ kubectl output length: {}", stdout.len());
            return parse_argocd_apps_json(&stdout);
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr);
            tracing::error!("❌ ArgoCD kubectl FAILED. Stderr: {}", stderr.trim());
            format!("kubectl failed: {}", stderr.trim())
        }
    } else {
        tracing::error!("❌ Failed to execute kubectl command");
        "Failed to execute kubectl command".to_string()
    };

    // Check if ArgoCD is installed if app fetch failed
    let argocd_pods_output = Command::new("kubectl")
        .args(["get", "pods", "-n", "argocd", "--no-headers"])
        .output()
        .await;

    if let Ok(result) = argocd_pods_output {
        if result.status.success() {
            let pods_output = String::from_utf8_lossy(&result.stdout);
            let pod_lines: Vec<&str> = pods_output
                .lines()
                .filter(|line| !line.trim().is_empty())
                .collect();

            if !pod_lines.is_empty() {
                let running_pods = pod_lines
                    .iter()
                    .filter(|line| line.contains("Running"))
                    .count();

                tracing::warn!(
                    "⚠️ ArgoCD installed ({}/{} pods running) but no apps found (or fetch failed)",
                    running_pods,
                    pod_lines.len()
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
                    "message": format!("ArgoCD installed ({}/{} pods running) but app fetch failed: {}", running_pods, pod_lines.len(), applications_error)
                }));
            }
        }
    }

    tracing::error!("❌ ArgoCD not detected or not accessible");
    Ok(json!({
        "total": 0,
        "healthy": 0,
        "unhealthy": 0,
        "synced": 0,
        "out_of_sync": 0,
        "progressing": 0,
        "upgrades_available": 0,
        "apps_with_issues": [],
        "apps_with_upgrades": [],
        "error": "ArgoCD not detected or not accessible"
    }))
}

fn parse_argocd_apps_json(json_str: &str) -> Result<Value, String> {
    let root: Value =
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let items = root["items"]
        .as_array()
        .ok_or("No items found in JSON response")?;

    if items.is_empty() {
        tracing::warn!("⚠️ ArgoCD JSON parsed successfully but 'items' array is empty.");
    } else {
        tracing::info!("✅ Found {} ArgoCD applications in JSON", items.len());
    }

    let mut healthy = 0;
    let mut unhealthy = 0;
    let mut synced = 0;
    let mut out_of_sync = 0;
    let mut progressing = 0;
    let mut apps_with_issues = Vec::new();
    let mut apps_with_upgrades = Vec::new();
    let total = items.len();

    for item in items {
        let name = item["metadata"]["name"].as_str().unwrap_or("unknown");
        let namespace = item["metadata"]["namespace"].as_str().unwrap_or("argocd");

        let health_status = item
            .pointer("/status/health/status")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let sync_status = item
            .pointer("/status/sync/status")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let revision = item
            .pointer("/status/sync/revision")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match health_status {
            "Healthy" => healthy += 1,
            "Degraded" | "Missing" | "Unknown" => unhealthy += 1,
            "Progressing" => progressing += 1,
            _ => {}
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
        "unhealthy": unhealthy,
        "synced": synced,
        "out_of_sync": out_of_sync,
        "progressing": progressing,
        "upgrades_available": apps_with_upgrades.len(),
        "apps_with_issues": apps_with_issues,
        "apps_with_upgrades": apps_with_upgrades
    }))
}

pub async fn sync_app(app_name: &str) -> Result<String, String> {
    tracing::info!("🔄 Triggering sync for ArgoCD app: {}", app_name);

    let output = Command::new("kubectl")
        .args([
            "patch",
            "application",
            app_name,
            "-n",
            "argocd",
            "--type",
            "merge",
            "-p",
            "{\"operation\": {\"sync\": {\"prune\": true}}}",
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute kubectl: {}", e))?;

    if output.status.success() {
        Ok(format!("Sync triggered for {}", app_name))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Sync failed: {}", stderr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_argocd_apps_json() {
        let json_data = r#"{
            "apiVersion": "v1",
            "items": [
                {
                    "metadata": {
                        "name": "app-healthy",
                        "namespace": "argocd"
                    },
                    "status": {
                        "health": { "status": "Healthy" },
                        "sync": { "status": "Synced", "revision": "rev1" }
                    }
                },
                {
                    "metadata": {
                        "name": "app-degraded",
                        "namespace": "argocd"
                    },
                    "status": {
                        "health": { "status": "Degraded" },
                        "sync": { "status": "OutOfSync", "revision": "rev2" }
                    }
                }
            ]
        }"#;

        let result = parse_argocd_apps_json(json_data).expect("Failed to parse JSON");

        assert_eq!(result["total"], 2);
        assert_eq!(result["healthy"], 1);
        assert_eq!(result["unhealthy"], 1);
        assert_eq!(result["synced"], 1);
        assert_eq!(result["out_of_sync"], 1);
        assert_eq!(result["apps_with_issues"].as_array().unwrap().len(), 1); // app-degraded
        assert_eq!(result["apps_with_issues"][0]["name"], "app-degraded");
    }

    #[test]
    fn test_parse_argocd_empty_list() {
        let json_data = r#"{
            "apiVersion": "v1",
            "items": []
        }"#;
        let result = parse_argocd_apps_json(json_data).expect("Failed to parse JSON");
        assert_eq!(result["total"], 0);
    }
}
