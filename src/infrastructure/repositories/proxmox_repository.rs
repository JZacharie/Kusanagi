use async_trait::async_trait;
use crate::domain::ports::{ProxmoxRepository, ClusterStatus, ProxmoxVM, ProxmoxContainer};
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacyProxmoxRepository;

#[async_trait]
impl ProxmoxRepository for LegacyProxmoxRepository {
    async fn get_cluster_status(&self) -> Result<ClusterStatus> {
        legacy::proxmox::get_cluster_status().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }

    async fn get_vms(&self) -> Result<Vec<ProxmoxVM>> {
        legacy::proxmox::get_vms().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }

    async fn get_containers(&self) -> Result<Vec<ProxmoxContainer>> {
        legacy::proxmox::get_containers().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }

    async fn vm_control(&self, vmid: u32, action: &str) -> Result<()> {
        legacy::proxmox::vm_control(vmid, action).await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }

    async fn ct_control(&self, vmid: u32, action: &str) -> Result<()> {
        legacy::proxmox::ct_control(vmid, action).await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }
}
