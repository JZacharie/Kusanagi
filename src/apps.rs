use kube::{
    api::{Api, ListParams},
    Client,
};
use k8s_openapi::api::core::v1::{Namespace, PersistentVolumeClaim, Pod, ResourceRequirements};
use serde::Serialize;
use std::collections::HashMap;
use tracing::info;

/// Application with resource usage
#[derive(Clone, Debug, Serialize)]
pub struct AppInfo {
    pub name: String,
    pub namespace: String,
    pub health_status: String,
    pub sync_status: String,
    pub argocd_url: String,
    // Resource usage
    pub pod_count: usize,
    pub ram_request: String,
    pub ram_limit: String,
    pub pvc_count: usize,
    pub pvc_size: String,
}

/// Response with all apps and their resources
#[derive(Clone, Debug, Serialize)]
pub struct AppsResponse {
    pub total_apps: usize,
    pub apps: Vec<AppInfo>,
}

/// Format bytes to human-readable
fn format_bytes(bytes: i64) -> String {
    if bytes >= 1024 * 1024 * 1024 * 1024 {
        format!("{:.1}Ti", bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}Gi", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.0}Mi", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0}Ki", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

/// Parse memory string to bytes
fn parse_memory(mem: &str) -> i64 {
    let mem = mem.trim();
    if mem.is_empty() {
        return 0;
    }
    
    let (num_str, unit) = if mem.ends_with("Ki") {
        (&mem[..mem.len()-2], 1024_i64)
    } else if mem.ends_with("Mi") {
        (&mem[..mem.len()-2], 1024_i64 * 1024)
    } else if mem.ends_with("Gi") {
        (&mem[..mem.len()-2], 1024_i64 * 1024 * 1024)
    } else if mem.ends_with("Ti") {
        (&mem[..mem.len()-2], 1024_i64 * 1024 * 1024 * 1024)
    } else if mem.ends_with('K') || mem.ends_with('k') {
        (&mem[..mem.len()-1], 1000_i64)
    } else if mem.ends_with('M') || mem.ends_with('m') {
        (&mem[..mem.len()-1], 1000_i64 * 1000)
    } else if mem.ends_with('G') || mem.ends_with('g') {
        (&mem[..mem.len()-1], 1000_i64 * 1000 * 1000)
    } else {
        (mem, 1_i64)
    };
    
    num_str.parse::<f64>().unwrap_or(0.0) as i64 * unit
}

/// Parse capacity string to bytes
fn parse_capacity(cap: &str) -> i64 {
    parse_memory(cap)
}

/// Get all ArgoCD applications with resource usage
pub async fn get_apps_with_resources() -> Result<AppsResponse, String> {
    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create K8s client: {}", e))?;

    info!("Fetching ArgoCD applications with resource usage");

    // Get ArgoCD applications
    let argocd_apps: Api<kube::api::DynamicObject> = Api::all_with(
        client.clone(),
        &kube::api::ApiResource {
            group: "argoproj.io".to_string(),
            version: "v1alpha1".to_string(),
            api_version: "argoproj.io/v1alpha1".to_string(),
            kind: "Application".to_string(),
            plural: "applications".to_string(),
        },
    );

    let apps = argocd_apps
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list applications: {}", e))?;

    // Get all pods grouped by namespace
    let pods_api: Api<Pod> = Api::all(client.clone());
    let pods = pods_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list pods: {}", e))?;

    // Build namespace -> pods map with memory
    let mut ns_pods: HashMap<String, Vec<&Pod>> = HashMap::new();
    for pod in &pods.items {
        let ns = pod.metadata.namespace.as_deref().unwrap_or("default");
        ns_pods.entry(ns.to_string()).or_default().push(pod);
    }

    // Get all PVCs grouped by namespace
    let pvcs_api: Api<PersistentVolumeClaim> = Api::all(client.clone());
    let pvcs = pvcs_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list PVCs: {}", e))?;

    // Build namespace -> PVCs map
    let mut ns_pvcs: HashMap<String, Vec<&PersistentVolumeClaim>> = HashMap::new();
    for pvc in &pvcs.items {
        let ns = pvc.metadata.namespace.as_deref().unwrap_or("default");
        ns_pvcs.entry(ns.to_string()).or_default().push(pvc);
    }

    let argocd_base_url = std::env::var("ARGOCD_URL")
        .unwrap_or_else(|_| "https://argocd.p.zacharie.org".to_string());

    let mut app_infos = Vec::new();

    for app in &apps.items {
        let name = app.metadata.name.as_deref().unwrap_or("unknown").to_string();
        
        // Get destination namespace from spec
        let dest_ns = app.data.get("spec")
            .and_then(|s| s.get("destination"))
            .and_then(|d| d.get("namespace"))
            .and_then(|n| n.as_str())
            .unwrap_or(&name)
            .to_string();

        // Get health and sync status
        let health_status = app.data.get("status")
            .and_then(|s| s.get("health"))
            .and_then(|h| h.get("status"))
            .and_then(|s| s.as_str())
            .unwrap_or("Unknown")
            .to_string();

        let sync_status = app.data.get("status")
            .and_then(|s| s.get("sync"))
            .and_then(|s| s.get("status"))
            .and_then(|s| s.as_str())
            .unwrap_or("Unknown")
            .to_string();

        // Calculate RAM usage for namespace
        let namespace_pods = ns_pods.get(&dest_ns).map(|v| v.as_slice()).unwrap_or(&[]);
        let pod_count = namespace_pods.len();
        
        let mut total_ram_request: i64 = 0;
        let mut total_ram_limit: i64 = 0;

        for pod in namespace_pods {
            if let Some(spec) = &pod.spec {
                for container in &spec.containers {
                    if let Some(resources) = &container.resources {
                        if let Some(requests) = &resources.requests {
                            if let Some(mem) = requests.get("memory") {
                                total_ram_request += parse_memory(&mem.0);
                            }
                        }
                        if let Some(limits) = &resources.limits {
                            if let Some(mem) = limits.get("memory") {
                                total_ram_limit += parse_memory(&mem.0);
                            }
                        }
                    }
                }
            }
        }

        // Calculate PVC size for namespace
        let namespace_pvcs = ns_pvcs.get(&dest_ns).map(|v| v.as_slice()).unwrap_or(&[]);
        let pvc_count = namespace_pvcs.len();
        
        let mut total_pvc_size: i64 = 0;
        for pvc in namespace_pvcs {
            if let Some(spec) = &pvc.spec {
                if let Some(resources) = &spec.resources {
                    if let Some(requests) = &resources.requests {
                        if let Some(storage) = requests.get("storage") {
                            total_pvc_size += parse_capacity(&storage.0);
                        }
                    }
                }
            }
        }

        app_infos.push(AppInfo {
            name: name.clone(),
            namespace: dest_ns,
            health_status,
            sync_status,
            argocd_url: format!("{}/applications/{}", argocd_base_url, name),
            pod_count,
            ram_request: format_bytes(total_ram_request),
            ram_limit: format_bytes(total_ram_limit),
            pvc_count,
            pvc_size: format_bytes(total_pvc_size),
        });
    }

    // Sort by RAM limit descending
    app_infos.sort_by(|a, b| {
        let a_ram = parse_memory(&a.ram_limit);
        let b_ram = parse_memory(&b.ram_limit);
        b_ram.cmp(&a_ram)
    });

    Ok(AppsResponse {
        total_apps: app_infos.len(),
        apps: app_infos,
    })
}
