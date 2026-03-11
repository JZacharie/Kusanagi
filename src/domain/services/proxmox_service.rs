use serde_json::{json, Value};
use tracing::{error, info, warn};

async fn get_proxmox_ticket(
    client: &reqwest::Client,
    url: &str,
    user: &str,
    password: &str,
) -> Option<(String, String)> {
    let auth_url = format!("{}/api2/json/access/ticket", url);

    // Handle username that might already have @pam
    let username = if user.contains('@') {
        user.to_string()
    } else {
        format!("{}@pam", user)
    };

    let params = [("username", username), ("password", password.to_string())];

    info!(
        "🔐 Proxmox Auth: Attempting login to {} as {}",
        url, params[0].1
    );

    match client.post(&auth_url).form(&params).send().await {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(data) = response.json::<Value>().await {
                    let ticket = data["data"]["ticket"].as_str().map(|s| s.to_string());
                    let csrf = data["data"]["CSRFPreventionToken"]
                        .as_str()
                        .map(|s| s.to_string());

                    if let (Some(t), Some(c)) = (ticket, csrf) {
                        return Some((t, c));
                    } else {
                        error!(
                            "❌ Proxmox Auth: {} - Missing ticket or CSRF in response: {:?}",
                            url, data
                        );
                    }
                } else {
                    error!("❌ Proxmox Auth: {} - Failed to parse JSON response", url);
                }
            } else {
                let status = response.status();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "no body".to_string());
                error!(
                    "❌ Proxmox Auth: {} - Failed with status {}. Body: {}",
                    url, status, body
                );
            }
        }
        Err(e) => error!("❌ Proxmox Auth: {} - Network error: {}", url, e),
    }
    None
}

pub async fn get_proxmox_vms(client: &reqwest::Client) -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();

    if proxmox_urls.is_empty() || proxmox_user.is_empty() {
        warn!("⚠️ Proxmox VMs: Missing credentials");
        return Ok(json!([]));
    }

    let urls: Vec<&str> = proxmox_urls.split(',').collect();

    let mut all_vms = Vec::new();

    for url in urls {
        let url = url.trim();
        if url.is_empty() {
            continue;
        }

        let Some((ticket, _csrf)) =
            get_proxmox_ticket(client, url, &proxmox_user, &proxmox_password).await
        else {
            warn!("⚠️ Proxmox VMs: {} auth failed", url);
            continue;
        };

        let api_url = format!("{}/api2/json/cluster/resources?type=vm", url);

        match client
            .get(&api_url)
            .header("Cookie", format!("PVEAuthCookie={}", ticket))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(items) = data["data"].as_array() {
                        let vms: Vec<Value> = items
                            .iter()
                            .filter(|vm| vm["type"].as_str() == Some("qemu"))
                            .map(|vm| {
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
                            })
                            .collect();
                        info!("✅ Proxmox VMs: Found {} from {}", vms.len(), url);
                        all_vms.extend(vms);
                    }
                }
            }
            Ok(response) => warn!("⚠️ Proxmox VMs: {} status {}", url, response.status()),
            Err(e) => error!("❌ Proxmox VMs: {} error: {}", url, e),
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
        if url.is_empty() {
            continue;
        }

        let Some((ticket, _csrf)) =
            get_proxmox_ticket(client, url, &proxmox_user, &proxmox_password).await
        else {
            continue;
        };

        let api_url = format!("{}/api2/json/cluster/resources?type=vm", url);

        match client
            .get(&api_url)
            .header("Cookie", format!("PVEAuthCookie={}", ticket))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(items) = data["data"].as_array() {
                        let containers: Vec<Value> = items
                            .iter()
                            .filter(|ct| ct["type"].as_str() == Some("lxc"))
                            .map(|ct| {
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
                            })
                            .collect();
                        info!(
                            "✅ Proxmox Containers: Found {} from {}",
                            containers.len(),
                            url
                        );
                        all_containers.extend(containers);
                    }
                }
            }
            _ => continue,
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
        if url.is_empty() {
            continue;
        }

        let Some((ticket, _csrf)) =
            get_proxmox_ticket(client, url, &proxmox_user, &proxmox_password).await
        else {
            continue;
        };

        let api_url = format!("{}/api2/json/nodes", url);

        match client
            .get(&api_url)
            .header("Cookie", format!("PVEAuthCookie={}", ticket))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(items) = data["data"].as_array() {
                        let nodes: Vec<Value> = items
                            .iter()
                            .map(|node| {
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
                            })
                            .collect();
                        info!("✅ Proxmox Nodes: Found {} from {}", nodes.len(), url);
                        all_nodes.extend(nodes);
                    }
                }
            }
            _ => continue,
        }
    }

    Ok(json!(all_nodes))
}

