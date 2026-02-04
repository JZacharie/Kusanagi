use crate::domain::ports::{ProxmoxRepository, ClusterStatus, ProxmoxVM, ProxmoxContainer};
use crate::error::Result;
use std::sync::Arc;

pub struct GetProxmoxClusterUseCase {
    proxmox_repo: Arc<dyn ProxmoxRepository>,
}

impl GetProxmoxClusterUseCase {
    pub fn new(proxmox_repo: Arc<dyn ProxmoxRepository>) -> Self {
        Self { proxmox_repo }
    }

    pub async fn execute(&self) -> Result<ClusterStatus> {
        self.proxmox_repo.get_cluster_status().await
    }
}

pub struct GetProxmoxVMsUseCase {
    proxmox_repo: Arc<dyn ProxmoxRepository>,
}

impl GetProxmoxVMsUseCase {
    pub fn new(proxmox_repo: Arc<dyn ProxmoxRepository>) -> Self {
        Self { proxmox_repo }
    }

    pub async fn execute(&self) -> Result<Vec<ProxmoxVM>> {
        self.proxmox_repo.get_vms().await
    }
}

pub struct ControlProxmoxVMUseCase {
    proxmox_repo: Arc<dyn ProxmoxRepository>,
}

impl ControlProxmoxVMUseCase {
    pub fn new(proxmox_repo: Arc<dyn ProxmoxRepository>) -> Self {
        Self { proxmox_repo }
    }

    pub async fn execute(&self, vmid: u32, action: &str) -> Result<()> {
        self.proxmox_repo.vm_control(vmid, action).await
    }
}
