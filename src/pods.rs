use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::{Pod, Service};
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet};
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams, LogParams},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, error, warn};
use tokio::time::{timeout, Duration};
use crate::AppState;

/// Request for logs
#[derive(Deserialize)]
pub struct LogsQuery {
    pub container: Option<String>,
    pub tail: Option<i64>,
}

/// Get logs for a specific pod
#[get("/api/pods/{namespace}/{name}/logs")]
pub async fn get_pod_logs_handler(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
    query: web::Query<LogsQuery>,
) -> impl Responder {
    let (namespace, name) = path.into_inner();
    let container = query.container.clone();
    let tail = query.tail.unwrap_or(200);

    match get_pod_logs(&data.client, &namespace, &name, container, tail).await {
        Ok(logs) => HttpResponse::Ok().body(logs),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e})),
    }
}

pub async fn get_pod_logs(
    client: &Client,
    namespace: &str,
    name: &str,
    container: Option<String>,
    tail_lines: i64,
) -> Result<String, String> {
    let pods: Api<Pod> = Api::namespaced(client.clone(), namespace);
    let lp = LogParams {
        container,
        tail_lines: Some(tail_lines),
        ..LogParams::default()
    };

    pods.logs(name, &lp)
        .await
        .map_err(|e| format!("Failed to fetch logs: {}", e))
}

#[derive(Deserialize)]
pub struct ScaleRequest {
    pub replicas: i32,
}

/// Scale a deployment or statefulset
#[post("/api/scale/{type}/{namespace}/{name}")]
pub async fn scale_resource_handler(
    data: web::Data<AppState>,
    path: web::Path<(String, String, String)>,
    body: web::Json<ScaleRequest>,
) -> impl Responder {
    let (resource_type, namespace, name) = path.into_inner();
    let replicas = body.replicas;

    let result = match resource_type.as_str() {
        "deployment" => scale_deployment(&data.client, &namespace, &name, replicas).await,
        "statefulset" => scale_statefulset(&data.client, &namespace, &name, replicas).await,
        _ => Err("Invalid resource type".to_string()),
    };

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({"status": "success", "message": format!("Scaled {} to {}", name, replicas)})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e})),
    }
}

async fn scale_deployment(client: &Client, namespace: &str, name: &str, replicas: i32) -> Result<(), String> {
    let api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    let patch = json!({
        "spec": {
            "replicas": replicas
        }
    });

    api.patch(name, &PatchParams::default(), &Patch::Merge(&patch))
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to scale deployment: {}", e))
}

async fn scale_statefulset(client: &Client, namespace: &str, name: &str, replicas: i32) -> Result<(), String> {
    let api: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
    let patch = json!({
        "spec": {
            "replicas": replicas
        }
    });

    api.patch(name, &PatchParams::default(), &Patch::Merge(&patch))
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to scale statefulset: {}", e))
}

#[get("/api/pods/status")]
pub async fn pods_status(data: web::Data<AppState>) -> impl Responder {
    match get_pods_status(&data.client).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e})),
    }
}

#[post("/api/pods/force-delete")]
pub async fn force_delete_pod_handler(
    data: web::Data<AppState>,
    body: web::Json<ForceDeleteRequest>,
) -> impl Responder {
    match force_delete_pod(&data.client, &body.namespace, &body.pod_name).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e})),
    }
}

/// Pods status response
#[derive(Clone, Debug, Serialize)]
pub struct PodsStatusResponse {
    pub total_pods: usize,
    pub running_pods: usize,
    pub pending_pods: usize,
    pub succeeded_pods: usize,
    pub failed_pods: usize,
    pub error_pods: usize,
    pub pods_in_error: Vec<PodInfo>,
}

