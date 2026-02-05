use serde_json::{json, Value};
use std::process::Command;

pub async fn get_proxmox_vms() -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    
    for url in urls {
        let url = url.trim();
        if url.is_empty() { continue; }
        
        let api_url = format!("{}:8006/api2/json/cluster/resources?type=vm", url);
        println!("[DEBUG] Trying Proxmox URL: {}", api_url);
        let proxmox_api_output = Command::new("curl")
            .args(&["-s", "-k", &api_url, "-H", "Authorization: PVEAPIToken=USER@REALM!TOKENID=UUID"])
            .output();
    
    if let Ok(result) = proxmox_api_output {
        println!("[DEBUG] Status: {}, Output: {}", result.status, String::from_utf8_lossy(&result.stdout));
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            // Check if response is HTML error page
            if json_str.starts_with("<!DOCTYPE") || json_str.contains("Error 404") {
                println!("[DEBUG] HTML error page detected, skipping URL");
                continue; // Try next URL
            }
            if let Ok(api_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(data) = api_data["data"].as_array() {
                    let vms: Vec<Value> = data.iter().map(|vm| {
                        json!({
                            "vmid": vm["vmid"],
                            "name": vm["name"],
                            "status": vm["status"],
                            "node": vm["node"],
                            "cpu": vm["cpu"],
                            "mem": vm["mem"],
                            "maxmem": vm["maxmem"],
                            "uptime": vm["uptime"]
                        })
                    }).collect();
                    
                    return Ok(json!(vms));
                }
            }
        }
    }
    }
    
    // Fallback: essayer qm list si disponible
    let qm_output = Command::new("qm")
        .args(&["list"])
        .output();
    
    if let Ok(result) = qm_output {
        if result.status.success() {
            let output_str = String::from_utf8_lossy(&result.stdout);
            let lines: Vec<&str> = output_str.lines().skip(1).collect(); // Skip header
            
            let vms: Vec<Value> = lines.iter().filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    Some(json!({
                        "vmid": parts[0],
                        "name": parts[1],
                        "status": parts[2],
                        "source": "qm_list"
                    }))
                } else {
                    None
                }
            }).collect();
            
            return Ok(json!(vms));
        }
    }
    
    // Fallback: chercher des processus QEMU
    let qemu_output = Command::new("ps")
        .args(&["aux"])
        .output();
    
    if let Ok(result) = qemu_output {
        if result.status.success() {
            let output_str = String::from_utf8_lossy(&result.stdout);
            let qemu_count = output_str.lines()
                .filter(|line| line.contains("qemu-system") || line.contains("kvm"))
                .count();
            
            if qemu_count > 0 {
                return Ok(json!([{
                    "vmid": "detected",
                    "name": "QEMU/KVM VMs detected",
                    "status": "running",
                    "count": qemu_count,
                    "source": "process_detection"
                }]));
            }
        }
    }
    
    Ok(json!([]))
}

