use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use std::time::{Duration, Instant};
use sysinfo::System;
use kube::{Client, Api, api::{Patch, PatchParams}};
use k8s_openapi::api::apps::v1::Deployment;
use tracing::{info, error, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::utils::MutexExt;

#[derive(Serialize, Clone)]
pub struct SystemStatus {
    pub uptime_secs: u64,
    pub cpu_usage: f32,          // Changed from cpu_usage_percent
    pub memory_usage_mb: u64,    // Changed from memory_usage_bytes
    pub version: String,
    pub start_time: String,
}

pub struct SystemManager {
    pub start_time: Instant,
    pub start_time_rfc3339: String,
    pub last_image_digest: Arc<Mutex<Option<String>>>,
    sys: Arc<std::sync::Mutex<System>>, // Using std::sync::Mutex for synchronous sysinfo
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            start_time_rfc3339: chrono::Utc::now().to_rfc3339(),
            last_image_digest: Arc::new(Mutex::new(None)),
            sys: Arc::new(std::sync::Mutex::new(System::new_all())),
        }
    }

    pub fn get_status(&self) -> SystemStatus {
        let mut sys = self.sys.lock_safe();
        sys.refresh_all();
        
        // Get current process metrics
        let pid = sysinfo::get_current_pid().unwrap();
        let cpu_usage = sys.process(pid).map(|p| p.cpu_usage()).unwrap_or(0.0);
        let memory_usage = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

        SystemStatus {
            uptime_secs: self.start_time.elapsed().as_secs(),
            cpu_usage,
            memory_usage_mb: memory_usage / 1024 / 1024,
            version: env!("CARGO_PKG_VERSION").to_string(),
            start_time: self.start_time_rfc3339.clone(),
        }
    }
}

#[get("/api/system/status")]
pub async fn system_status_handler(manager: web::Data<SystemManager>) -> impl Responder {
    HttpResponse::Ok().json(manager.get_status())
}

/// Task to check for image updates in GHCR and trigger rollout
pub async fn start_auto_update_task(client: Client, last_digest: Arc<Mutex<Option<String>>>) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // Check every 5 minutes
    
    loop {
        interval.tick().await;
        info!("Checking for Kusanagi image updates...");

        match check_for_new_image(&client).await {
            Ok(Some(new_digest)) => {
                let mut current = last_digest.lock().await;
                if current.as_ref() != Some(&new_digest) {
                    if current.is_some() {
                        info!("New image detected! Digest: {}. Triggering rollout...", new_digest);
                        if let Err(e) = trigger_rollout(&client).await {
                            error!("Failed to trigger rollout: {}", e);
                        }
                    }
                    *current = Some(new_digest);
                }
            }
            Ok(None) => info!("No new image digest found or unable to check."),
            Err(e) => warn!("Auto-update check failed: {}", e),
        }
    }
}

async fn check_for_new_image(client: &Client) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Get current image digest from the deployment status
    let api: Api<Deployment> = Api::namespaced(client.clone(), "default"); // Assuming default namespace
    let _kusanagi_deploy = match api.get("kusanagi").await {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };

    // Note: In a real scenario, we might want to check the actual registry.
    // However, if we simply want to trigger a restart when a new image is pushed,
    // we can rely on the user or CI/CD to update the image tag, OR
    // we can try to hit GHCR API.
    
    // For this demonstration, we'll try to get the digest from GHCR if public.
    // ghcr.io/v2/jzacharie/kusanagi/manifests/latest
    let registry_url = "https://ghcr.io/v2/jzacharie/kusanagi/manifests/latest";
    let http_client = reqwest::Client::new();
    
    // We need to get a token for GHCR first if it's not wide open
    // For simplicity, we'll try HEAD request and check Docker-Content-Digest
    let response = http_client.head(registry_url)
        .header("Accept", "application/vnd.docker.distribution.manifest.v2+json")
        .send()
        .await?;

    if let Some(digest) = response.headers().get("docker-content-digest") {
        return Ok(Some(digest.to_str()?.to_string()));
    }

    Ok(None)
}

