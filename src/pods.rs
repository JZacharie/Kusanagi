use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, DeleteParams, ListParams, Patch, PatchParams},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

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
pub async fn get_pods_status() -> Result<PodsStatusResponse, String> {
    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create Kubernetes client: {}", e))?;

    let pods_api: Api<Pod> = Api::all(client);

    let pods = pods_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list pods: {}", e))?;

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
pub async fn force_delete_pod(namespace: &str, pod_name: &str) -> Result<ForceDeleteResponse, String> {
    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create Kubernetes client: {}", e))?;

    let pods_api: Api<Pod> = Api::namespaced(client, namespace);

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
