use serde_json::{json, Value};
use std::process::Command;

pub async fn get_argocd_status() -> Result<Value, String> {
    eprintln!("🔍 Fetching ArgoCD status...");
    
    // Try kubectl for ArgoCD applications
    let kubectl_output = Command::new("kubectl")
        .args(&["get", "applications", "-n", "argocd", "-o", "json"])
        .output();
    
    if let Ok(result) = kubectl_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(apps_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(items) = apps_data["items"].as_array() {
                    eprintln!("✅ ArgoCD: Found {} applications via kubectl", items.len());
                    return parse_argocd_apps(items);
                }
            }
        } else {
            let stderr = String::from_utf8_lossy(&result.stderr);
            eprintln!("⚠️ ArgoCD kubectl error: {}", stderr.trim());
        }
    }
    
    // Check if ArgoCD is installed
    let argocd_pods_output = Command::new("kubectl")
        .args(&["get", "pods", "-n", "argocd", "--no-headers"])
        .output();
    
    if let Ok(result) = argocd_pods_output {
        if result.status.success() {
            let pods_output = String::from_utf8_lossy(&result.stdout);
            let pod_lines: Vec<&str> = pods_output.lines().filter(|line| !line.trim().is_empty()).collect();
            
            if !pod_lines.is_empty() {
                let running_pods = pod_lines.iter()
                    .filter(|line| line.contains("Running"))
                    .count();
                
                eprintln!("⚠️ ArgoCD installed ({}/{} pods running) but no apps found", running_pods, pod_lines.len());
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
    
    eprintln!("❌ ArgoCD not detected or not accessible");
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

fn parse_argocd_apps(items: &Vec<Value>) -> Result<Value, String> {
    let mut healthy = 0;
    let mut unhealthy = 0;
    let mut synced = 0;
    let mut out_of_sync = 0;
    let mut progressing = 0;
    let mut apps_with_issues = Vec::new();
    let mut apps_with_upgrades = Vec::new();
    
    for app in items {
        let name = app["metadata"]["name"].as_str().unwrap_or("unknown");
        let namespace = app["metadata"]["namespace"].as_str().unwrap_or("argocd");
        let health_status = app["status"]["health"]["status"].as_str().unwrap_or("Unknown");
        let sync_status = app["status"]["sync"]["status"].as_str().unwrap_or("Unknown");
        let revision = app["status"]["sync"]["revision"].as_str().unwrap_or("");
        
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
            "message": app["status"]["health"]["message"].as_str().unwrap_or(""),
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
        "total": items.len(),
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
