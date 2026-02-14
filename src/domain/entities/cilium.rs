//! Cilium Domain Entities
//!
//! Entities for Cilium network visualization and Hubble flows

use serde::{Deserialize, Serialize};

// ==================== Network Flows ====================

/// Network flow between services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkFlow {
    pub source_namespace: String,
    pub source_pod: String,
    pub source_labels: Vec<String>,
    pub destination_namespace: String,
    pub destination_pod: String,
    pub destination_labels: Vec<String>,
    pub destination_port: u16,
    pub protocol: String,
    pub verdict: String, // "FORWARDED", "DROPPED", "AUDIT"
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_seen: String,
}

/// Flow matrix entry (aggregated flows between namespaces/services)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowMatrixEntry {
    pub source: String,
    pub destination: String,
    pub protocol: String,
    // Using u16 for port is standard, but keeping compatible with legacy
    pub port: u16,
    pub flow_count: u64,
    pub bytes_total: u64,
    pub verdict: String,
}

/// Hubble flows response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubbleFlowsResponse {
    pub total_flows: u64,
    pub flows: Vec<NetworkFlow>,
    pub matrix: Vec<FlowMatrixEntry>,
    pub namespaces: Vec<String>,
    pub timestamp: String,
}

// ==================== Metrics ====================

/// Bandwidth metrics per service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthMetrics {
    pub namespace: String,
    pub service: String,
    pub ingress_bytes_per_sec: f64,
    pub egress_bytes_per_sec: f64,
    pub connection_count: u64,
}

// ==================== Anomalies ====================

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAnomaly {
    pub anomaly_type: String, // "unexpected_flow", "traffic_spike", "dropped_traffic"
    pub severity: String,     // "low", "medium", "high"
    pub source: String,
    pub destination: String,
    pub description: String,
    pub timestamp: String,
}
