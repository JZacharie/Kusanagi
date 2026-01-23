use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxmoxVM {
    pub vmid: u64,
    pub name: String,
    pub status: String,
    pub node: String,
    pub cpu: f64,
    pub mem: u64,
    pub maxmem: u64,
    pub disk: u64,
    pub maxdisk: u64,
    pub uptime: u64,
    pub netin: u64,
    pub netout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxmoxContainer {
    pub vmid: u64,
    pub name: String,
    pub status: String,
    pub node: String,
    pub cpu: f64,
    pub mem: u64,
    pub maxmem: u64,
    pub disk: u64,
    pub maxdisk: u64,
    pub uptime: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxmoxNode {
    pub node: String,
    pub status: String,
    pub cpu: f64,
    pub maxcpu: u32,
    pub mem: u64,
    pub maxmem: u64,
    pub disk: u64,
    pub maxdisk: u64,
    pub uptime: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClusterStatus {
    pub name: String,
    pub nodes: u32,
    pub quorate: bool,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProxmoxApiResponse<T> {
    data: T,
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginResponse {
    ticket: String,
    #[serde(rename = "CSRFPreventionToken")]
    csrf_prevention_token: String,
}

pub struct ProxmoxClient {
    nodes: Vec<ProxmoxNodeClient>,
}

struct ProxmoxNodeClient {
    base_url: String,
    client: reqwest::Client,
    token: Option<String>,
    ticket: Option<String>,
    csrf_token: Option<String>,
}

impl ProxmoxClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let urls_str = env::var("PROXMOX_URLS")
            .or_else(|_| env::var("PROXMOX_URL"))
            .unwrap_or_else(|_| "https://proxmox.local:8006".to_string());
        
        let user = env::var("PROXMOX_USER")
            .unwrap_or_else(|_| "root@pam".to_string());
        let password = env::var("PROXMOX_PASSWORD").ok();
        let token_id = env::var("PROXMOX_TOKEN_ID").ok();
        let token_secret = env::var("PROXMOX_TOKEN_SECRET").ok();

        let urls: Vec<&str> = urls_str.split(',').map(|s| s.trim()).collect();
        let mut node_clients = Vec::new();

        for url in urls {
            if url.is_empty() { continue; }
            
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(10))
                .build()?;

            let mut node_client = ProxmoxNodeClient {
                base_url: url.to_string(),
                client,
                token: None,
                ticket: None,
                csrf_token: None,
            };

            // Prefer API Token if available
            if let (Some(tid), Some(ts)) = (&token_id, &token_secret) {
                node_client.token = Some(format!("PVEAPIToken={}!{}={}", user, tid, ts));
            } else if let Some(pwd) = &password {
                // Otherwise use password to get a ticket
                match Self::login(&node_client.client, url, &user, pwd).await {
                    Ok(login) => {
                        node_client.ticket = Some(format!("PVEAuthCookie={}", login.ticket));
                        node_client.csrf_token = Some(login.csrf_prevention_token);
                    }
                    Err(e) => {
                        warn!("Failed to login to Proxmox at {}: {}", url, e);
                    }
                }
            }

            node_clients.push(node_client);
        }

        Ok(Self { nodes: node_clients })
    }

    async fn login(client: &reqwest::Client, base_url: &str, user: &str, password: &str) -> Result<LoginResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/api2/json/access/ticket", base_url);
        let response = client.post(&url)
            .form(&[("username", user), ("password", password)])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("Login failed: {}", response.status()).into());
        }

        let api_response: ProxmoxApiResponse<LoginResponse> = response.json().await?;
        Ok(api_response.data)
    }

    async fn get<T: for<'de> Deserialize<'de> + Clone>(
        &self,
        path: &str,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>> {
        let mut all_data = Vec::new();

        for node in &self.nodes {
            let url = format!("{}/api2/json{}", node.base_url, path);
            let mut request = node.client.get(&url);

            if let Some(token) = &node.token {
                request = request.header("Authorization", token);
            } else if let Some(ticket) = &node.ticket {
                request = request.header("Cookie", ticket);
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<ProxmoxApiResponse<T>>().await {
                            Ok(api_response) => {
                                all_data.push(api_response.data);
                            }
                            Err(e) => warn!("Failed to parse response from {}: {}", node.base_url, e),
                        }
                    } else {
                        warn!("Proxmox node {} returned error: {}", node.base_url, response.status());
                    }
                }
                Err(e) => warn!("Failed to connect to Proxmox node {}: {}", node.base_url, e),
            }
        }

        if all_data.is_empty() && !self.nodes.is_empty() {
            return Err("Failed to retrieve data from any Proxmox node".into());
        }

        Ok(all_data)
    }

    async fn post<T: for<'de> Deserialize<'de> + Clone>(
        &self,
        path: &str,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>> {
        let mut all_data = Vec::new();

        for node in &self.nodes {
            let url = format!("{}/api2/json{}", node.base_url, path);
            let mut request = node.client.post(&url);

            if let Some(token) = &node.token {
                request = request.header("Authorization", token);
            } else if let Some(ticket) = &node.ticket {
                request = request.header("Cookie", ticket);
                if let Some(csrf) = &node.csrf_token {
                    request = request.header("CSRFPreventionToken", csrf);
                }
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<ProxmoxApiResponse<T>>().await {
                            Ok(api_response) => {
                                all_data.push(api_response.data);
                            }
                            Err(e) => warn!("Failed to parse POST response from {}: {}", node.base_url, e),
                        }
                    } else {
                        warn!("Proxmox node {} returned error on POST: {}", node.base_url, response.status());
                    }
                }
                Err(e) => warn!("Failed to connect to Proxmox node {}: {}", node.base_url, e),
            }
        }

        if all_data.is_empty() && !self.nodes.is_empty() {
            return Err("Failed to execute POST on any Proxmox node".into());
        }

        Ok(all_data)
    }

    pub async fn vm_control(
        &self,
        node: &str,
        vmid: u64,
        action: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let path = format!("/nodes/{}/qemu/{}/status/{}", node, vmid, action);
        let upids: Vec<String> = self.post(&path).await?;
        Ok(upids.get(0).cloned().unwrap_or_default())
    }

    pub async fn ct_control(
        &self,
        node: &str,
        vmid: u64,
        action: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let path = format!("/nodes/{}/lxc/{}/status/{}", node, vmid, action);
        let upids: Vec<String> = self.post(&path).await?;
        Ok(upids.get(0).cloned().unwrap_or_default())
    }

    pub async fn get_nodes(&self) -> Result<Vec<ProxmoxNode>, Box<dyn std::error::Error>> {
        let results: Vec<Vec<ProxmoxNode>> = self.get("/nodes").await?;
        Ok(results.into_iter().flatten().collect())
    }

    pub async fn get_vms(&self) -> Result<Vec<ProxmoxVM>, Box<dyn std::error::Error>> {
        let nodes = self.get_nodes().await?;
        let mut all_vms = Vec::new();

        for node in nodes {
            let path = format!("/nodes/{}/qemu", node.node);
            match self.get::<Vec<ProxmoxVM>>(&path).await {
                Ok(results) => {
                    for mut vms in results {
                        for vm in &mut vms {
                            vm.node = node.node.clone();
                        }
                        all_vms.extend(vms);
                    }
                }
                Err(e) => warn!("Failed to get VMs from node {}: {}", node.node, e),
            }
        }

        Ok(all_vms)
    }

    pub async fn get_containers(&self) -> Result<Vec<ProxmoxContainer>, Box<dyn std::error::Error>> {
        let nodes = self.get_nodes().await?;
        let mut all_containers = Vec::new();

        for node in nodes {
            let path = format!("/nodes/{}/lxc", node.node);
            match self.get::<Vec<ProxmoxContainer>>(&path).await {
                Ok(results) => {
                    for mut containers in results {
                        for ct in &mut containers {
                            ct.node = node.node.clone();
                        }
                        all_containers.extend(containers);
                    }
                }
                Err(e) => warn!("Failed to get containers from node {}: {}", node.node, e),
            }
        }

        Ok(all_containers)
    }

    pub async fn get_cluster_status(&self) -> Result<ClusterStatus, Box<dyn std::error::Error>> {
        #[derive(Deserialize, Clone)]
        struct ClusterInfo {
            name: Option<String>,
            nodes: Option<u32>,
            quorate: Option<u32>,
            version: Option<String>,
        }

        let results: Vec<Vec<ClusterInfo>> = self.get("/cluster/status").await?;
        
        for info in results {
            if let Some(cluster) = info.iter().find(|i| i.name.is_some()) {
                return Ok(ClusterStatus {
                    name: cluster.name.clone().unwrap_or_else(|| "unknown".to_string()),
                    nodes: cluster.nodes.unwrap_or(0),
                    quorate: cluster.quorate.unwrap_or(0) == 1,
                    version: cluster.version.clone().unwrap_or_else(|| "unknown".to_string()),
                });
            }
        }

        Err("No cluster info found on any node".into())
    }
}

