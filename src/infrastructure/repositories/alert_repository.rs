//! Alert Repository Implementation
//!
//! Infrastructure adapter implementing the AlertRepository port.
//! Handles Alertmanager API calls with caching.

use crate::domain::entities::{Alert, AlertsResponse};
use crate::domain::ports::AlertRepository;
use crate::domain::services::alert_service::AlertDomainService;
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
// chrono types not used directly
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

const DEFAULT_ALERTMANAGER_URL: &str =
    "http://kube-prometheus-stack-alertmanager.kube-prometheus-stack.svc:9093";
const CACHE_TTL_SECONDS: u64 = 120;

/// Alertmanager API response structures
#[derive(Debug, Deserialize)]
struct AmAlert {
    labels: HashMap<String, String>,
    annotations: HashMap<String, String>,
    #[serde(rename = "startsAt")]
    starts_at: String,
    #[serde(rename = "endsAt")]
    _ends_at: String,
    #[serde(rename = "generatorURL")]
    generator_url: String,
    fingerprint: String,
    status: AmAlertStatus,
}

#[derive(Debug, Deserialize)]
struct AmAlertStatus {
    state: String,
}

/// Cache for alert data
pub struct AlertsCache {
    pub alerts: RwLock<Option<(AlertsResponse, Instant)>>,
}

impl Default for AlertsCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AlertsCache {
    pub fn new() -> Self {
        Self {
            alerts: RwLock::new(None),
        }
    }
}

static ALERTS_CACHE: OnceLock<AlertsCache> = OnceLock::new();

fn get_alerts_cache() -> &'static AlertsCache {
    ALERTS_CACHE.get_or_init(AlertsCache::new)
}

/// Alert repository implementation
pub struct AlertRepositoryImpl {
    client: reqwest::Client,
    alertmanager_url: String,
    domain_service: AlertDomainService,
    username: Option<String>,
    password: Option<String>,
}

impl AlertRepositoryImpl {
    /// Create a new repository instance
    pub fn new() -> Self {
        let alertmanager_url = std::env::var("ALERTMANAGER_URL").unwrap_or_else(|_| {
            warn!("ALERTMANAGER_URL not set, using default local K8s service URL");
            DEFAULT_ALERTMANAGER_URL.to_string()
        });

        let username = std::env::var("ALERTMANAGER_USERNAME").ok();
        let password = std::env::var("ALERTMANAGER_PASSWORD").ok();

        if username.is_none() || password.is_none() {
            warn!("No Alertmanager credentials found in env vars (ALERTMANAGER_USERNAME/PASSWORD)");
        }

        let client = reqwest::Client::new();

        Self {
            client,
            alertmanager_url,
            domain_service: AlertDomainService::new(),
            username,
            password,
        }
    }

    /// Get Alertmanager URL
    pub fn get_alertmanager_url(&self) -> &str {
        &self.alertmanager_url
    }

    /// Check if running in local mode
    fn check_local_mode(&self) -> bool {
        std::env::var("KUSANAGI_MODE").unwrap_or_default() == "local"
    }

    /// Fetch alerts from Alertmanager API
    async fn fetch_from_alertmanager(&self) -> Result<Vec<AmAlert>> {
        let url = format!("{}/api/v2/alerts", self.alertmanager_url);

        let mut request = self.client.get(&url);
        request = request.query(&[
            ("active", "true"),
            ("silenced", "false"),
            ("inhibited", "false"),
        ]);

        // Add Basic Auth if credentials are provided
        if let (Some(ref username), Some(ref password)) = (&self.username, &self.password) {
            debug!("🔑 Using Alertmanager Basic Auth (User: {})", username);
            request = request.basic_auth(username, Some(password));
        }

        let response = request
            .timeout(Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| {
                KusanagiError::external_service(format!("Alertmanager request failed: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(KusanagiError::external_service(format!(
                "Alertmanager returned status: {}",
                response.status()
            )));
        }

        let am_alerts: Vec<AmAlert> = response.json().await.map_err(|e| {
            KusanagiError::serialization(format!("Failed to parse Alertmanager response: {}", e))
        })?;

        Ok(am_alerts)
    }

