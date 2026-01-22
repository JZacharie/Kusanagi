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

pub struct ProxmoxClient {
    base_url: String,
    client: reqwest::Client,
    token: String,
}

impl ProxmoxClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let base_url = env::var("PROXMOX_URL")
            .unwrap_or_else(|_| "https://proxmox.local:8006".to_string());
        let user = env::var("PROXMOX_USER")
            .unwrap_or_else(|_| "root@pam".to_string());
        let token_id = env::var("PROXMOX_TOKEN_ID")
            .unwrap_or_else(|_| "".to_string());
        let token_secret = env::var("PROXMOX_TOKEN_SECRET")
            .unwrap_or_else(|_| "".to_string());

        let token = format!("PVEAPIToken={}!{}={}", user, token_id, token_secret);

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true) // For self-signed certs
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        Ok(Self {
            base_url,
            client,
            token,
        })
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}/api2/json{}", self.base_url, path);
        
        info!("Proxmox API request: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            error!("Proxmox API error: {} - {}", status, body);
            return Err(format!("Proxmox API error: {}", status).into());
        }

        let api_response: ProxmoxApiResponse<T> = response.json().await?;
        Ok(api_response.data)
    }

    async fn post<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}/api2/json{}", self.base_url, path);
        
        info!("Proxmox API POST request: {}", url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.token)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            error!("Proxmox API error: {} - {}", status, body);
            return Err(format!("Proxmox API error: {}", status).into());
        }

        let api_response: ProxmoxApiResponse<T> = response.json().await?;
        Ok(api_response.data)
    }

    pub async fn vm_control(
        &self,
        node: &str,
        vmid: u64,
        action: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let path = format!("/nodes/{}/qemu/{}/status/{}", node, vmid, action);
        let upid: String = self.post(&path).await?;
        Ok(upid)
    }

    pub async fn ct_control(
        &self,
        node: &str,
        vmid: u64,
        action: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let path = format!("/nodes/{}/lxc/{}/status/{}", node, vmid, action);
        let upid: String = self.post(&path).await?;
        Ok(upid)
    }

    pub async fn get_nodes(&self) -> Result<Vec<ProxmoxNode>, Box<dyn std::error::Error>> {
        self.get("/nodes").await
    }

    pub async fn get_vms(&self) -> Result<Vec<ProxmoxVM>, Box<dyn std::error::Error>> {
        let nodes: Vec<ProxmoxNode> = self.get_nodes().await?;
        let mut all_vms = Vec::new();

        for node in nodes {
            match self.get::<Vec<ProxmoxVM>>(&format!("/nodes/{}/qemu", node.node)).await {
                Ok(mut vms) => {
                    for vm in &mut vms {
                        vm.node = node.node.clone();
                    }
                    all_vms.extend(vms);
                }
                Err(e) => {
                    warn!("Failed to get VMs from node {}: {}", node.node, e);
                }
            }
        }

        Ok(all_vms)
    }

    pub async fn get_containers(&self) -> Result<Vec<ProxmoxContainer>, Box<dyn std::error::Error>> {
        let nodes: Vec<ProxmoxNode> = self.get_nodes().await?;
        let mut all_containers = Vec::new();

        for node in nodes {
            match self.get::<Vec<ProxmoxContainer>>(&format!("/nodes/{}/lxc", node.node)).await {
                Ok(mut containers) => {
                    for ct in &mut containers {
                        ct.node = node.node.clone();
                    }
                    all_containers.extend(containers);
                }
                Err(e) => {
                    warn!("Failed to get containers from node {}: {}", node.node, e);
                }
            }
        }

        Ok(all_containers)
    }

    pub async fn get_cluster_status(&self) -> Result<ClusterStatus, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct ClusterInfo {
            name: Option<String>,
            nodes: Option<u32>,
            quorate: Option<u32>,
            version: Option<String>,
        }

        let info: Vec<ClusterInfo> = self.get("/cluster/status").await?;
        
        let cluster = info.iter().find(|i| i.name.is_some()).ok_or("No cluster info found")?;

        Ok(ClusterStatus {
            name: cluster.name.clone().unwrap_or_else(|| "unknown".to_string()),
            nodes: cluster.nodes.unwrap_or(0),
            quorate: cluster.quorate.unwrap_or(0) == 1,
            version: cluster.version.clone().unwrap_or_else(|| "unknown".to_string()),
        })
    }
}

// API Handlers
pub async fn get_vms_handler() -> Result<HttpResponse> {
    match ProxmoxClient::new() {
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
    match ProxmoxClient::new() {
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
    match ProxmoxClient::new() {
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
    match ProxmoxClient::new() {
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
    
    match ProxmoxClient::new() {
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
    
    match ProxmoxClient::new() {
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
