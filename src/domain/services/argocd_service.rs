use serde_json::{json, Value};
use std::process::Command;

pub async fn get_argocd_status() -> Result<Value, String> {
    // Essayer l'API ArgoCD sur le port standard
    let argocd_api_output = Command::new("curl")
        .args(&["-s", "-k", "http://localhost:8081/api/v1/applications", "-H", "Accept: application/json"])
        .output();
    
    if let Ok(result) = argocd_api_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(apps_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(items) = apps_data["items"].as_array() {
                    let mut healthy_count = 0;
                    let mut synced_count = 0;
                    let total_count = items.len();
                    
                    for app in items {
                        let health = app["status"]["health"]["status"].as_str().unwrap_or("Unknown");
                        let sync = app["status"]["sync"]["status"].as_str().unwrap_or("Unknown");
                        
                        if health == "Healthy" { healthy_count += 1; }
                        if sync == "Synced" { synced_count += 1; }
                    }
                    
                    return Ok(json!({
                        "healthy": healthy_count == total_count && total_count > 0,
                        "apps": total_count,
                        "healthy_apps": healthy_count,
                        "synced_apps": synced_count,
                        "source": "argocd_api"
                    }));
                }
            }
        }
    }
    
    // Fallback: essayer kubectl pour ArgoCD
    let kubectl_output = Command::new("kubectl")
        .args(&["get", "applications", "-n", "argocd", "-o", "json"])
        .output();
    
    if let Ok(result) = kubectl_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if let Ok(apps_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(items) = apps_data["items"].as_array() {
                    let mut healthy_count = 0;
                    let total_count = items.len();
                    
                    for app in items {
                        if let Some(health) = app["status"]["health"]["status"].as_str() {
                            if health == "Healthy" { healthy_count += 1; }
                        }
                    }
                    
                    return Ok(json!({
                        "healthy": healthy_count == total_count && total_count > 0,
                        "apps": total_count,
                        "healthy_apps": healthy_count,
                        "synced_apps": healthy_count,
                        "source": "kubectl"
                    }));
                }
            }
        }
    }
    
    // Fallback: vérifier si ArgoCD est installé
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
                
                return Ok(json!({
                    "healthy": running_pods == pod_lines.len(),
                    "apps": 0,
                    "healthy_apps": 0,
                    "synced_apps": 0,
                    "argocd_pods": pod_lines.len(),
                    "running_pods": running_pods,
                    "source": "pods_check"
                }));
            }
        }
    }
    
    // Fallback final: ArgoCD non détecté
    Ok(json!({
        "healthy": false,
        "apps": 0,
        "healthy_apps": 0,
        "synced_apps": 0,
        "message": "ArgoCD not detected or not accessible"
    }))
}
