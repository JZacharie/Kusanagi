use async_trait::async_trait;
use crate::domain::ports::{ProxmoxRepository, ClusterStatus, ProxmoxVM, ProxmoxContainer};
use crate::error::{Result, KusanagiError};
// use crate::legacy:: // Disabled for core versionproxmox::ProxmoxClient;

pub struct LegacyProxmoxRepository;

#[async_trait]
impl ProxmoxRepository for LegacyProxmoxRepository {
    async fn get_cluster_status(&self) -> Result<ClusterStatus> {
        let client = ProxmoxClient::new().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))?;
        client.get_cluster_status().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }

    async fn get_vms(&self) -> Result<Vec<ProxmoxVM>> {
        let client = ProxmoxClient::new().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))?;
        client.get_vms().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }

    async fn get_containers(&self) -> Result<Vec<ProxmoxContainer>> {
        let client = ProxmoxClient::new().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))?;
        client.get_containers().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }

    async fn vm_control(&self, vmid: u32, action: &str) -> Result<()> {
        // This would need server and node parameters, simplified for now
        Err(KusanagiError::not_implemented("vm_control requires server and node parameters"))
    }

    async fn ct_control(&self, vmid: u32, action: &str) -> Result<()> {
        // This would need server and node parameters, simplified for now
        Err(KusanagiError::not_implemented("ct_control requires server and node parameters"))
    }
}