pub async fn get_proxmox_containers() -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    
    for url in urls {
        let url = url.trim();
        if url.is_empty() { continue; }
        
        let api_url = format!("{}:8006/api2/json/cluster/resources?type=lxc", url);
        println!("[DEBUG] Trying Proxmox containers URL: {}", api_url);
        let proxmox_api_output = Command::new("curl")
            .args(&["-s", "-k", &api_url, "-H", "Authorization: PVEAPIToken=USER@REALM!TOKENID=UUID"])
            .output();
    
    if let Ok(result) = proxmox_api_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if json_str.starts_with("<!DOCTYPE") || json_str.contains("Error 404") {
                continue;
            }
            if let Ok(api_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(data) = api_data["data"].as_array() {
                    let containers: Vec<Value> = data.iter().map(|ct| {
                        json!({
                            "vmid": ct["vmid"],
                            "name": ct["name"],
                            "status": ct["status"],
                            "node": ct["node"],
                            "cpu": ct["cpu"],
                            "mem": ct["mem"],
                            "maxmem": ct["maxmem"],
                            "uptime": ct["uptime"]
                        })
                    }).collect();
                    
                    return Ok(json!(containers));
                }
            }
        }
    }
    }
    
    // Fallback: essayer pct list si disponible
    let pct_output = Command::new("pct")
        .args(&["list"])
        .output();
    
    if let Ok(result) = pct_output {
        if result.status.success() {
            let output_str = String::from_utf8_lossy(&result.stdout);
            let lines: Vec<&str> = output_str.lines().skip(1).collect(); // Skip header
            
            let containers: Vec<Value> = lines.iter().filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    Some(json!({
                        "vmid": parts[0],
                        "name": parts[1],
                        "status": parts[2],
                        "source": "pct_list"
                    }))
                } else {
                    None
                }
            }).collect();
            
            return Ok(json!(containers));
        }
    }
    
    // Fallback: chercher des containers LXC
    let lxc_output = Command::new("lxc-ls")
        .args(&["-f"])
        .output();
    
    if let Ok(result) = lxc_output {
        if result.status.success() {
            let output_str = String::from_utf8_lossy(&result.stdout);
            let lines: Vec<&str> = output_str.lines().skip(1).collect();
            
            let containers: Vec<Value> = lines.iter().filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(json!({
                        "name": parts[0],
                        "status": parts[1],
                        "source": "lxc_ls"
                    }))
                } else {
                    None
                }
            }).collect();
            
            return Ok(json!(containers));
        }
    }
    
    Ok(json!([]))
}

pub async fn get_proxmox_nodes() -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    
    for url in urls {
        let url = url.trim();
        if url.is_empty() { continue; }
        
        let api_url = format!("{}:8006/api2/json/nodes", url);
        println!("[DEBUG] Trying Proxmox nodes URL: {}", api_url);
        let proxmox_api_output = Command::new("curl")
            .args(&["-s", "-k", &api_url, "-H", "Authorization: PVEAPIToken=USER@REALM!TOKENID=UUID"])
            .output();
    
    if let Ok(result) = proxmox_api_output {
        if result.status.success() {
            let json_str = String::from_utf8_lossy(&result.stdout);
            if json_str.starts_with("<!DOCTYPE") || json_str.contains("Error 404") {
                continue;
            }
            if let Ok(api_data) = serde_json::from_str::<Value>(&json_str) {
                if let Some(data) = api_data["data"].as_array() {
                    let nodes: Vec<Value> = data.iter().map(|node| {
                        json!({
                            "node": node["node"],
                            "status": node["status"],
                            "cpu": node["cpu"],
                            "mem": node["mem"],
                            "maxmem": node["maxmem"],
                            "disk": node["disk"],
                            "maxdisk": node["maxdisk"],
                            "uptime": node["uptime"]
                        })
                    }).collect();
                    
                    return Ok(json!(nodes));
                }
            }
        }
    }
    }
    
    // Fallback: essayer pvecm status si disponible
    let pvecm_output = Command::new("pvecm")
        .args(&["status"])
        .output();
    
    if let Ok(result) = pvecm_output {
        if result.status.success() {
            let output_str = String::from_utf8_lossy(&result.stdout);
            if output_str.contains("Cluster information") {
                return Ok(json!([{
                    "node": "local",
                    "status": "online",
                    "source": "pvecm_status",
                    "cluster_detected": true
                }]));
            }
        }
    }
    
    // Fallback: vérifier si on est sur un système Proxmox
    let pve_version_output = Command::new("pveversion")
        .output();
    
    if let Ok(result) = pve_version_output {
        if result.status.success() {
            let output_str = String::from_utf8_lossy(&result.stdout);
            return Ok(json!([{
                "node": "localhost",
                "status": "online",
                "version": output_str.trim(),
                "source": "pveversion"
            }]));
        }
    }
    
    // Fallback final: pas de Proxmox détecté
    Ok(json!([]))
}