pub async fn get_all_proxmox_volumes(client: &reqwest::Client) -> Result<Value, String> {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();

    if proxmox_urls.is_empty() || proxmox_user.is_empty() {
        return Ok(json!([]));
    }

    let urls: Vec<&str> = proxmox_urls.split(',').collect();
    let mut all_volumes = Vec::new();

    for url in urls {
        let url = url.trim();
        if url.is_empty() {
            continue;
        }

        let Some((ticket, _csrf)) =
            get_proxmox_ticket(client, url, &proxmox_user, &proxmox_password).await
        else {
            continue;
        };

        // First get nodes
        let nodes_api_url = format!("{}/api2/json/nodes", url);
        let mut node_names = Vec::new();

        if let Ok(res) = client.get(&nodes_api_url).header("Cookie", format!("PVEAuthCookie={}", ticket)).send().await {
            if let Ok(data) = res.json::<Value>().await {
                if let Some(nodes) = data["data"].as_array() {
                    for node in nodes {
                        if let Some(n) = node["node"].as_str() {
                            node_names.push(n.to_string());
                        }
                    }
                }
            }
        }

        for node in node_names {
            // Get all storages for this node
            let storages_api_url = format!("{}/api2/json/nodes/{}/storage", url, node);
            let mut storage_names = Vec::new();

            if let Ok(res) = client.get(&storages_api_url).header("Cookie", format!("PVEAuthCookie={}", ticket)).send().await {
                if let Ok(data) = res.json::<Value>().await {
                    if let Some(storages) = data["data"].as_array() {
                        for storage in storages {
                            if let Some(s) = storage["storage"].as_str() {
                                storage_names.push(s.to_string());
                            }
                        }
                    }
                }
            }

            for storage in storage_names {
                let content_api_url = format!("{}/api2/json/nodes/{}/storage/{}/content", url, node, storage);
                if let Ok(res) = client.get(&content_api_url).header("Cookie", format!("PVEAuthCookie={}", ticket)).send().await {
                    if let Ok(data) = res.json::<Value>().await {
                        if let Some(contents) = data["data"].as_array() {
                            for item in contents {
                                let mut vol = item.clone();
                                vol["proxmox_node"] = json!(node);
                                vol["proxmox_storage"] = json!(storage);
                                vol["proxmox_url"] = json!(url);
                                all_volumes.push(vol);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(json!(all_volumes))
}

pub async fn delete_proxmox_volume(
    client: &reqwest::Client,
    server: &str,
    node: &str,
    storage: &str,
    volume: &str,
) -> Result<String, String> {
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();

    let Some((ticket, csrf)) =
        get_proxmox_ticket(client, server, &proxmox_user, &proxmox_password).await
    else {
        return Err(format!("Auth failed for server {}", server));
    };

    let api_url = format!(
        "{}/api2/json/nodes/{}/storage/{}/content/{}",
        server, node, storage, volume
    );

    match client
        .delete(&api_url)
        .header("Cookie", format!("PVEAuthCookie={}", ticket))
        .header("CSRFPreventionToken", csrf)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Ok(format!("Volume {} deleted from {} on {}", volume, storage, node))
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(format!("Proxmox API error: {} - {}", status, body))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn check_proxmox_health(client: &reqwest::Client) {
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();

    if proxmox_urls.is_empty() {
        log::warn!("⚠️  Proxmox health check skipped: PROXMOX_URLS not set");
        return;
    }

    log::info!("🔍 Checking Proxmox servers connectivity...");

    let urls: Vec<&str> = proxmox_urls.split(',').collect();

    for url in urls {
        let url = url.trim();
        if url.is_empty() {
            continue;
        }

        match get_proxmox_ticket(client, url, &proxmox_user, &proxmox_password).await {
            Some(_) => log::debug!("✅ Proxmox Server [{}]: ONLINE", url),
            None => log::warn!("❌ Proxmox Server [{}]: OFFLINE", url),
        }
    }
}

pub async fn vm_control(
    client: &reqwest::Client,
    server: &str,
    node: &str,
    vmid: u64,
    action: &str,
) -> Result<String, String> {
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();

    // Validate action
    match action {
        "start" | "stop" | "reset" | "shutdown" | "reboot" => {}
        _ => return Err(format!("Invalid action: {}", action)),
    }

    let Some((ticket, csrf)) =
        get_proxmox_ticket(client, server, &proxmox_user, &proxmox_password).await
    else {
        return Err(format!("Auth failed for server {}", server));
    };

    let api_url = format!(
        "{}/api2/json/nodes/{}/qemu/{}/status/{}",
        server, node, vmid, action
    );

    match client
        .post(&api_url)
        .header("Cookie", format!("PVEAuthCookie={}", ticket))
        .header("CSRFPreventionToken", csrf)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(upid) = data["data"].as_str() {
                        return Ok(upid.to_string());
                    }
                }
                Ok("Command sent".to_string())
            } else {
                Err(format!("Proxmox API error: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn ct_control(
    client: &reqwest::Client,
    server: &str,
    node: &str,
    vmid: u64,
    action: &str,
) -> Result<String, String> {
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();

    // Validate action
    match action {
        "start" | "stop" | "reset" | "shutdown" | "reboot" => {}
        _ => return Err(format!("Invalid action: {}", action)),
    }

    let Some((ticket, csrf)) =
        get_proxmox_ticket(client, server, &proxmox_user, &proxmox_password).await
    else {
        return Err(format!("Auth failed for server {}", server));
    };

    let api_url = format!(
        "{}/api2/json/nodes/{}/lxc/{}/status/{}",
        server, node, vmid, action
    );

    match client
        .post(&api_url)
        .header("Cookie", format!("PVEAuthCookie={}", ticket))
        .header("CSRFPreventionToken", csrf)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(upid) = data["data"].as_str() {
                        return Ok(upid.to_string());
                    }
                }
                Ok("Command sent".to_string())
            } else {
                Err(format!("Proxmox API error: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}

pub async fn deploy_docker_compose_to_proxmox(
    client: &reqwest::Client,
    yaml_content: &str,
    target_node: Option<&str>,
) -> Result<Vec<crate::interfaces::http::handlers::business::proxmox_compose_handlers::ServiceDeployResult>, String> {
    use crate::interfaces::http::handlers::business::proxmox_compose_handlers::ServiceDeployResult;
    
    // Parse Compose YAML
    let compose: serde_yaml::Value = serde_yaml::from_str(yaml_content)
        .map_err(|e| format!("Invalid YAML: {}", e))?;
    
    let services = compose.get("services")
        .and_then(|v| v.as_mapping())
        .ok_or("Missing or invalid 'services' in compose file")?;
    
    let mut results = Vec::new();
    let node = target_node.unwrap_or("aquabot");
    
    let proxmox_urls = std::env::var("PROXMOX_URLS").unwrap_or_default();
    let server = proxmox_urls.split(',').next().unwrap_or_default().trim();
    
    if server.is_empty() {
        return Err("PROXMOX_URLS not set".to_string());
    }

    for (name_val, config) in services {
        let service_name = name_val.as_str().unwrap_or("unknown").to_string();
        let image = config.get("image").and_then(|v| v.as_str()).unwrap_or("");
        
        if image.is_empty() {
            results.push(ServiceDeployResult {
                service_name: service_name.clone(),
                status: "error".to_string(),
                message: "Missing 'image' for service".to_string(),
            });
            continue;
        }

        // Create LXC container
        match create_lxc_from_image(client, server, node, &service_name, image).await {
            Ok(upid) => {
                results.push(ServiceDeployResult {
                    service_name: service_name.clone(),
                    status: "success".to_string(),
                    message: format!("Deployment started: UPID {}", upid),
                });
            }
            Err(e) => {
                results.push(ServiceDeployResult {
                    service_name: service_name.clone(),
                    status: "error".to_string(),
                    message: e,
                });
            }
        }
    }
    
    Ok(results)
}

async fn create_lxc_from_image(
    client: &reqwest::Client,
    server: &str,
    node: &str,
    name: &str,
    image: &str,
) -> Result<String, String> {
    let proxmox_user = std::env::var("PROXMOX_USER").unwrap_or_default();
    let proxmox_password = std::env::var("PROXMOX_PASSWORD").unwrap_or_default();

    let Some((ticket, csrf)) =
        get_proxmox_ticket(client, server, &proxmox_user, &proxmox_password).await
    else {
        return Err(format!("Auth failed for server {}", server));
    };

    // Find next available VMID
    let nextid_url = format!("{}/api2/json/cluster/nextid", server);
    let vmid = match client.get(&nextid_url).header("Cookie", format!("PVEAuthCookie={}", ticket)).send().await {
        Ok(res) if res.status().is_success() => {
            let data = res.json::<Value>().await.map_err(|e| e.to_string())?;
            data["data"].as_u64().ok_or("Failed to get next VMID")?
        }
        _ => return Err("Failed to get next available VMID".to_string()),
    };

    let api_url = format!("{}/api2/json/nodes/{}/lxc", server, node);
    
    // Convert image to Proxmox OCI format if it's just a docker image name
    let ostemplate = if image.starts_with("docker://") {
        image.to_string()
    } else {
        format!("docker://{}", image)
    };

    let params = [
        ("vmid", vmid.to_string()),
        ("ostemplate", ostemplate),
        ("hostname", name.to_string()),
        ("memory", "512".to_string()),
        ("swap", "512".to_string()),
        ("net0", "name=eth0,bridge=vmbr0,ip=dhcp".to_string()),
        ("storage", "local-lvm".to_string()),
        ("unprivileged", "1".to_string()),
        ("start", "1".to_string()),
    ];

    match client
        .post(&api_url)
        .header("Cookie", format!("PVEAuthCookie={}", ticket))
        .header("CSRFPreventionToken", csrf)
        .form(&params)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(data) = response.json::<Value>().await {
                    if let Some(upid) = data["data"].as_str() {
                        return Ok(upid.to_string());
                    }
                }
                Ok("Command sent".to_string())
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                Err(format!("Proxmox API error: {} - {}", status, body))
            }
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}
