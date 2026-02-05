use k8s_openapi::api::core::v1::{Namespace, PersistentVolumeClaim, Pod};
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet};
use k8s_openapi::api::batch::v1::{Job, CronJob};
use k8s_openapi::api::networking::v1::Ingress;
use kube::{Client, Api, api::ListParams};
use serde::Serialize;
use tracing::info;

/// Cluster overview response
#[derive(Clone, Debug, Serialize)]
pub struct ClusterOverview {
    pub namespace_count: usize,
    pub namespaces: Vec<NamespaceInfo>,
    pub pvc_count: usize,
    pub pvc_total_capacity: String,
    pub pvcs: Vec<PvcInfo>,
    pub empty_namespaces: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NamespaceInfo {
    pub name: String,
    pub status: String,
    pub labels: std::collections::BTreeMap<String, String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PvcInfo {
    pub name: String,
    pub namespace: String,
    pub storage_class: Option<String>,
    pub capacity: String,
    pub capacity_bytes: i64,
    pub status: String,
    pub access_modes: Vec<String>,
    pub bound_to: Option<String>,
}

/// Get cluster overview with namespaces and PVCs
pub async fn get_cluster_overview(client: &Client) -> Result<ClusterOverview, String> {

    let ns_api: Api<Namespace> = Api::all(client.clone());
    let pvc_api: Api<PersistentVolumeClaim> = Api::all(client.clone());

    // Get namespaces
    let namespaces = ns_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list namespaces: {}", e))?;

    let namespace_infos: Vec<NamespaceInfo> = namespaces
        .items
        .iter()
        .map(|ns| {
            let name = ns.metadata.name.clone().unwrap_or_default();
            let status = ns
                .status
                .as_ref()
                .and_then(|s| s.phase.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            let labels = ns.metadata.labels.clone().unwrap_or_default();
            NamespaceInfo { name, status, labels }
        })
        .collect();

    // Get PVCs
    let pvcs = pvc_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list PVCs: {}", e))?;

    let mut total_bytes: i64 = 0;
    let pvc_infos: Vec<PvcInfo> = pvcs
        .items
        .iter()
        .map(|pvc| {
            let name = pvc.metadata.name.clone().unwrap_or_default();
            let namespace = pvc.metadata.namespace.clone().unwrap_or_default();
            
            let storage_class = pvc
                .spec
                .as_ref()
                .and_then(|s| s.storage_class_name.clone());
            
            let capacity = pvc
                .status
                .as_ref()
                .and_then(|s| s.capacity.as_ref())
                .and_then(|c| c.get("storage"))
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".to_string());
            
            let capacity_bytes = parse_capacity_to_bytes(&capacity);
            total_bytes += capacity_bytes;
            
            let status = pvc
                .status
                .as_ref()
                .and_then(|s| s.phase.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            
            let access_modes = pvc
                .spec
                .as_ref()
                .and_then(|s| s.access_modes.clone())
                .unwrap_or_default();
            
            let bound_to = pvc
                .spec
                .as_ref()
                .and_then(|s| s.volume_name.clone());

            PvcInfo {
                name,
                namespace,
                storage_class,
                capacity,
                capacity_bytes,
                status,
                access_modes,
                bound_to,
            }
        })
        .collect();

    let pvc_total_capacity = format_bytes(total_bytes);

    // Get empty namespaces
    let empty_namespaces = get_empty_namespaces(client).await.unwrap_or_default();

    info!(
        "Cluster overview: {} namespaces, {} PVCs ({}), {} empty namespaces",
        namespace_infos.len(),
        pvc_infos.len(),
        pvc_total_capacity,
        empty_namespaces.len()
    );

    Ok(ClusterOverview {
        namespace_count: namespace_infos.len(),
        namespaces: namespace_infos,
        pvc_count: pvc_infos.len(),
        pvc_total_capacity,
        pvcs: pvc_infos,
        empty_namespaces,
    })
}

/// Identifie les namespaces sans workload (Pod, Deployment, StatefulSet, DaemonSet, Job, CronJob) ni Ingress
pub async fn get_empty_namespaces(client: &Client) -> Result<Vec<String>, String> {
    let ns_api: Api<Namespace> = Api::all(client.clone());
    let namespaces = ns_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list namespaces: {}", e))?;

    let mut empty_ns = Vec::new();

    for ns in namespaces.items {
        let ns_name = ns.metadata.name.clone().unwrap_or_default();
        if ns_name == "kube-system" || ns_name == "kube-public" || ns_name == "kube-node-lease" {
            continue;
        }

        let lp = ListParams::default();
        
        // Check for Pods
        let pod_api: Api<Pod> = Api::namespaced(client.clone(), &ns_name);
        if !pod_api.list(&lp).await.map(|l| l.items.is_empty()).unwrap_or(true) {
            continue;
        }

        // Check for Deployments
        let deploy_api: Api<Deployment> = Api::namespaced(client.clone(), &ns_name);
        if !deploy_api.list(&lp).await.map(|l| l.items.is_empty()).unwrap_or(true) {
            continue;
        }

        // Check for StatefulSets
        let sts_api: Api<StatefulSet> = Api::namespaced(client.clone(), &ns_name);
        if !sts_api.list(&lp).await.map(|l| l.items.is_empty()).unwrap_or(true) {
            continue;
        }

        // Check for DaemonSets
        let ds_api: Api<DaemonSet> = Api::namespaced(client.clone(), &ns_name);
        if !ds_api.list(&lp).await.map(|l| l.items.is_empty()).unwrap_or(true) {
            continue;
        }

        // Check for Jobs
        let job_api: Api<Job> = Api::namespaced(client.clone(), &ns_name);
        if !job_api.list(&lp).await.map(|l| l.items.is_empty()).unwrap_or(true) {
            continue;
        }

        // Check for CronJobs
        let cj_api: Api<CronJob> = Api::namespaced(client.clone(), &ns_name);
        if !cj_api.list(&lp).await.map(|l| l.items.is_empty()).unwrap_or(true) {
            continue;
        }

        // Check for Ingresses
        let ingress_api: Api<Ingress> = Api::namespaced(client.clone(), &ns_name);
        if !ingress_api.list(&lp).await.map(|l| l.items.is_empty()).unwrap_or(true) {
            continue;
        }

        empty_ns.push(ns_name);
    }

    Ok(empty_ns)
}

/// Parse capacity string (e.g., "10Gi", "500Mi") to bytes
fn parse_capacity_to_bytes(capacity: &str) -> i64 {
    let trimmed = capacity.trim();
    
    if trimmed.ends_with("Ti") {
        let value: f64 = trimmed.trim_end_matches("Ti").parse().unwrap_or(0.0);
        (value * 1024.0 * 1024.0 * 1024.0 * 1024.0) as i64
    } else if trimmed.ends_with("Gi") {
        let value: f64 = trimmed.trim_end_matches("Gi").parse().unwrap_or(0.0);
        (value * 1024.0 * 1024.0 * 1024.0) as i64
    } else if trimmed.ends_with("Mi") {
        let value: f64 = trimmed.trim_end_matches("Mi").parse().unwrap_or(0.0);
        (value * 1024.0 * 1024.0) as i64
    } else if trimmed.ends_with("Ki") {
        let value: f64 = trimmed.trim_end_matches("Ki").parse().unwrap_or(0.0);
        (value * 1024.0) as i64
    } else {
        trimmed.parse().unwrap_or(0)
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: i64) -> String {
    const TI: i64 = 1024 * 1024 * 1024 * 1024;
    const GI: i64 = 1024 * 1024 * 1024;
    const MI: i64 = 1024 * 1024;

    if bytes >= TI {
        format!("{:.1}Ti", bytes as f64 / TI as f64)
    } else if bytes >= GI {
        format!("{:.1}Gi", bytes as f64 / GI as f64)
    } else if bytes >= MI {
        format!("{:.0}Mi", bytes as f64 / MI as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