    /// Transform API alerts to domain alerts
    fn transform_alerts(&self, am_alerts: Vec<AmAlert>) -> AlertsResponse {
        let alerts: Vec<Alert> = am_alerts
            .into_iter()
            .map(|am_alert| {
                let severity = self
                    .domain_service
                    .parse_severity(am_alert.labels.get("severity").map(|s| s.as_str()))
                    .to_string();

                let name = self.domain_service.extract_alert_name(&am_alert.labels);
                let namespace = self.domain_service.extract_namespace(&am_alert.labels);
                let pod = self.domain_service.extract_pod(&am_alert.labels);
                let summary = self.domain_service.extract_summary(&am_alert.annotations);
                let description = self
                    .domain_service
                    .extract_description(&am_alert.annotations);
                let started_at = self.domain_service.parse_datetime(&am_alert.starts_at);

                self.domain_service.build_alert(
                    name,
                    severity,
                    am_alert.status.state.clone(),
                    summary,
                    description,
                    namespace,
                    pod,
                    started_at,
                    am_alert.fingerprint,
                    Some(am_alert.generator_url),
                )
            })
            .collect();

        self.domain_service.categorize_alerts(alerts)
    }

    /// Update cache with new alerts
    async fn update_cache(&self, alerts: AlertsResponse) {
        let mut cache = get_alerts_cache().alerts.write().await;
        *cache = Some((alerts, Instant::now()));
        debug!("✅ Updated Alertmanager cache");
    }

    /// Get from cache if valid
    async fn get_from_cache(&self) -> Option<AlertsResponse> {
        let cache = get_alerts_cache().alerts.read().await;
        if let Some((ref alerts, timestamp)) = *cache {
            if timestamp.elapsed() < Duration::from_secs(CACHE_TTL_SECONDS) {
                debug!("Returning cached alerts");
                return Some(alerts.clone());
            }
        }
        None
    }
}

#[async_trait]
impl AlertRepository for AlertRepositoryImpl {
    async fn get_active_alerts(&self) -> Result<AlertsResponse> {
        // Check local mode
        if self.check_local_mode() {
            info!("AlertManager running in local mode, returning mock alerts");
            return Ok(self.domain_service.create_mock_alerts());
        }

        // Fetch from API
        let am_alerts = self.fetch_from_alertmanager().await?;
        let alerts = self.transform_alerts(am_alerts);

        // Update cache
        self.update_cache(alerts.clone()).await;

        Ok(alerts)
    }

    async fn get_cached_alerts(&self) -> Result<AlertsResponse> {
        // Check local mode
        if self.check_local_mode() {
            return Ok(self.domain_service.create_mock_alerts());
        }

        // Try cache first
        if let Some(cached) = self.get_from_cache().await {
            return Ok(cached);
        }

        // Fetch fresh data
        self.get_active_alerts().await
    }

    async fn refresh_alerts(&self) -> Result<AlertsResponse> {
        self.get_active_alerts().await
    }

    fn is_local_mode(&self) -> bool {
        self.check_local_mode()
    }
}

impl Default for AlertRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

/// Background task to refresh Alertmanager cache
pub async fn start_background_refresh(repository: AlertRepositoryImpl) {
    info!("🚀 Starting Alertmanager background refresh task");

    let mut interval = tokio::time::interval(Duration::from_secs(CACHE_TTL_SECONDS));

    loop {
        interval.tick().await;
        debug!("🔄 Refreshing Alertmanager cache...");

        match repository.get_active_alerts().await {
            Ok(_) => {
                debug!("✅ Updated Alertmanager cache via background task");
            }
            Err(e) => {
                error!("❌ Failed to refresh alerts: {}", e);
            }
        }
    }
}
