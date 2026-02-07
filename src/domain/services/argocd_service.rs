use serde_json::{json, Value};
use tokio::process::Command;

pub async fn get_argocd_status() -> Result<Value, String> {
    tracing::info!("🔍 Fetching ArgoCD status...");
    
    // OPTIMIZATION: Use custom-columns to avoid parsing massive JSON with history/managedFields
    // Columns: NAME, NAMESPACE, HEALTH, SYNC, REVISION
    let kubectl_output = Command::new("kubectl")
        .args(&["get", "applications", "-n", "argocd", "--no-headers", 
                "-o", "custom-columns=NAME:.metadata.name,NS:.metadata.namespace,HEALTH:.status.health.status,SYNC:.status.sync.status,REV:.status.sync.revision"])
        .output()
        .await;
    
    if let Ok(result) = kubectl_output {
        if result.status.success() {
            let stdout = String::from_utf8_lossy(&result.stdout);
            return parse_argocd_apps_text(&stdout);
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr);
            tracing::warn!("⚠️ ArgoCD kubectl error: {}", stderr.trim());
        }
    }
    
    // Check if ArgoCD is installed if app fetch failed
    let argocd_pods_output = Command::new("kubectl")
        .args(&["get", "pods", "-n", "argocd", "--no-headers"])
        .output()
        .await;
    
    if let Ok(result) = argocd_pods_output {
        if result.status.success() {
            let pods_output = String::from_utf8_lossy(&result.stdout);
            let pod_lines: Vec<&str> = pods_output.lines().filter(|line| !line.trim().is_empty()).collect();
            
            if !pod_lines.is_empty() {
                let running_pods = pod_lines.iter()
                    .filter(|line| line.contains("Running"))
                    .count();
                
                tracing::warn!("⚠️ ArgoCD installed ({}/{} pods running) but no apps found", running_pods, pod_lines.len());
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
                    "message": format!("ArgoCD installed ({}/{} pods running) but no apps found", running_pods, pod_lines.len())
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

fn parse_argocd_apps_text(stdout: &str) -> Result<Value, String> {
    let mut healthy = 0;
    let mut unhealthy = 0;
    let mut synced = 0;
    let mut out_of_sync = 0;
    let mut progressing = 0;
    let mut apps_with_issues = Vec::new();
    let mut apps_with_upgrades = Vec::new();
    let mut total = 0;
    
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Expecting at least 4 columns: NAME, NS, HEALTH, SYNC. REV is optional/might be empty but split_whitespace handles it.
        if parts.len() < 4 { continue; }
        
        total += 1;
        let name = parts[0];
        let namespace = parts[1];
        let health_status = parts[2];
        let sync_status = parts[3];
        let revision = if parts.len() > 4 { parts[4] } else { "" };
        
        match health_status {
            "Healthy" => healthy += 1,
            "Degraded" | "Missing" | "Unknown" => unhealthy += 1,
            "Progressing" => progressing += 1,
            _ => {} // Handle custom statuses if any
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
            "message": "", // Omitted for performance/safety with custom-columns
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
