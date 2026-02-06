use serde_json::{json, Value};

async fn get_proxmox_ticket(client: &reqwest::Client, url: &str, user: &str, password: &str) -> Option<(String, String)> {
    let auth_url = format!("{}/api2/json/access/ticket", url);
    
    // Handle username that might already have @pam
    let username = if user.contains('@') {
        user.to_string()
    } else {
        format!("{}@pam", user)
    };
    
    let params = [("username", username), ("password", password.to_string())];
    
    eprintln!("🔐 Proxmox Auth: Attempting login to {} as {}", url, params[0].1);
    
    match client.post(&auth_url).form(&params).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(data) = response.json::<Value>().await {
                    let ticket = data["data"]["ticket"].as_str().map(|s| s.to_string());
                    let csrf = data["data"]["CSRFPreventionToken"].as_str().map(|s| s.to_string());
                    
                    if let (Some(t), Some(c)) = (ticket, csrf) {
                        return Some((t, c));
                    } else {
                        eprintln!("❌ Proxmox Auth: {} - Missing ticket or CSRF in response: {:?}", url, data);
                    }
                } else {
                    eprintln!("❌ Proxmox Auth: {} - Failed to parse JSON response", url);
                }
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_else(|_| "no body".to_string());
                eprintln!("❌ Proxmox Auth: {} - Failed with status {}. Body: {}", url, status, body);
            }
        }
        Err(e) => eprintln!("❌ Proxmox Auth: {} - Network error: {}", url, e)
    }
    None
}

pub async fn get_proxmox_vms(client: &reqwest::Client) -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();
    
    if proxmox_urls.is_empty() || proxmox_user.is_empty() {
        eprintln!("⚠️ Proxmox VMs: Missing credentials");
        return Ok(json!([]));
    }
    
    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    
    let mut all_vms = Vec::new();

    for url in urls {
        let url = url.trim();
        if url.is_empty() { continue; }
        
        let Some((ticket, _csrf)) = get_proxmox_ticket(client, url, &proxmox_user, &proxmox_password).await else {
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
                        all_vms.extend(vms);
                    }
                }
            }
            Ok(response) => eprintln!("⚠️ Proxmox VMs: {} status {}", url, response.status()),
            Err(e) => eprintln!("❌ Proxmox VMs: {} error: {}", url, e)
        }
    }
    
    Ok(json!(all_vms))
}

pub async fn get_proxmox_containers(client: &reqwest::Client) -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();
    
    if proxmox_urls.is_empty() || proxmox_user.is_empty() {
        return Ok(json!([]));
    }
    
    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    
    let mut all_containers = Vec::new();

    for url in urls {
        let url = url.trim();
        if url.is_empty() { continue; }
        
        let Some((ticket, _csrf)) = get_proxmox_ticket(client, url, &proxmox_user, &proxmox_password).await else {
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
                        all_containers.extend(containers);
                    }
                }
            }
            _ => continue
        }
    }
    
    Ok(json!(all_containers))
}

pub async fn get_proxmox_nodes(client: &reqwest::Client) -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();
    
    if proxmox_urls.is_empty() || proxmox_user.is_empty() {
        return Ok(json!([]));
    }
    
    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    
    let mut all_nodes = Vec::new();

    for url in urls {
        let url = url.trim();
        if url.is_empty() { continue; }
        
        let Some((ticket, _csrf)) = get_proxmox_ticket(client, url, &proxmox_user, &proxmox_password).await else {
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
                                "uptime": node["uptime"],
                                "server": url
                            })
                        }).collect();
                        eprintln!("✅ Proxmox Nodes: Found {} from {}", nodes.len(), url);
                        all_nodes.extend(nodes);
                    }
                }
            }
            _ => continue
        }
    }
    
    Ok(json!(all_nodes))
}
