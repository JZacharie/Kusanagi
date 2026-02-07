use std::sync::Arc;
use std::time::Duration;

pub fn setup_rustls() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .ok();
}

pub fn setup_logging() {
    env_logger::Builder::from_default_env()
        .filter_module(
            "kusanagi::domain::services::proxmox_service",
            log::LevelFilter::Error,
        )
        .format_timestamp_millis()
        .init();
}

pub fn setup_caches() -> (
    Arc<crate::AdvancedCache<String>>,
    Arc<crate::AdvancedCache<String>>,
    Arc<crate::AdvancedCache<String>>,
) {
    let k8s_cache = Arc::new(crate::AdvancedCache::new(Duration::from_secs(60)));
    let argocd_cache = Arc::new(crate::AdvancedCache::new(Duration::from_secs(600)));
    let general_cache = Arc::new(crate::AdvancedCache::new(Duration::from_secs(120)));
    
    (k8s_cache, argocd_cache, general_cache)
}

pub fn setup_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default()
}

pub async fn setup_kube_client() -> Option<kube::Client> {
    match kube::Client::try_default().await {
        Ok(client) => {
            tracing::info!("✅ Kubernetes client initialized");
            Some(client)
        }
        Err(e) => {
            tracing::warn!("⚠️  Kubernetes not available: {}", e);
            None
        }
    }
}
