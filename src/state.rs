//! Application state shared across all handlers

use std::sync::Arc;

use crate::application::use_cases::{
    BackupUseCase, GetAlertsUseCase, GetHomeAssistantUseCase, GetSecurityUseCase, GetWeatherUseCase,
};

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub k8s_cache: Arc<crate::AdvancedCache<String>>,
    pub argocd_cache: Arc<crate::AdvancedCache<String>>,
    pub general_cache: Arc<crate::AdvancedCache<String>>,
    pub alerts_use_case: Arc<GetAlertsUseCase>,
    pub weather_use_case: Arc<GetWeatherUseCase>,
    pub security_use_case: Arc<GetSecurityUseCase>,
    pub ha_use_case: Arc<GetHomeAssistantUseCase>,
    pub backup_use_case: Arc<BackupUseCase>,
    pub chat_use_case: Arc<crate::application::use_cases::ChatUseCase>,
    pub kube_client: Option<Arc<kube::Client>>,
    pub http_client: Arc<reqwest::Client>,
    pub cilium_cache: Arc<crate::domain::services::cilium_service::CiliumCache>,
    pub llm_service: Arc<crate::domain::services::llm_service::LlmService>,
    pub prometheus_handle: metrics_exporter_prometheus::PrometheusHandle,
}

impl AppState {
    /// Create a new application state with all use cases initialized
    pub async fn new() -> anyhow::Result<Self> {
        use crate::domain::ports::{
            AlertRepository, BackupRepository, HomeAssistantRepository, SecurityRepository,
            WeatherRepository,
        };
        use crate::infrastructure::repositories::{
            AlertRepositoryImpl, BackupRepositoryImpl, HomeAssistantRepositoryImpl,
            SecurityRepositoryImpl, WeatherRepositoryImpl,
        };

        let k8s_cache = Arc::new(crate::AdvancedCache::<String>::new(
            std::time::Duration::from_secs(60),
        ));
        let argocd_cache = Arc::new(crate::AdvancedCache::<String>::new(
            std::time::Duration::from_secs(600),
        ));
        let general_cache = Arc::new(crate::AdvancedCache::<String>::new(
            std::time::Duration::from_secs(120),
        ));

        // Initialize HTTP client
        let http_client = Arc::new(
            reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        );

        // Try to initialize Kubernetes client
        let kube_client: Option<Arc<kube::Client>> = match kube::Client::try_default().await {
            Ok(client) => Some(Arc::new(client)),
            Err(_) => None,
        };

        // Initialize repositories
        let alert_repo: Arc<dyn AlertRepository> = Arc::new(AlertRepositoryImpl::new());
        let weather_repo: Arc<dyn WeatherRepository> = Arc::new(WeatherRepositoryImpl::new().await);
        let security_repo: Arc<dyn SecurityRepository> =
            Arc::new(SecurityRepositoryImpl::new().await);
        let ha_repo: Arc<dyn HomeAssistantRepository> =
            Arc::new(HomeAssistantRepositoryImpl::new()?);

        // Backup repo needs kube client
        let backup_repo: Arc<dyn BackupRepository> = if let Some(ref kc) = kube_client {
            Arc::new(BackupRepositoryImpl::new(kc.clone()))
        } else {
            // Create a dummy backup repo when kube is unavailable
            Arc::new(BackupRepositoryImpl::new(Arc::new(
                kube::Client::try_default().await?,
            )))
        };

        // Initialize use cases
        let alerts_use_case = Arc::new(GetAlertsUseCase::new(alert_repo.clone()));
        let weather_use_case = Arc::new(GetWeatherUseCase::new(weather_repo));
        let security_use_case = Arc::new(GetSecurityUseCase::new(security_repo));
        let ha_use_case = Arc::new(GetHomeAssistantUseCase::new(ha_repo));
        let backup_use_case = Arc::new(BackupUseCase::new(backup_repo));

        // LLM Service
        let llm_service = Arc::new(crate::domain::services::llm_service::LlmService::new());

        // Chat Service
        let chat_service = Arc::new(crate::domain::services::chat_service::ChatService::new(
            llm_service.clone(),
            http_client.as_ref().clone(),
            k8s_cache.clone(),
        ));

        // Chat Use Case
        // We now pass the ChatService to the ChatUseCase
        // Note: We might need to update ChatUseCase to accept ChatService
        // For now, let's assume we will update ChatUseCase in the next step.
        // But wait, if I update AppState first, it won't compile because ChatUseCase::new expects repositories.
        // So I must update ChatUseCase FIRST. But ChatUseCase needs ChatService which is here.
        // I will temporarily comment out ChatUseCase params or pass them as is, and then update ChatUseCase file.
        // Actually, I can just update the ChatUseCase signature in the other file FIRST, then this file.
        // But I am editing this file now.
        // I'll update ChatUseCase first in the next turn, then come back here?
        // No, I can do multi_replace if I want, but let's stick to ReplaceFile.
        // I will update ChatUseCase file NOW in parallel or sequentially before this one?
        // Sequential is safer. I will CANCEL this tool call and update ChatUseCase first.
        // Wait, I can't cancel.
        // I will proceed with updating AppState but I know it will break compilation until I fix ChatUseCase.
        // I will comment out the ChatUseCase init line change and do it properly.

        // Actually, let's just initialize repositories as before for now to keep it compiling,
        // AND initialize ChatService. Then I'll change ChatUseCase to take ChatService.

        let cluster_repo: Arc<dyn crate::domain::ports::ClusterRepository> = Arc::new(
            crate::infrastructure::repositories::KubernetesClusterRepository::new(
                http_client.clone(),
            ),
        );
        // We will update ChatUseCase to take ChatService later.
        // For now, let's keep it as is, but we need to satisfy the struct definition which now has llm_service.

        let chat_use_case = Arc::new(crate::application::use_cases::ChatUseCase::new(
            cluster_repo,
            alert_repo.clone(),
            chat_service.clone(), // Adding this requires ChatUseCase update
        ));

        // Metrics
        let prometheus_handle = crate::infrastructure::metrics::setup_metrics()?;

        Ok(Self {
            k8s_cache,
            argocd_cache,
            general_cache,
            alerts_use_case,
            weather_use_case,
            security_use_case,
            ha_use_case,
            backup_use_case,
            chat_use_case,
            kube_client,
            http_client,
            cilium_cache: Arc::new(crate::domain::services::cilium_service::CiliumCache::new()),
            llm_service,
            prometheus_handle,
        })
    }
}