/// Individual pod information  
#[derive(Clone, Debug, Serialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub node: Option<String>,
    pub restart_count: i32,
    pub age: String,
    pub age_seconds: i64,
    pub containers: Vec<ContainerInfo>,
    // Resource usage from Prometheus
    pub cpu_usage: Option<f64>,      // in cores
    pub memory_usage: Option<i64>,   // in bytes
    // Resource limits from Pod Spec (Sum of containers)
    pub cpu_limit: Option<f64>,      // in cores
    pub memory_limit: Option<i64>,   // in bytes
    pub cpu_request: Option<f64>,    // in cores
    pub memory_request: Option<i64>, // in bytes
}

/// Container status information
#[derive(Clone, Debug, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub ready: bool,
    pub restart_count: i32,
    pub state: String,
    pub reason: Option<String>,
    pub message: Option<String>,
}

/// Error reasons we want to detect
const ERROR_REASONS: &[&str] = &[
    "CrashLoopBackOff",
    "ImagePullBackOff",
    "ErrImagePull",
    "CreateContainerConfigError",
    "CreateContainerError",
    "RunContainerError",
    "OOMKilled",
    "Error",
    "InvalidImageName",
    "ContainerCannotRun",
    "DeadlineExceeded",
    "Evicted",
];

/// Get pods status with focus on error pods
pub async fn get_pods_status(client: &Client) -> Result<PodsStatusResponse, String> {

    // let _services: Api<Service> = Api::all(client.clone());
    let pods_api: Api<Pod> = Api::all(client.clone());

    let start = std::time::Instant::now();
    info!("Starting to list pods (timeout: 10s)...");

    let result = timeout(Duration::from_secs(10), pods_api.list(&ListParams::default())).await;

    let pods = match result {
        Ok(Ok(p)) => p,
        Ok(Err(e)) => {
            error!("Failed to list pods: {}", e);
            return Err(format!("Failed to list pods: {}", e));
        }
        Err(_) => {
            error!("K8s API list pods timed out after 10s");
            return Err("K8s API list pods timed out".to_string());
        }
    };

    let duration = start.elapsed();
    info!("K8s API list pods took: {:?}", duration);

    // Fetch resource usage from Prometheus (best effort)
    let usage_map = crate::prometheus::get_pods_resource_usage().await.unwrap_or_else(|_| std::collections::HashMap::new());


    let now = Utc::now();
    let mut response = PodsStatusResponse {
        total_pods: pods.items.len(),
        running_pods: 0,
        pending_pods: 0,
        succeeded_pods: 0,
        failed_pods: 0,
        error_pods: 0,
        pods_in_error: Vec::new(),
    };

    for pod in pods.items {
        let name = pod.metadata.name.clone().unwrap_or_default();
        let namespace = pod.metadata.namespace.clone().unwrap_or_default();
        
        // Skip metrics proxy pods from the error list as requested by user
        if name.contains("k3s-metrics-proxy") {
            continue;
        }
        
        let status = pod.status.as_ref();
        let spec = pod.spec.as_ref();
        
        let phase = status
            .and_then(|s| s.phase.as_ref())
            .map(|p| p.as_str())
            .unwrap_or("Unknown");

        // Count by phase
        match phase {
            "Running" => response.running_pods += 1,
            "Pending" => response.pending_pods += 1,
            "Succeeded" => response.succeeded_pods += 1,
            "Failed" => response.failed_pods += 1,
            _ => {}
        }

        // Get node name
        let node = spec.and_then(|s| s.node_name.clone());

        // Calculate age
        let (age, age_seconds) = pod
            .metadata
            .creation_timestamp
            .as_ref()
            .and_then(|ts| {
                DateTime::parse_from_rfc3339(&ts.0.to_rfc3339()).ok().map(|dt| {
                    let duration = now.signed_duration_since(dt.with_timezone(&Utc));
                    let seconds = duration.num_seconds();
                    (format_age(seconds), seconds)
                })
            })
            .unwrap_or(("Unknown".to_string(), 0));

        // Analyze container statuses
        let mut containers: Vec<ContainerInfo> = Vec::new();
        let mut total_restarts: i32 = 0;
        let mut pod_error_reason: Option<String> = None;
        let mut pod_error_message: Option<String> = None;
        let mut is_error_pod = false;

        // Check if phase indicates error
        if phase == "Failed" {
            is_error_pod = true;
            pod_error_reason = status.and_then(|s| s.reason.clone());
            pod_error_message = status.and_then(|s| s.message.clone());
        }

        // Check container statuses
        if let Some(container_statuses) = status.and_then(|s| s.container_statuses.as_ref()) {
            for cs in container_statuses {
                total_restarts += cs.restart_count;
                
                let (state, reason, message) = get_container_state_info(cs);
                
                // Check for error reasons
                if let Some(ref r) = reason {
                    if ERROR_REASONS.iter().any(|er| r.contains(er)) {
                        is_error_pod = true;
                        if pod_error_reason.is_none() {
                            pod_error_reason = reason.clone();
                            pod_error_message = message.clone();
                        }
                    }
                }
                
                containers.push(ContainerInfo {
                    name: cs.name.clone(),
                    ready: cs.ready,
                    restart_count: cs.restart_count,
                    state,
                    reason,
                    message,
                });
            }
        }

        // Check init container statuses
        if let Some(init_container_statuses) = status.and_then(|s| s.init_container_statuses.as_ref()) {
            for cs in init_container_statuses {
                let (state, reason, message) = get_container_state_info(cs);
                
                // Check for error reasons in init containers
                if let Some(ref r) = reason {
                    if ERROR_REASONS.iter().any(|er| r.contains(er)) {
                        is_error_pod = true;
                        if pod_error_reason.is_none() {
                            pod_error_reason = reason.clone();
                            pod_error_message = message.clone();
                        }
                    }
                }
                
                containers.push(ContainerInfo {
                    name: format!("init:{}", cs.name),
                    ready: cs.ready,
                    restart_count: cs.restart_count,
                    state,
                    reason,
                    message,
                });
            }
        }

        // Check for high restart count (>5 is concerning)
        if total_restarts > 5 && !is_error_pod {
            is_error_pod = true;
            pod_error_reason = Some(format!("HighRestartCount ({})", total_restarts));
        }

        // Add to error list if applicable
        if is_error_pod {
            response.error_pods += 1;
            response.pods_in_error.push(PodInfo {
                name,
                namespace,
                status: phase.to_string(),
                reason: pod_error_reason,
                message: pod_error_message,
                node,
                restart_count: total_restarts,
                age,
                age_seconds,
                containers,
                cpu_usage: usage_map.get(&(namespace.clone(), name.clone())).map(|u| u.0),
                memory_usage: usage_map.get(&(namespace.clone(), name.clone())).map(|u| u.1),
                cpu_limit: get_pod_resource_sum(spec, "limits", "cpu"),
                memory_limit: get_pod_resource_sum(spec, "limits", "memory"),
                cpu_request: get_pod_resource_sum(spec, "requests", "cpu"),
                memory_request: get_pod_resource_sum(spec, "requests", "memory"),
            });
        }
    }

    // Sort error pods by restart count (highest first), then by age (newest first)
    response.pods_in_error.sort_by(|a, b| {
        b.restart_count.cmp(&a.restart_count)
            .then_with(|| a.age_seconds.cmp(&b.age_seconds))
    });

    info!(
        "Pods status: {} total, {} running, {} error",
        response.total_pods, response.running_pods, response.error_pods
    );

    Ok(response)
}

