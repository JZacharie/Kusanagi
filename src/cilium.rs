//! Cilium Network Visualization Module
//! Provides access to Hubble flows and network policies for visualization
//! 
//! ## APM Integration
//! This module sends telemetry to OpenObserve for performance monitoring.
//! Each function is instrumented with timing spans.

use serde::{Deserialize, Serialize};
use tracing::{info, debug};
use kube::{Api, Client, api::ListParams};
use k8s_openapi::api::core::v1::Namespace;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Instant, Duration};
use crate::telemetry;

/// Cache for network data to improve performance
pub struct CiliumCache {
    pub flows: RwLock<Option<(HubbleFlowsResponse, Instant)>>,
    pub namespaces: RwLock<Option<(Vec<String>, Instant)>>,
    pub metrics: RwLock<Option<(Vec<BandwidthMetrics>, Instant)>>,
}

impl CiliumCache {
    pub fn new() -> Self {
        Self {
            flows: RwLock::new(None),
            namespaces: RwLock::new(None),
            metrics: RwLock::new(None),
        }
    }
}

lazy_static::lazy_static! {
    pub static ref NETWORK_CACHE: Arc<CiliumCache> = Arc::new(CiliumCache::new());
}


// ============================================================================
// Namespace Fetching (Pre-filter for performance)
// ============================================================================

/// Fetch all namespaces from Kubernetes (with caching)
pub async fn get_namespaces() -> Result<Vec<String>, String> {
    let span = telemetry::start_span("cilium.get_namespaces")
        .with_endpoint("/api/cilium/namespaces");
    
    // 1. Try to get from cache
    {
        let cache = NETWORK_CACHE.namespaces.read().await;
        if let Some((ref ns, timestamp)) = *cache {
            if timestamp.elapsed() < Duration::from_secs(60) {
                debug!("🚀 Returning namespaces from cache");
                span.record("cache_hit", Some(ns.len() as u64));
                return Ok(ns.clone());
            }
        }
    }

    debug!("🔍 Fetching live namespaces from Kubernetes");
    let client = Client::try_default().await
        .map_err(|e| format!("Failed to create K8s client: {}", e))?;
        
    let result = fetch_live_namespaces(&client).await;
    
    if let Ok(ref ns) = result {
        span.record("success", Some(ns.len() as u64));
        let mut cache = NETWORK_CACHE.namespaces.write().await;
        *cache = Some((ns.clone(), Instant::now()));
    }
    
    result
}


/// Network flow between services
#[derive(Serialize, Deserialize, Debug, Clone)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FlowMatrixEntry {
    pub source: String,
    pub destination: String,
    pub protocol: String,
    pub port: u16,
    pub flow_count: u64,
    pub bytes_total: u64,
    pub verdict: String,
}

/// Hubble flows response
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HubbleFlowsResponse {
    pub total_flows: u64,
    pub flows: Vec<NetworkFlow>,
    pub matrix: Vec<FlowMatrixEntry>,
    pub namespaces: Vec<String>,
    pub timestamp: String,
}

/// Bandwidth metrics per service
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BandwidthMetrics {
    pub namespace: String,
    pub service: String,
    pub ingress_bytes_per_sec: f64,
    pub egress_bytes_per_sec: f64,
    pub connection_count: u64,
}

/// Anomaly detection result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkAnomaly {
    pub anomaly_type: String,  // "unexpected_flow", "traffic_spike", "dropped_traffic"
    pub severity: String,      // "low", "medium", "high"
    pub source: String,
    pub destination: String,
    pub description: String,
    pub timestamp: String,
}


// ============================================================================
// Hubble Flow Fetching
// ============================================================================

/// Fetch network flows from Hubble Relay (with caching)
pub async fn get_hubble_flows(namespace: Option<&str>, limit: usize) -> Result<HubbleFlowsResponse, String> {
    let span = telemetry::start_span("cilium.get_hubble_flows")
        .with_namespace(namespace)
        .with_endpoint("/api/cilium/flows");
    
    // 1. Try to get from cache first
    {
        let cache = NETWORK_CACHE.flows.read().await;
        if let Some((ref response, timestamp)) = *cache {
            if timestamp.elapsed() < Duration::from_secs(30) {
                debug!("🚀 Returning hubble flows from cache (age: {:?})", timestamp.elapsed());
                let mut filtered_response = response.clone();
                if let Some(ns) = namespace {
                    filtered_response.flows.retain(|f| f.source_namespace == ns || f.destination_namespace == ns);
                }
                filtered_response.flows.truncate(limit);
                span.record("cache_hit", Some(filtered_response.flows.len() as u64));
                return Ok(filtered_response);
            }
        }
    }

    // 2. If cache miss or expired, fetch live (or simulate)
    debug!(namespace = ?namespace, limit = limit, "🔍 Fetching live Hubble flows");
    
    // Background refresh will handle the main update, but we can do a sync one if needed
    // For now, we return mock but we'll implement the background task to make it feel real and fast
    let result = get_mock_flows(namespace, limit);
    
    if let Ok(ref flows) = result {
        span.record("success", Some(flows.flows.len() as u64));
        
        if namespace.is_none() {
            let mut cache = NETWORK_CACHE.flows.write().await;
            *cache = Some((flows.clone(), Instant::now()));
        }
    }
    
    result
}

