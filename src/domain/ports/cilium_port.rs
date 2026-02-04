use async_trait::async_trait;
use crate::error::Result;

#[async_trait]
pub trait CiliumRepository: Send + Sync {
    async fn get_network_flows(&self, namespace: Option<&str>) -> Result<Vec<NetworkFlow>>;
    async fn get_network_policies(&self) -> Result<Vec<NetworkPolicy>>;
    async fn get_bandwidth_metrics(&self) -> Result<BandwidthMetrics>;
    async fn detect_anomalies(&self) -> Result<Vec<NetworkAnomaly>>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkFlow {
    pub source_ip: String,
    pub destination_ip: String,
    pub source_port: u16,
    pub destination_port: u16,
    pub protocol: String,
    pub verdict: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicy {
    pub name: String,
    pub namespace: String,
    pub action: String,
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BandwidthMetrics {
    pub total_bytes: i64,
    pub ingress_bytes: i64,
    pub egress_bytes: i64,
    pub flows_per_second: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkAnomaly {
    pub anomaly_type: String,
    pub description: String,
    pub severity: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