/// Extract container state information
fn get_container_state_info(cs: &k8s_openapi::api::core::v1::ContainerStatus) -> (String, Option<String>, Option<String>) {
    if let Some(state) = &cs.state {
        if let Some(_running) = &state.running {
            return ("Running".to_string(), None, None);
        }
        if let Some(waiting) = &state.waiting {
            return (
                "Waiting".to_string(),
                waiting.reason.clone(),
                waiting.message.clone(),
            );
        }
        if let Some(terminated) = &state.terminated {
            return (
                "Terminated".to_string(),
                terminated.reason.clone(),
                terminated.message.clone(),
            );
        }
    }
    ("Unknown".to_string(), None, None)
}

/// Format age in human readable format
fn format_age(seconds: i64) -> String {
    if seconds < 0 {
        return "just now".to_string();
    }

    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{}d{}h", days, hours)
    } else if hours > 0 {
        format!("{}h{}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m", minutes)
    } else {
        format!("{}s", seconds)
    }
}

/// Request to force delete a pod
#[derive(Clone, Debug, Deserialize)]
pub struct ForceDeleteRequest {
    pub namespace: String,
    pub pod_name: String,
}

/// Response from force delete operation
#[derive(Clone, Debug, Serialize)]
pub struct ForceDeleteResponse {
    pub success: bool,
    pub message: String,
    pub pod_name: String,
    pub namespace: String,
}