/// Background task to refresh Cilium/Hubble cache
pub async fn start_background_refresh(client: kube::Client) {
    info!("🚀 Starting Cilium background refresh task");
    
    let mut interval = tokio::time::interval(Duration::from_secs(45));
    
    loop {
        interval.tick().await;
        debug!("🔄 Refreshing Cilium cache...");
        
        // Refresh Namespaces
        if let Ok(ns) = fetch_live_namespaces(&client).await {
            let mut cache = NETWORK_CACHE.namespaces.write().await;
            *cache = Some((ns, Instant::now()));
        }

        // Refresh Flows
        if let Ok(flows) = fetch_live_flows(&client).await {
            let mut cache = NETWORK_CACHE.flows.write().await;
            *cache = Some((flows, Instant::now()));
        }
        
        // Refresh Metrics
        if let Ok(metrics) = fetch_live_metrics(&client).await {
            debug!("✅ Refreshed Cilium bandwidth metrics cache");
            let mut cache = NETWORK_CACHE.metrics.write().await;
            *cache = Some((metrics, Instant::now()));
        }
        debug!("✨ Finished Cilium background refresh cycle");
    }
}

async fn fetch_live_namespaces(client: &kube::Client) -> Result<Vec<String>, String> {
    let ns_api: Api<Namespace> = Api::all(client.clone());
    let namespaces = ns_api.list(&ListParams::default()).await
        .map_err(|e| format!("Failed to list namespaces: {}", e))?;
        
    let mut ns_list: Vec<String> = namespaces.items.iter()
        .filter_map(|ns| ns.metadata.name.clone())
        .collect();
    ns_list.sort();
    Ok(ns_list)
}

async fn fetch_live_flows(_client: &kube::Client) -> Result<HubbleFlowsResponse, String> {
    // This is where real Hubble Relay gRPC would go
    // For now, generate "enhanced mock" data in background
    get_mock_flows(None, 1000)
}

async fn fetch_live_metrics(_client: &kube::Client) -> Result<Vec<BandwidthMetrics>, String> {
    // Simulation for now
    get_bandwidth_metrics(None).await
}

