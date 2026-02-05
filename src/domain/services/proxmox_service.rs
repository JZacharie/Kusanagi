use serde_json::{json, Value};

pub async fn get_proxmox_vms() -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();
    
    if proxmox_urls.is_empty() || proxmox_user.is_empty() {
        return Ok(json!([]));
    }
    
    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    
    for url in urls {
        let url = url.trim();
        if url.is_empty() { continue; }
        
        let api_url = format!("{}/api2/json/cluster/resources?type=vm", url);
        
        match client.get(&api_url)
            .basic_auth(&proxmox_user, Some(&proxmox_password))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(items) = data["data"].as_array() {
                        let vms: Vec<Value> = items.iter().map(|vm| {
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
            _ => continue
        }
    }
    
    Ok(json!([]))
}

pub async fn get_proxmox_containers() -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();
    
    if proxmox_urls.is_empty() || proxmox_user.is_empty() {
        return Ok(json!([]));
    }
    
    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    
    for url in urls {
        let url = url.trim();
        if url.is_empty() { continue; }
        
        let api_url = format!("{}/api2/json/cluster/resources?type=lxc", url);
        
        match client.get(&api_url)
            .basic_auth(&proxmox_user, Some(&proxmox_password))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(items) = data["data"].as_array() {
                        let containers: Vec<Value> = items.iter().map(|ct| {
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
            _ => continue
        }
    }
    
    Ok(json!([]))
}

pub async fn get_proxmox_nodes() -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();
    
    if proxmox_urls.is_empty() || proxmox_user.is_empty() {
        return Ok(json!([]));
    }
    
    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    
    for url in urls {
        let url = url.trim();
        if url.is_empty() { continue; }
        
        let api_url = format!("{}/api2/json/nodes", url);
        
        match client.get(&api_url)
            .basic_auth(&proxmox_user, Some(&proxmox_password))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(items) = data["data"].as_array() {
                        let nodes: Vec<Value> = items.iter().map(|node| {
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
            _ => continue
        }
    }
    
    Ok(json!([]))
}
