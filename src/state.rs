//! Application state shared across all handlers

use std::sync::Arc;

use crate::application::use_cases::{
    A2UIUseCase, BackupUseCase, GetAlertsUseCase, GetHomeAssistantUseCase, GetSecurityUseCase,
    GetWeatherUseCase,
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
    pub a2ui_use_case: Arc<A2UIUseCase>,
    pub business_use_case: Arc<crate::application::use_cases::GetBusinessOverviewUseCase>,
    pub kube_client: Option<Arc<kube::Client>>,
    pub http_client: Arc<reqwest::Client>,
    pub cilium_cache: Arc<crate::domain::services::cilium_service::CiliumCache>,
    pub llm_service: Arc<crate::domain::services::llm_service::LlmService>,
    pub mqtt_state: crate::domain::services::mqtt_service::MqttState,
    pub prometheus_handle: metrics_exporter_prometheus::PrometheusHandle,
    pub kubernetes_repository: Arc<dyn crate::domain::ports::KubernetesRepository>,
    pub ws_broadcast: tokio::sync::broadcast::Sender<
        crate::interfaces::http::handlers::core::websocket::NotificationMessage,
    >,
    pub namespace: String,
}

impl AppState {
    /// Create a new application state with all use cases initialized
    pub async fn new() -> anyhow::Result<Self> {
        use crate::domain::ports::{
            A2UIRepository, AlertRepository, BackupRepository, HomeAssistantRepository,
            SecurityRepository, WeatherRepository,
        };
        use crate::infrastructure::repositories::{
            A2UIRepositoryImpl, AlertRepositoryImpl, BackupRepositoryImpl,
            CloudflareRepositoryImpl, HomeAssistantRepositoryImpl, NoOpBackupRepository,
            SecurityRepositoryImpl, WeatherRepositoryImpl,
        };
        use crate::init::{setup_caches, setup_http_client_arc, setup_kube_client_arc};

        // Initialize caches and clients using init helpers
        let (k8s_cache, argocd_cache, general_cache) = setup_caches();
        let http_client = setup_http_client_arc();
        let kube_client = setup_kube_client_arc().await;
        let namespace =
            std::env::var("KUSANAGI_NAMESPACE").unwrap_or_else(|_| "unknown".to_string());

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
        let a2ui_repo: Arc<dyn A2UIRepository> =
            Arc::new(A2UIRepositoryImpl::new(general_cache.clone()));

        // Initialize use cases
        let alerts_use_case = Arc::new(GetAlertsUseCase::new(alert_repo.clone()));
        let weather_use_case = Arc::new(GetWeatherUseCase::new(weather_repo));
        let security_use_case = Arc::new(GetSecurityUseCase::new(security_repo));
        let ha_use_case = Arc::new(GetHomeAssistantUseCase::new(ha_repo));
        let backup_use_case = Arc::new(BackupUseCase::new(backup_repo, k8s_cache.clone()));
        let a2ui_use_case = Arc::new(A2UIUseCase::new(a2ui_repo));

        let cf_repo = Arc::new(CloudflareRepositoryImpl::new());
        let business_use_case =
            Arc::new(crate::application::use_cases::GetBusinessOverviewUseCase::new(cf_repo));

        // S3 & Transcription
        let s3_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .load()
            .await;
        let mut s3_builder = aws_sdk_s3::config::Builder::from(&s3_config);

        // Use custom verifier if needed (Minio usually)
        if std::env::var("S3_INSECURE")
            .map(|v| v == "true")
            .unwrap_or(false)
        {
            s3_builder = crate::infrastructure::s3_utils::configure_insecure_s3(s3_builder);
        }

        let s3_client = aws_sdk_s3::Client::from_conf(s3_builder.build());
        let s3_bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "kusanagi".to_string());
        let transcription_repo = Arc::new(
            crate::infrastructure::repositories::S3TranscriptionRepository::new(
                s3_client, s3_bucket,
            ),
        );

        // External MQTT Notification
        let notification_repo = Arc::new(
            crate::infrastructure::repositories::MqttNotificationRepository::new(
                "ipv4.zacharie.org".to_string(),
                1883,
                Some("joseph".to_string()),
                Some("2f21ZxB5JC6XfujK".to_string()),
                namespace.clone(),
            )
            .await,
        );

        // Initialize services
        let llm_service = Arc::new(crate::domain::services::llm_service::LlmService::new());

        let process_audio_use_case =
            Arc::new(crate::application::use_cases::ProcessAudioUseCase::new(
                llm_service.clone(),
                transcription_repo,
                notification_repo,
            ));
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

        // WebSocket Broadcast Channel
        let (ws_broadcast, _) = tokio::sync::broadcast::channel(100);

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
            a2ui_use_case,
            business_use_case,
            kube_client,
            http_client,
            cilium_cache: Arc::new(crate::domain::services::cilium_service::CiliumCache::new()),
            llm_service,
            mqtt_state: crate::domain::services::mqtt_service::MqttState::new()
                .with_namespace(namespace.clone())
                .with_process_audio(process_audio_use_case)
                .with_broadcast(ws_broadcast.clone()),
            prometheus_handle,
            kubernetes_repository,
            ws_broadcast,
            namespace,
        })
    }
}
