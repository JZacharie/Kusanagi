use async_trait::async_trait;
use crate::domain::ports::{CiliumRepository, NetworkFlow, NetworkPolicy, BandwidthMetrics, NetworkAnomaly};
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacyCiliumRepository;

#[async_trait]
impl CiliumRepository for LegacyCiliumRepository {
    async fn get_network_flows(&self, namespace: Option<&str>) -> Result<Vec<NetworkFlow>> {
        legacy::cilium::get_hubble_flows(namespace, 100).await
            .map_err(|e| KusanagiError::external_api("Cilium", &e.to_string()))
    }

    async fn get_network_policies(&self) -> Result<Vec<NetworkPolicy>> {
        // Simplified - would map from legacy cilium module
        Ok(vec![])
    }

    async fn get_bandwidth_metrics(&self) -> Result<BandwidthMetrics> {
        legacy::cilium::get_bandwidth_metrics().await
            .map_err(|e| KusanagiError::external_api("Cilium", &e.to_string()))
    }

    async fn detect_anomalies(&self) -> Result<Vec<NetworkAnomaly>> {
        legacy::cilium::detect_anomalies().await
            .map_err(|e| KusanagiError::external_api("Cilium", &e.to_string()))
    }
}
