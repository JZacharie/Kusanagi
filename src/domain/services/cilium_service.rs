//! Cilium Service - Domain Service for Network Visualization
//!
//! Handles retrieval and caching of Hubble flows and network metrics.

use crate::domain::entities::cilium::{
    BandwidthMetrics, FlowMatrixEntry, HubbleFlowsResponse, NetworkAnomaly, NetworkFlow,
};
use k8s_openapi::api::core::v1::Namespace;
use kube::{api::ListParams, Api, Client};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Cache types
type FlowsCache = Arc<RwLock<Option<(HubbleFlowsResponse, Instant)>>>;
type NamespacesCache = Arc<RwLock<Option<(Vec<String>, Instant)>>>;
type MetricsCache = Arc<RwLock<Option<(Vec<BandwidthMetrics>, Instant)>>>;

/// Cache for network data to improve performance
#[derive(Clone)]
pub struct CiliumCache {
    pub flows: FlowsCache,
    pub namespaces: NamespacesCache,
    pub metrics: MetricsCache,
}

impl CiliumCache {
    pub fn new() -> Self {
        Self {
            flows: Arc::new(RwLock::new(None)),
            namespaces: Arc::new(RwLock::new(None)),
            metrics: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for CiliumCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Service for Cilium network visualization
#[derive(Clone)]
pub struct CiliumService {
    client: Client,
    cache: Arc<CiliumCache>,
}

impl CiliumService {
    /// Create a new Cilium service
    pub fn new(client: Client, cache: Arc<CiliumCache>) -> Self {
        Self { client, cache }
    }

    /// Start the background refresh task
    pub fn start_background_refresh(&self) {
        let client = self.client.clone();
        let cache = self.cache.clone();

        tokio::spawn(async move {
            info!("🚀 Starting Cilium background refresh task");
            let mut interval = tokio::time::interval(Duration::from_secs(120));

            loop {
                interval.tick().await;
                debug!("🔄 Refreshing Cilium cache...");

                // Refresh Namespaces
                match fetch_live_namespaces(&client).await {
                    Ok(ns) => {
                        let mut ns_cache = cache.namespaces.write().await;
                        *ns_cache = Some((ns, Instant::now()));
                    }
                    Err(e) => warn!("Failed to refresh namespaces: {}", e),
                }

                // Refresh Flows
                match fetch_live_flows(&client).await {
                    Ok(flows) => {
                        let mut flows_cache = cache.flows.write().await;
                        *flows_cache = Some((flows, Instant::now()));
                    }
                    Err(e) => warn!("Failed to refresh flows: {}", e),
                }

                // Refresh Metrics
                match fetch_live_metrics(&client).await {
                    Ok(metrics) => {
                        let mut metrics_cache = cache.metrics.write().await;
                        *metrics_cache = Some((metrics, Instant::now()));
                        debug!("✅ Refreshed Cilium bandwidth metrics cache");
                    }
                    Err(e) => warn!("Failed to refresh metrics: {}", e),
                }

                debug!("✨ Finished Cilium background refresh cycle");
            }
        });
    }

    /// Get all namespaces (cached)
    pub async fn get_namespaces(&self) -> Result<Vec<String>, String> {
        // 1. Try to get from cache
        {
            let cache = self.cache.namespaces.read().await;
            if let Some((ref ns, timestamp)) = *cache {
                if timestamp.elapsed() < Duration::from_secs(60) {
                    debug!("🚀 Returning namespaces from cache");
                    return Ok(ns.clone());
                }
            }
        }

        debug!("🔍 Fetching live namespaces from Kubernetes");
        let result = fetch_live_namespaces(&self.client).await;

        if let Ok(ref ns) = result {
            let mut cache = self.cache.namespaces.write().await;
            *cache = Some((ns.clone(), Instant::now()));
        }

        result
    }

    /// Get Hubble flows (cached)
    pub async fn get_hubble_flows(
        &self,
        namespace: Option<&str>,
        limit: usize,
    ) -> Result<HubbleFlowsResponse, String> {
        // 1. Try to get from cache first
        {
            let cache = self.cache.flows.read().await;
            if let Some((ref response, timestamp)) = *cache {
                if timestamp.elapsed() < Duration::from_secs(30) {
                    debug!(
                        "🚀 Returning hubble flows from cache (age: {:?})",
                        timestamp.elapsed()
                    );
                    let mut filtered_response = response.clone();
                    if let Some(ns) = namespace {
                        filtered_response
                            .flows
                            .retain(|f| f.source_namespace == ns || f.destination_namespace == ns);
                    }
                    filtered_response.flows.truncate(limit);
                    return Ok(filtered_response);
                }
            }
        }

        // 2. If cache miss or expired, generate mock data
        debug!(namespace = ?namespace, limit = limit, "🔍 Generating network flows data");
        let result = get_mock_flows(namespace, limit);

        if let Ok(ref flows) = result {
            // Update cache with new data only if no namespace filter was applied
            // (so we cache the full dataset)
            if namespace.is_none() {
                let mut cache = self.cache.flows.write().await;
                *cache = Some((flows.clone(), Instant::now()));
            }
        }

        result
    }

    /// Generate flow matrix for visualization (cached)
    pub async fn get_flow_matrix(
        &self,
        namespace: Option<&str>,
    ) -> Result<Vec<FlowMatrixEntry>, String> {
        debug!(namespace = ?namespace, "🔍 Generating flow matrix from flows");
        let response = self.get_hubble_flows(namespace, 1000).await?;
        Ok(response.matrix)
    }

    /// Get bandwidth metrics per service (cached)
    pub async fn get_bandwidth_metrics(
        &self,
        namespace: Option<&str>,
    ) -> Result<Vec<BandwidthMetrics>, String> {
        // 1. Try cache
        {
            let cache = self.cache.metrics.read().await;
            if let Some((ref metrics, timestamp)) = *cache {
                if timestamp.elapsed() < Duration::from_secs(60) {
                    debug!("🚀 Returning bandwidth metrics from cache");
                    let result = if let Some(ns) = namespace {
                        metrics
                            .iter()
                            .filter(|m| m.namespace == ns)
                            .cloned()
                            .collect()
                    } else {
                        metrics.clone()
                    };
                    return Ok(result);
                }
            }
        }

        debug!(namespace = ?namespace, "🔍 Fetching bandwidth metrics");

        // Simulation for now
        let result = fetch_live_metrics(&self.client).await?;

        // Update cache
        if namespace.is_none() {
            let mut cache = self.cache.metrics.write().await;
            *cache = Some((result.clone(), Instant::now()));
        }

        // Filter if needed
        let final_result = if let Some(ns) = namespace {
            result
                .into_iter()
                .filter(|m| m.namespace == ns)
                .collect::<Vec<_>>()
        } else {
            result
        };

        Ok(final_result)
    }

    /// Detect network anomalies
    pub async fn detect_anomalies(
        &self,
        namespace: Option<&str>,
    ) -> Result<Vec<NetworkAnomaly>, String> {
        debug!(namespace = ?namespace, "🔍 Running anomaly detection");

        let mock_anomalies = vec![NetworkAnomaly {
            anomaly_type: "unexpected_flow".to_string(),
            severity: "medium".to_string(),
            source: "unknown-pod/default".to_string(),
            destination: "argocd-server/argocd".to_string(),
            description: "Unexpected traffic from unknown source to ArgoCD".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }];

        let result = if let Some(ns) = namespace {
            mock_anomalies
                .into_iter()
                .filter(|a| a.source.contains(ns) || a.destination.contains(ns))
                .collect::<Vec<_>>()
        } else {
            mock_anomalies
        };

        Ok(result)
    }
}

// ==================== Helper Functions ====================

async fn fetch_live_namespaces(client: &kube::Client) -> Result<Vec<String>, String> {
    let ns_api: Api<Namespace> = Api::all(client.clone());
    let namespaces = ns_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list namespaces: {}", e))?;

    let mut ns_list: Vec<String> = namespaces
        .items
        .iter()
        .filter_map(|ns| ns.metadata.name.clone())
        .collect();
    ns_list.sort();
    Ok(ns_list)
}

async fn fetch_live_flows(_client: &kube::Client) -> Result<HubbleFlowsResponse, String> {
    // This is where real Hubble Relay gRPC would go
    // For now, generate "enhanced mock" data
    get_mock_flows(None, 1000)
}

async fn fetch_live_metrics(_client: &kube::Client) -> Result<Vec<BandwidthMetrics>, String> {
    // Simulation for now
    Ok(vec![
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
    ])
}

fn get_mock_flows(namespace: Option<&str>, limit: usize) -> Result<HubbleFlowsResponse, String> {
    let namespaces = [
        "default",
        "kube-system",
        "argocd",
        "monitoring",
        "kusanagi",
        "n8n",
        "paperless",
        "minio",
    ];

    let mut flows = vec![];
    let mut matrix = vec![];

    // Generate sample flows
    let sample_flows = [
        (
            "argocd",
            "argocd-server",
            "kusanagi",
            "kusanagi-app",
            8080,
            "TCP",
            1024,
        ),
        (
            "monitoring",
            "prometheus",
            "kusanagi",
            "kusanagi-app",
            8080,
            "TCP",
            2048,
        ),
        ("default", "nginx", "kube-system", "coredns", 53, "UDP", 256),
        ("n8n", "n8n-main", "minio", "minio-api", 9000, "TCP", 4096),
        (
            "paperless",
            "paperless-web",
            "monitoring",
            "grafana",
            3000,
            "TCP",
            512,
        ),
    ];

    for (src_ns, src_pod, dst_ns, dst_pod, port, proto, bytes) in sample_flows.iter() {
        if namespace
            .map(|n| n == *src_ns || n == *dst_ns)
            .unwrap_or(true)
        {
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
