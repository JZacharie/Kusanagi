use serde_json::{json, Value};

async fn get_proxmox_ticket(client: &reqwest::Client, url: &str, user: &str, password: &str) -> Option<(String, String)> {
    let auth_url = format!("{}/api2/json/access/ticket", url);
    let params = [("username", format!("{}@pam", user)), ("password", password.to_string())];
    
    match client.post(&auth_url).form(&params).send().await {
        Ok(response) if response.status().is_success() => {
            if let Ok(data) = response.json::<Value>().await {
                let ticket = data["data"]["ticket"].as_str()?.to_string();
                let csrf = data["data"]["CSRFPreventionToken"].as_str()?.to_string();
                return Some((ticket, csrf));
            }
        }
        _ => {}
    }
    None
}

pub async fn get_proxmox_vms() -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();
    
    if proxmox_urls.is_empty() || proxmox_user.is_empty() {
        eprintln!("⚠️ Proxmox VMs: Missing credentials");
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
        
        let Some((ticket, _csrf)) = get_proxmox_ticket(&client, url, &proxmox_user, &proxmox_password).await else {
            eprintln!("⚠️ Proxmox VMs: {} auth failed", url);
            continue;
        };
        
        let api_url = format!("{}/api2/json/cluster/resources?type=vm", url);
        
        match client.get(&api_url).header("Cookie", format!("PVEAuthCookie={}", ticket)).send().await {
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
                                "uptime": vm["uptime"],
                                "server": url
                            })
                        }).collect();
                        eprintln!("✅ Proxmox VMs: Found {} from {}", vms.len(), url);
                        return Ok(json!(vms));
                    }
                }
            }
            Ok(response) => eprintln!("⚠️ Proxmox VMs: {} status {}", url, response.status()),
            Err(e) => eprintln!("❌ Proxmox VMs: {} error: {}", url, e)
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
        
        let Some((ticket, _csrf)) = get_proxmox_ticket(&client, url, &proxmox_user, &proxmox_password).await else {
            continue;
        };
        
        let api_url = format!("{}/api2/json/cluster/resources?type=lxc", url);
        
        match client.get(&api_url).header("Cookie", format!("PVEAuthCookie={}", ticket)).send().await {
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
                                "uptime": ct["uptime"],
                                "server": url
                            })
                        }).collect();
                        eprintln!("✅ Proxmox Containers: Found {} from {}", containers.len(), url);
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
        
        let Some((ticket, _csrf)) = get_proxmox_ticket(&client, url, &proxmox_user, &proxmox_password).await else {
            continue;
        };
        
        let api_url = format!("{}/api2/json/nodes", url);
        
        match client.get(&api_url).header("Cookie", format!("PVEAuthCookie={}", ticket)).send().await {
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
                        eprintln!("✅ Proxmox Nodes: Found {} from {}", nodes.len(), url);
                        return Ok(json!(nodes));
                    }
                }
            }
            _ => continue
        }
    }
    
    Ok(json!([]))
}
