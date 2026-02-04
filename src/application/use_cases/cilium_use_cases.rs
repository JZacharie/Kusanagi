use crate::domain::ports::{CiliumRepository, NetworkFlow, NetworkPolicy, BandwidthMetrics, NetworkAnomaly};
use crate::error::Result;
use std::sync::Arc;

pub struct GetNetworkFlowsUseCase {
    cilium_repo: Arc<dyn CiliumRepository>,
}

impl GetNetworkFlowsUseCase {
    pub fn new(cilium_repo: Arc<dyn CiliumRepository>) -> Self {
        Self { cilium_repo }
    }

    pub async fn execute(&self, namespace: Option<&str>) -> Result<Vec<NetworkFlow>> {
        self.cilium_repo.get_network_flows(namespace).await
    }
}

pub struct GetNetworkPoliciesUseCase {
    cilium_repo: Arc<dyn CiliumRepository>,
}

impl GetNetworkPoliciesUseCase {
    pub fn new(cilium_repo: Arc<dyn CiliumRepository>) -> Self {
        Self { cilium_repo }
    }

    pub async fn execute(&self) -> Result<Vec<NetworkPolicy>> {
        self.cilium_repo.get_network_policies().await
    }
}

pub struct GetBandwidthMetricsUseCase {
    cilium_repo: Arc<dyn CiliumRepository>,
}

impl GetBandwidthMetricsUseCase {
    pub fn new(cilium_repo: Arc<dyn CiliumRepository>) -> Self {
        Self { cilium_repo }
    }

    pub async fn execute(&self) -> Result<BandwidthMetrics> {
        self.cilium_repo.get_bandwidth_metrics().await
    }
}
