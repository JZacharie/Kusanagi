use k8s_openapi::api::core::v1::{PersistentVolumeClaim, Pod, Node};
use kube::{
    api::{Api, ListParams},
    Client,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use std::collections::HashMap;

/// Storage status response
#[derive(Clone, Debug, Serialize)]
pub struct StorageStatusResponse {
    pub pvc_count: usize,
    pub pvc_total_capacity_bytes: u64,
    pub pvc_total_usage_bytes: u64,
    pub pvcs: Vec<PvcInfo>,
}

/// Individual PVC information
#[derive(Clone, Debug, Serialize)]
pub struct PvcInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub capacity: String,
    pub capacity_bytes: u64,
    pub used_bytes: Option<u64>,
    pub usage_percent: Option<f64>,
    pub storage_class: String,
    pub access_modes: Vec<String>,
    pub volume_name: String,
    pub pods_using: Vec<String>,
}

/// Kubelet Summary API structures (simplified)
#[derive(Debug, Deserialize)]
struct KubeletStatsSummary {
    pods: Vec<KubeletPodStats>,
}

#[derive(Debug, Deserialize)]
struct KubeletPodStats {
    podRef: KubeletPodRef,
    volume: Option<Vec<KubeletVolumeStats>>,
}

#[derive(Debug, Deserialize)]
struct KubeletPodRef {
    name: String,
    namespace: String,
}

#[derive(Debug, Deserialize)]
struct KubeletVolumeStats {
    name: String,
    usedBytes: Option<u64>,
    capacityBytes: Option<u64>,
    pvcRef: Option<KubeletPvcRef>,
}

#[derive(Debug, Deserialize)]
struct KubeletPvcRef {
    name: String,
    namespace: String,
}

/// Get all PVCs with usage information
pub async fn get_storage_status() -> Result<StorageStatusResponse, String> {
    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create Kubernetes client: {}", e))?;

    let pvc_api: Api<PersistentVolumeClaim> = Api::all(client.clone());
    let node_api: Api<Node> = Api::all(client.clone());

    // 1. List all PVCs
    let pvcs = pvc_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list PVCs: {}", e))?;

    // 2. List all Nodes to query stats
    let nodes = node_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list Nodes: {}", e))?;

    // 3. Collect usage stats from all nodes in parallel
    // Map: (Namespace, PvcName) -> UsedBytes
    let mut usage_map: HashMap<(String, String), u64> = HashMap::new();

    // We'll query nodes sequentially for simplicity to avoid complex async iterator handling in this snippet,
    // but in production parallel futures would be better.
    for node in nodes.items {
        let node_name = node.metadata.name.clone().unwrap_or_default();
        
        // Query Kubelet Stats Summary
        // Path: /api/v1/nodes/{node_name}/proxy/stats/summary
        let request = http::Request::builder()
            .uri(format!("/api/v1/nodes/{}/proxy/stats/summary", node_name))
            .body(vec![])
            .map_err(|e| format!("Failed to build request: {}", e))?;

        match client.request::<KubeletStatsSummary>(request).await {
            Ok(summary) => {
                for pod in summary.pods {
                    if let Some(volumes) = pod.volume {
                        for vol in volumes {
                            if let Some(pvc_ref) = vol.pvcRef {
                                if let Some(used) = vol.usedBytes {
                                    usage_map.insert(
                                        (pvc_ref.namespace, pvc_ref.name),
                                        used
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // Just log error and continue, don't fail entire request if one node fails
                error!("Failed to fetch stats from node {}: {}", node_name, e);
            }
        }
    }

    // 4. Build response
    let mut response = StorageStatusResponse {
        pvc_count: pvcs.items.len(),
        pvc_total_capacity_bytes: 0,
        pvc_total_usage_bytes: 0,
        pvcs: Vec::new(),
    };

    for pvc in pvcs.items {
        let name = pvc.metadata.name.unwrap_or_default();
        let namespace = pvc.metadata.namespace.unwrap_or_default();
        let spec = pvc.spec.unwrap_or_default();
        let status = pvc.status.unwrap_or_default();

        let phase = status.phase.unwrap_or_else(|| "Unknown".to_string());
        
        let capacity_str = status.capacity
            .as_ref()
            .and_then(|c| c.get("storage"))
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());
            
        let capacity_bytes = parse_capacity(&capacity_str);
        
        // Get storage class
        let storage_class = spec.storage_class_name.unwrap_or_default();
        
        // Get access modes
        let access_modes = spec.access_modes.unwrap_or_default();
        
        // Get volume name
        let volume_name = spec.volume_name.unwrap_or_default();

        // Get usage from map
        let used_bytes = usage_map.get(&(namespace.clone(), name.clone())).copied();
        
        let usage_percent = if let Some(used) = used_bytes {
            if capacity_bytes > 0 {
                Some((used as f64 / capacity_bytes as f64) * 100.0)
            } else {
                Some(0.0)
            }
        } else {
            None
        };

        // Update totals
        response.pvc_total_capacity_bytes += capacity_bytes;
        if let Some(used) = used_bytes {
            response.pvc_total_usage_bytes += used;
        }

        response.pvcs.push(PvcInfo {
            name,
            namespace,
            status: phase,
            capacity: capacity_str,
            capacity_bytes,
            used_bytes,
            usage_percent,
            storage_class,
            access_modes,
            volume_name,
            pods_using: Vec::new(), // Note: To populate this we'd need to list Pods and check volumes
        });
    }

    Ok(response)
}

fn parse_capacity(cap: &str) -> u64 {
    let cap = cap.trim();
    if cap.ends_with("Gi") {
        cap.trim_end_matches("Gi").parse::<f64>().unwrap_or(0.0) as u64 * 1024 * 1024 * 1024
    } else if cap.ends_with("Mi") {
        cap.trim_end_matches("Mi").parse::<f64>().unwrap_or(0.0) as u64 * 1024 * 1024
    } else if cap.ends_with("Ki") {
        cap.trim_end_matches("Ki").parse::<f64>().unwrap_or(0.0) as u64 * 1024
    } else {
        cap.parse::<u64>().unwrap_or(0)
    }
}
