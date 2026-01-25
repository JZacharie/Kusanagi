use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt, ProcessExt};
use kube::{Client, Api, api::{Patch, PatchParams}};
use k8s_openapi::api::apps::v1::Deployment;
use tracing::{info, error, warn};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Serialize, Clone)]
pub struct SystemStatus {
    pub uptime_secs: u64,
    pub cpu_usage_percent: f32,
    pub memory_usage_bytes: u64,
    pub version: String,
    pub start_time: String,
}

pub struct SystemManager {
    pub start_time: Instant,
    pub start_time_rfc3339: String,
    pub last_image_digest: Arc<Mutex<Option<String>>>,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            start_time_rfc3339: chrono::Utc::now().to_rfc3339(),
            last_image_digest: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_status(&self) -> SystemStatus {
        let mut sys = System::new_all();
        sys.refresh_all();
        
        // Get current process metrics
        let pid = sysinfo::get_current_pid().unwrap();
        let cpu_usage = sys.process(pid).map(|p| p.cpu_usage()).unwrap_or(0.0);
        let memory_usage = sys.process(pid).map(|p| p.memory()).unwrap_or(0);

        SystemStatus {
            uptime_secs: self.start_time.elapsed().as_secs(),
            cpu_usage_percent: cpu_usage,
            memory_usage_bytes: memory_usage,
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

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(system_status_handler);
}
