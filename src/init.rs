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

#[allow(clippy::type_complexity)]
pub fn setup_caches() -> (
    Arc<crate::AdvancedCache<String>>,
    Arc<crate::AdvancedCache<String>>,
    Arc<crate::AdvancedCache<String>>,
) {
    // Increased TTL to reduce API pressure on Kubernetes
    let k8s_cache = Arc::new(crate::AdvancedCache::new(Duration::from_secs(300))); // 5 minutes
    let argocd_cache = Arc::new(crate::AdvancedCache::new(Duration::from_secs(600))); // 10 minutes
    let general_cache = Arc::new(crate::AdvancedCache::new(Duration::from_secs(300))); // 5 minutes

    (k8s_cache, argocd_cache, general_cache)
}

pub fn setup_http_client_arc() -> Arc<reqwest::Client> {
    Arc::new(
        reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default(),
    )
}

pub async fn setup_kube_client_arc() -> Option<Arc<kube::Client>> {
    match kube::Client::try_default().await {
        Ok(client) => {
            tracing::info!("✅ Kubernetes client initialized");
            Some(Arc::new(client))
        }
        Err(e) => {
            tracing::warn!("⚠️  Kubernetes not available: {}", e);
            None
        }
    }
}