async fn trigger_rollout(client: &Client) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let api: Api<Deployment> = Api::namespaced(client.clone(), "default");
    
    let now = chrono::Utc::now().to_rfc3339();
    let patch = serde_json::json!({
        "spec": {
            "template": {
                "metadata": {
                    "annotations": {
                        "kusanagi.io/restartedAt": now
                    }
                }
            }
        }
    });

    api.patch("kusanagi", &PatchParams::default(), &Patch::Merge(&patch)).await?;
    info!("Rolling restart triggered successfully at {}", now);
    Ok(())
}

#[get("/api/system/logs")]
pub async fn system_logs_handler(data: web::Data<crate::AppState>) -> impl Responder {
    let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "kusanagi".to_string());
    let namespace = std::env::var("POD_NAMESPACE").unwrap_or_else(|_| "kusanagi".to_string());
    let dev_mode = std::env::var("DEV_MODE").unwrap_or_default() == "true";

    // 1. Try to get logs using hostname as pod name
    match crate::legacy::pods::get_pod_logs(&data.client, &namespace, &hostname, None, 1000).await {
        Ok(logs) => {
            if logs.is_empty() {
                return HttpResponse::Ok().body("No logs available (empty response from Kubernetes)");
            }
            HttpResponse::Ok().body(logs)
        },
        Err(e) => {
            warn!("Failed to fetch logs for pod {}: {}. Trying fallbacks...", hostname, e);
            
            // 2. Try to find pod by label
            let pods_api: Api<k8s_openapi::api::core::v1::Pod> = Api::namespaced(data.client.clone(), &namespace);
            
            // Strategy 2a: Try by label
            let lp = kube::api::ListParams::default().labels("app=kusanagi"); 
            if let Ok(pod_list) = pods_api.list(&lp).await {
                if let Some(pod) = pod_list.items.first() {
                    let pod_name = pod.metadata.name.clone().unwrap_or_default();
                    match crate::legacy::pods::get_pod_logs(&data.client, &namespace, &pod_name, None, 1000).await {
                        Ok(logs) => return HttpResponse::Ok().body(logs),
                        Err(inner_e) => warn!("Found pod {} by label but failed to fetch logs: {}", pod_name, inner_e),
                    }
                }
            }

            // Strategy 2b: Try by finding any pod with "kusanagi" in the name
            let lp_all = kube::api::ListParams::default();
            if let Ok(pod_list) = pods_api.list(&lp_all).await {
                if let Some(pod) = pod_list.items.iter().find(|p| p.metadata.name.as_deref().unwrap_or_default().contains("kusanagi")) {
                    let pod_name = pod.metadata.name.clone().unwrap_or_default();
                    info!("Found potential kusanagi pod: {}", pod_name);
                    match crate::legacy::pods::get_pod_logs(&data.client, &namespace, &pod_name, None, 1000).await {
                        Ok(logs) => return HttpResponse::Ok().body(logs),
                        Err(inner_e) => warn!("Found pod {} by name search but failed to fetch logs: {}", pod_name, inner_e),
                    }
                }
            }

            // Strategy 3: Try default namespace if different
            if namespace != "default" {
                if let Ok(logs) = crate::legacy::pods::get_pod_logs(&data.client, "default", &hostname, None, 1000).await {
                    return HttpResponse::Ok().body(logs);
                }
            }

            // Strategy 4: In dev mode or when K8s logs fail, return a friendly message
            let error_msg = if dev_mode {
                format!(
                    "=== Kusanagi Logs (Development Mode) ===\n\n\
                    Pod logs are not available in development mode.\n\
                    To see logs, run: cargo run 2>&1 | tee kusanagi.log\n\n\
                    Hostname: {}\n\
                    Namespace: {}\n\
                    Original error: {}",
                    hostname, namespace, e
                )
            } else {
                format!(
                    "=== Kusanagi Logs Unavailable ===\n\n\
                    Could not fetch logs from Kubernetes.\n\n\
                    Tried:\n\
                    - Pod: {} in namespace {}\n\
                    - Pods with label 'app=kusanagi'\n\
                    - Pods with name containing 'kusanagi'\n\n\
                    Error: {}\n\n\
                    Please check:\n\
                    1. The pod is running: kubectl get pods -n {}\n\
                    2. The pod has logs: kubectl logs -n {} {}\n\
                    3. K8s API is accessible",
                    hostname, namespace, e, namespace, namespace, hostname
                )
            };
            
            HttpResponse::Ok().content_type("text/plain").body(error_msg)
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(system_status_handler)
       .service(system_logs_handler);
}