/// Force delete a pod by removing finalizers and deleting with 0 grace period
/// This is useful for pods stuck in Terminating state
pub async fn force_delete_pod(client: &Client, namespace: &str, pod_name: &str) -> Result<ForceDeleteResponse, String> {

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), namespace);

    info!("Force deleting pod {}/{}", namespace, pod_name);

    // Step 1: Remove all finalizers using JSON Patch
    let patch = json!({
        "metadata": {
            "finalizers": null
        }
    });

    match pods_api
        .patch(
            pod_name,
            &PatchParams::default(),
            &Patch::Merge(&patch),
        )
        .await
    {
        Ok(_) => info!("Removed finalizers from pod {}/{}", namespace, pod_name),
        Err(e) => {
            // Pod might not exist or might not have finalizers, continue anyway
            info!("Note: Could not patch finalizers for {}/{}: {}", namespace, pod_name, e);
        }
    }

    // Step 2: Delete the pod with grace_period_seconds = 0
    let delete_params = DeleteParams {
        grace_period_seconds: Some(0),
        ..Default::default()
    };

    match pods_api.delete(pod_name, &delete_params).await {
        Ok(_) => {
            info!("Successfully force deleted pod {}/{}", namespace, pod_name);
            Ok(ForceDeleteResponse {
                success: true,
                message: format!("Pod {} successfully force deleted", pod_name),
                pod_name: pod_name.to_string(),
                namespace: namespace.to_string(),
            })
        }
        Err(e) => {
            let error_msg = format!("Failed to delete pod {}/{}: {}", namespace, pod_name, e);
            tracing::error!("{}", error_msg);
            Ok(ForceDeleteResponse {
                success: false,
                message: error_msg,
                pod_name: pod_name.to_string(),
                namespace: namespace.to_string(),
            })
        }
    }
}

/// Response for bulk delete operation
#[derive(Clone, Debug, Serialize)]
pub struct BulkDeleteResponse {
    pub success: bool,
    pub message: String,
    pub deleted_count: usize,
    pub failed_count: usize,
}

#[post("/api/pods/delete-error-pods")]
pub async fn delete_error_pods_handler(
    data: web::Data<AppState>,
) -> impl Responder {
    match delete_error_pods(&data.client).await {
        Ok(res) => HttpResponse::Ok().json(res),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e})),
    }
}

/// Delete all pods in Error/Failed state with a 4s delay between each
pub async fn delete_error_pods(client: &Client) -> Result<BulkDeleteResponse, String> {
    info!("Starting bulk deletion of error pods");
    
    // Get current status to find error pods
    let status = get_pods_status(client).await?;
    
    let total_error_pods = status.pods_in_error.len();
    info!("Found {} pods in error state", total_error_pods);
    
    let mut deleted_count = 0;
    let mut failed_count = 0;
    
    for (index, pod) in status.pods_in_error.iter().enumerate() {
        info!("Processing pod {}/{} ({}/{})", index + 1, total_error_pods, pod.namespace, pod.name);
        
        match force_delete_pod(client, &pod.namespace, &pod.name).await {
            Ok(res) => {
                if res.success {
                    deleted_count += 1;
                } else {
                    failed_count += 1;
                }
            },
            Err(_) => failed_count += 1,
        }
        
        // Wait 4 seconds between deletions, but not after the last one
        if index < total_error_pods - 1 {
            info!("Waiting 4s before next deletion...");
            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        }
    }
    
    Ok(BulkDeleteResponse {
        success: true,
        message: format!("Processed {} error pods. Deleted: {}, Failed: {}", total_error_pods, deleted_count, failed_count),
        deleted_count,
        failed_count,
    })
}
}