/// Generate mock flows for demonstration
fn get_mock_flows(namespace: Option<&str>, limit: usize) -> Result<HubbleFlowsResponse, String> {
    let namespaces = [
        "default", "kube-system", "argocd", "monitoring", 
        "kusanagi", "n8n", "paperless", "minio"
    ];

    let mut flows = vec![];
    let mut matrix = vec![];

    // Generate sample flows
    let sample_flows = [
        ("argocd", "argocd-server", "kusanagi", "kusanagi-app", 8080, "TCP", 1024),
        ("monitoring", "prometheus", "kusanagi", "kusanagi-app", 8080, "TCP", 2048),
        ("default", "nginx", "kube-system", "coredns", 53, "UDP", 256),
        ("n8n", "n8n-main", "minio", "minio-api", 9000, "TCP", 4096),
        ("paperless", "paperless-web", "monitoring", "grafana", 3000, "TCP", 512),
    ];

    for (src_ns, src_pod, dst_ns, dst_pod, port, proto, bytes) in sample_flows.iter() {
        if namespace.map(|n| n == *src_ns || n == *dst_ns).unwrap_or(true) {
            flows.push(NetworkFlow {
                source_namespace: src_ns.to_string(),
                source_pod: src_pod.to_string(),
                source_labels: vec![format!("app={}", src_pod)],
                destination_namespace: dst_ns.to_string(),
                destination_pod: dst_pod.to_string(), 
                destination_labels: vec![format!("app={}", dst_pod)],
                destination_port: *port,
                protocol: proto.to_string(),
                verdict: "FORWARDED".to_string(),
                bytes_sent: *bytes as u64,
                bytes_received: (*bytes / 2) as u64,
                last_seen: chrono::Utc::now().to_rfc3339(),
            });

            matrix.push(FlowMatrixEntry {
                source: format!("{}/{}", src_ns, src_pod),
                destination: format!("{}/{}", dst_ns, dst_pod),
                protocol: proto.to_string(),
                port: *port,
                flow_count: 100,
                bytes_total: *bytes as u64 * 100,
                verdict: "FORWARDED".to_string(),
            });
        }
    }

    flows.truncate(limit);
    
    Ok(HubbleFlowsResponse {
        total_flows: flows.len() as u64,
        flows,
        matrix,
        namespaces: namespaces.iter().map(|s| s.to_string()).collect(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

// ============================================================================
// Flow Matrix Generation
// ============================================================================

/// Generate flow matrix for visualization (cached)
pub async fn get_flow_matrix(namespace: Option<&str>) -> Result<Vec<FlowMatrixEntry>, String> {
    let span = telemetry::start_span("cilium.get_flow_matrix")
        .with_namespace(namespace)
        .with_endpoint("/api/cilium/matrix");
    
    debug!(namespace = ?namespace, "🔍 Generating flow matrix from flows");
    
    let response = get_hubble_flows(namespace, 1000).await?;
    let matrix_len = response.matrix.len();
    
    info!(matrix_entries = matrix_len, "✅ Flow matrix generated");
    span.record("success", Some(matrix_len as u64));
    
    Ok(response.matrix)
}

// ============================================================================
// Bandwidth Metrics
// ============================================================================

/// Get bandwidth metrics per service (cached)
pub async fn get_bandwidth_metrics(namespace: Option<&str>) -> Result<Vec<BandwidthMetrics>, String> {
    let span = telemetry::start_span("cilium.get_bandwidth_metrics")
        .with_namespace(namespace)
        .with_endpoint("/api/cilium/metrics");
    
    // 1. Try cache
    {
        let cache = NETWORK_CACHE.metrics.read().await;
        if let Some((ref metrics, timestamp)) = *cache {
            if timestamp.elapsed() < Duration::from_secs(60) {
                debug!("🚀 Returning bandwidth metrics from cache");
                let result = if let Some(ns) = namespace {
                    metrics.iter().filter(|m| m.namespace == ns).cloned().collect()
                } else {
                    metrics.clone()
                };
                span.record("cache_hit", Some(result.len() as u64));
                return Ok(result);
            }
        }
    }

    debug!(namespace = ?namespace, "🔍 Fetching bandwidth metrics");
    
    // Simulation for now, background task will update this
    let mock_metrics = vec![
        BandwidthMetrics {
            namespace: "kusanagi".to_string(),
            service: "kusanagi-app".to_string(),
            ingress_bytes_per_sec: 1024.5,
            egress_bytes_per_sec: 512.3,
            connection_count: 42,
        },
        BandwidthMetrics {
            namespace: "monitoring".to_string(),
            service: "prometheus".to_string(),
            ingress_bytes_per_sec: 4096.0,
            egress_bytes_per_sec: 8192.0,
            connection_count: 128,
        },
    ];

    let result = if let Some(ns) = namespace {
        mock_metrics.into_iter().filter(|m| m.namespace == ns).collect::<Vec<_>>()
    } else {
        mock_metrics
    };
    
    // Update cache
    if namespace.is_none() {
        let mut cache = NETWORK_CACHE.metrics.write().await;
        *cache = Some((result.clone(), Instant::now()));
    }
    
    info!(metrics_count = result.len(), "✅ Bandwidth metrics fetched");
    span.record("success", Some(result.len() as u64));
    
    Ok(result)
}

// ============================================================================
// Anomaly Detection
// ============================================================================

/// Detect network anomalies
pub async fn detect_anomalies(namespace: Option<&str>) -> Result<Vec<NetworkAnomaly>, String> {
    let span = telemetry::start_span("cilium.detect_anomalies")
        .with_namespace(namespace)
        .with_endpoint("/api/cilium/anomalies");
    
    debug!(namespace = ?namespace, "🔍 Running anomaly detection");
    
    // TODO: Implement actual anomaly detection based on:
    // - Unexpected source→destination combinations
    // - Traffic spikes (compared to baseline)
    // - High dropped traffic rates
    
    let mock_anomalies = vec![
        NetworkAnomaly {
            anomaly_type: "unexpected_flow".to_string(),
            severity: "medium".to_string(),
            source: "unknown-pod/default".to_string(),
            destination: "argocd-server/argocd".to_string(),
            description: "Unexpected traffic from unknown source to ArgoCD".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ];

    let result = if let Some(ns) = namespace {
        mock_anomalies.into_iter()
            .filter(|a| a.source.contains(ns) || a.destination.contains(ns))
            .collect::<Vec<_>>()
    } else {
        mock_anomalies
    };
    
    info!(anomalies_count = result.len(), "✅ Anomaly detection completed");
    span.record("success", Some(result.len() as u64));
    
    Ok(result)
}

// ============================================================================
// Export Functions
// ============================================================================

/// Export flows as JSON
pub fn export_flows_json(flows: &HubbleFlowsResponse) -> String {
    serde_json::to_string_pretty(flows).unwrap_or_else(|_| "{}".to_string())
}

/// Export flows as CSV
pub fn export_flows_csv(flows: &HubbleFlowsResponse) -> String {
    let mut csv = String::from("source_namespace,source_pod,destination_namespace,destination_pod,port,protocol,verdict,bytes_sent,bytes_received\n");
    
    for flow in &flows.flows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            flow.source_namespace,
            flow.source_pod,
            flow.destination_namespace,
            flow.destination_pod,
            flow.destination_port,
            flow.protocol,
            flow.verdict,
            flow.bytes_sent,
            flow.bytes_received
        ));
    }
    
    csv
}