// API Handlers
pub async fn get_vms_handler() -> Result<HttpResponse> {
    match ProxmoxClient::new().await {
        Ok(client) => match client.get_vms().await {
            Ok(vms) => {
                info!("Retrieved {} VMs from Proxmox", vms.len());
                Ok(HttpResponse::Ok().json(vms))
            }
            Err(e) => {
                error!("Failed to get VMs: {}", e);
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": format!("Failed to fetch VMs: {}", e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Proxmox client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Proxmox not configured: {}", e)
            })))
        }
    }
}

pub async fn get_containers_handler() -> Result<HttpResponse> {
    match ProxmoxClient::new().await {
        Ok(client) => match client.get_containers().await {
            Ok(containers) => {
                info!("Retrieved {} containers from Proxmox", containers.len());
                Ok(HttpResponse::Ok().json(containers))
            }
            Err(e) => {
                error!("Failed to get containers: {}", e);
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": format!("Failed to fetch containers: {}", e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Proxmox client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Proxmox not configured: {}", e)
            })))
        }
    }
}

pub async fn get_nodes_handler() -> Result<HttpResponse> {
    match ProxmoxClient::new().await {
        Ok(client) => match client.get_nodes().await {
            Ok(nodes) => {
                info!("Retrieved {} nodes from Proxmox", nodes.len());
                Ok(HttpResponse::Ok().json(nodes))
            }
            Err(e) => {
                error!("Failed to get nodes: {}", e);
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": format!("Failed to fetch nodes: {}", e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Proxmox client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Proxmox not configured: {}", e)
            })))
        }
    }
}

