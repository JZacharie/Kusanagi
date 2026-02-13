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

        // Chat Use Case
        let cluster_repo: Arc<dyn crate::domain::ports::ClusterRepository> = Arc::new(
            crate::infrastructure::repositories::KubernetesClusterRepository::new(
                http_client.clone(),
            ),
        );
        let chat_use_case = Arc::new(crate::application::use_cases::ChatUseCase::new(
            cluster_repo,
            alert_repo.clone(),
        ));

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
        })
    }
}