/// Parse k8s CPU quantity to cores (f64)
/// 100m -> 0.1
/// 1 -> 1.0
fn parse_cpu(s: &str) -> f64 {
    if let Some(stripped) = s.strip_suffix('m') {
        stripped.parse::<f64>().unwrap_or(0.0) / 1000.0
    } else {
        s.parse::<f64>().unwrap_or(0.0)
    }
}

/// Parse k8s memory quantity to bytes (i64)
/// 128Mi -> 128 * 1024 * 1024
/// 1G -> 1 * 1000 * 1000 * 1000
fn parse_memory(s: &str) -> i64 {
    let s = s.trim();
    if let Some(stripped) = s.strip_suffix("Ki") {
        stripped.parse::<f64>().unwrap_or(0.0) as i64 * 1024
    } else if let Some(stripped) = s.strip_suffix("Mi") {
        stripped.parse::<f64>().unwrap_or(0.0) as i64 * 1024 * 1024
    } else if let Some(stripped) = s.strip_suffix("Gi") {
        stripped.parse::<f64>().unwrap_or(0.0) as i64 * 1024 * 1024 * 1024
    } else if let Some(stripped) = s.strip_suffix("Ti") {
        stripped.parse::<f64>().unwrap_or(0.0) as i64 * 1024 * 1024 * 1024 * 1024
    } else if let Some(stripped) = s.strip_suffix("m") {
        // e.g. 100m bytes? Rare but possible in some generic resource contexts, usually invalid for memory
        (stripped.parse::<f64>().unwrap_or(0.0) / 1000.0) as i64
    } else if let Some(stripped) = s.strip_suffix("K") {
        stripped.parse::<f64>().unwrap_or(0.0) as i64 * 1000
    } else if let Some(stripped) = s.strip_suffix("M") {
        stripped.parse::<f64>().unwrap_or(0.0) as i64 * 1000 * 1000
    } else if let Some(stripped) = s.strip_suffix("G") {
        stripped.parse::<f64>().unwrap_or(0.0) as i64 * 1000 * 1000 * 1000
    } else if let Some(stripped) = s.strip_suffix("T") {
        stripped.parse::<f64>().unwrap_or(0.0) as i64 * 1000 * 1000 * 1000 * 1000
    } else {
        s.parse::<i64>().unwrap_or(0)
    }
}


fn get_pod_resource_sum(spec: Option<&k8s_openapi::api::core::v1::PodSpec>, req_type: &str, resource_name: &str) -> Option<f64> {
    let spec = spec?;
    let mut sum: f64 = 0.0;
    
    // Sum containers
    for container in &spec.containers {
         if let Some(resources) = &container.resources {
            let map = if req_type == "limits" { &resources.limits } else { &resources.requests };
             if let Some(map) = map {
                if let Some(qty) = map.get(resource_name) {
                    if resource_name == "cpu" {
                        sum += parse_cpu(&qty.0);
                    } else {
                         sum += parse_memory(&qty.0) as f64;
                    }
                }
             }
         }
    }
    
    // Sum init containers
    if let Some(init_containers) = &spec.init_containers {
        // Init containers run sequentially, so the requirement is the MAX of any init container
        // But for "limits" usually we care about the max spike. 
        // For sizing, it's complex (max(init) + sum(app)).
        // Simplified: just taking the app containers sum for now as that's the steady state.
        // User asked for "resources limits", which usually implies what the pod is reserving/capped at during runtime.
        // I will ignore init containers for the sum to show the "App" limits.
    }
    
    if sum > 0.0 { Some(sum) } else { None }
}
