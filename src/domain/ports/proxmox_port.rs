use async_trait::async_trait;
use crate::error::Result;

#[async_trait]
pub trait ProxmoxRepository: Send + Sync {
    async fn get_cluster_status(&self) -> Result<ClusterStatus>;
    async fn get_vms(&self) -> Result<Vec<ProxmoxVM>>;
    async fn get_containers(&self) -> Result<Vec<ProxmoxContainer>>;
    async fn vm_control(&self, vmid: u32, action: &str) -> Result<()>;
    async fn ct_control(&self, vmid: u32, action: &str) -> Result<()>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClusterStatus {
    pub nodes: i32,
    pub vms: i32,
    pub containers: i32,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProxmoxVM {
    pub vmid: u32,
    pub name: String,
    pub status: String,
    pub cpu: f64,
    pub memory: i64,
    pub uptime: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProxmoxContainer {
    pub vmid: u32,
    pub name: String,
    pub status: String,
    pub cpu: f64,
    pub memory: i64,
    pub uptime: i64,
}
