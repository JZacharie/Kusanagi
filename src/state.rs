//! Application state shared across all handlers

use std::sync::Arc;

use crate::application::use_cases::{
    BackupUseCase, GetAlertsUseCase, GetHomeAssistantUseCase, GetSecurityUseCase, GetWeatherUseCase,
};
use crate::domain::entities::BackupsResponse;
use crate::domain::ports::BackupRepository;

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
    pub mqtt_state: crate::domain::services::mqtt_service::MqttState,
    pub prometheus_handle: metrics_exporter_prometheus::PrometheusHandle,
    pub kubernetes_repository: Arc<dyn crate::domain::ports::KubernetesRepository>,
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
            NoOpBackupRepository, SecurityRepositoryImpl, WeatherRepositoryImpl,
        };
        use crate::init::{setup_caches, setup_http_client_arc, setup_kube_client_arc};

        // Initialize caches and clients using init helpers
        let (k8s_cache, argocd_cache, general_cache) = setup_caches();
        let http_client = setup_http_client_arc();
        let kube_client = setup_kube_client_arc().await;

        // Initialize repositories
        let alert_repo: Arc<dyn AlertRepository> = Arc::new(AlertRepositoryImpl::new());
        let weather_repo: Arc<dyn WeatherRepository> = Arc::new(WeatherRepositoryImpl::new().await);
        let security_repo: Arc<dyn SecurityRepository> =
            Arc::new(SecurityRepositoryImpl::new().await);
        let ha_repo: Arc<dyn HomeAssistantRepository> =
            Arc::new(HomeAssistantRepositoryImpl::new()?);

        // Backup repo with proper fallback
        let backup_repo: Arc<dyn BackupRepository> = if let Some(ref kc) = kube_client {
            Arc::new(BackupRepositoryImpl::new(kc.clone()))
        } else {
            tracing::warn!("Kubernetes not available - using NoOp backup repository");
            Arc::new(NoOpBackupRepository)
        };

        // Initialize use cases
        let alerts_use_case = Arc::new(GetAlertsUseCase::new(alert_repo.clone()));
        let weather_use_case = Arc::new(GetWeatherUseCase::new(weather_repo));
        let security_use_case = Arc::new(GetSecurityUseCase::new(security_repo));
        let ha_use_case = Arc::new(GetHomeAssistantUseCase::new(ha_repo));
        let backup_use_case = Arc::new(BackupUseCase::new(backup_repo, k8s_cache.clone()));

        // Services
        let llm_service = Arc::new(crate::domain::services::llm_service::LlmService::new());
        let chat_service = Arc::new(crate::domain::services::chat_service::ChatService::new(
            llm_service.clone(),
            http_client.as_ref().clone(),
            k8s_cache.clone(),
            kube_client.clone(),
        ));

        let cluster_repo: Arc<dyn crate::domain::ports::ClusterRepository> = Arc::new(
            crate::infrastructure::repositories::KubernetesClusterRepository::new(
                http_client.clone(),
                k8s_cache.clone(),
            ),
        );

        let chat_use_case = Arc::new(crate::application::use_cases::ChatUseCase::new(
            cluster_repo,
            alert_repo.clone(),
            chat_service.clone(),
        ));

        // Metrics
        let prometheus_handle = crate::infrastructure::metrics::setup_metrics()?;

        let kubernetes_repository = Arc::new(
            crate::infrastructure::repositories::KubernetesRepositoryImpl::new(
                http_client.clone(),
                kube_client.clone(),
                k8s_cache.clone(),
            ),
        );

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
            mqtt_state: crate::domain::services::mqtt_service::MqttState::new(),
            prometheus_handle,
            kubernetes_repository,
        })
    }
}