pub async fn get_cluster_handler() -> Result<HttpResponse> {
    match ProxmoxClient::new().await {
        Ok(client) => match client.get_cluster_status().await {
            Ok(status) => {
                info!("Retrieved cluster status from Proxmox");
                Ok(HttpResponse::Ok().json(status))
            }
            Err(e) => {
                error!("Failed to get cluster status: {}", e);
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": format!("Failed to fetch cluster status: {}", e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Proxmox client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Proxmox not configured: {}", e)
            })))
        }
    }
}

pub async fn vm_control_handler(
    params: web::Path<(u64, String, String)>,
) -> Result<HttpResponse> {
    let (vmid, node, action) = params.into_inner();
    
    match ProxmoxClient::new().await {
        Ok(client) => match client.vm_control(&node, vmid, &action).await {
            Ok(upid) => {
                info!("VM {} {} on node {} initiated: {}", vmid, action, node, upid);
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                    "upid": upid,
                    "message": format!("VM {} {} order sent", vmid, action)
                })))
            }
            Err(e) => {
                error!("Failed to control VM {}: {}", vmid, e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to {} VM: {}", action, e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Proxmox client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Proxmox not configured: {}", e)
            })))
        }
    }
}

pub async fn ct_control_handler(
    params: web::Path<(u64, String, String)>,
) -> Result<HttpResponse> {
    let (vmid, node, action) = params.into_inner();
    
    match ProxmoxClient::new().await {
        Ok(client) => match client.ct_control(&node, vmid, &action).await {
            Ok(upid) => {
                info!("Container {} {} on node {} initiated: {}", vmid, action, node, upid);
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "status": "success",
                    "upid": upid,
                    "message": format!("Container {} {} order sent", vmid, action)
                })))
            }
            Err(e) => {
                error!("Failed to control Container {}: {}", vmid, e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to {} Container: {}", action, e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Proxmox client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Proxmox not configured: {}", e)
            })))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/proxmox")
            .route("/vms", web::get().to(get_vms_handler))
            .route("/containers", web::get().to(get_containers_handler))
            .route("/nodes", web::get().to(get_nodes_handler))
            .route("/cluster", web::get().to(get_cluster_handler))
            .route("/vm/{vmid}/node/{node}/status/{action}", web::post().to(vm_control_handler))
            .route("/ct/{vmid}/node/{node}/status/{action}", web::post().to(ct_control_handler)),
    );
}
