use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Single alert from Alertmanager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub name: String,
    pub severity: String,
    pub state: String,
    pub summary: String,
    pub description: Option<String>,
    pub namespace: Option<String>,
    pub pod: Option<String>,
    pub started_at: DateTime<Utc>,
    pub fingerprint: String,
}

/// Grouped alerts response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsResponse {
    pub critical: Vec<Alert>,
    pub warning: Vec<Alert>,
    pub info: Vec<Alert>,
    pub total: i32,
    pub firing: i32,
    pub pending: i32,
}

/// Alertmanager API response structures
#[derive(Debug, Deserialize)]
struct AmAlert {
    labels: std::collections::HashMap<String, String>,
    annotations: std::collections::HashMap<String, String>,
    #[serde(rename = "startsAt")]
    starts_at: String,
    #[serde(rename = "endsAt")]
    _ends_at: String,
    fingerprint: String,
    status: AmAlertStatus,
}

#[derive(Debug, Deserialize)]
struct AmAlertStatus {
    state: String,
}
fn get_alertmanager_url() -> String {
    std::env::var("ALERTMANAGER_URL").unwrap_or_else(|_| {
        tracing::warn!("ALERTMANAGER_URL not set, using default local K8s service URL");
        "http://kube-prometheus-stack-alertmanager.kube-prometheus-stack.svc:9093".to_string()
    })
}

use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

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

/// Get cached active alerts
pub async fn get_cached_active_alerts() -> Result<AlertsResponse, String> {
    // Check if running in local mode
    if std::env::var("KUSANAGI_MODE").unwrap_or_default() == "local" {
        tracing::info!("AlertManager running in local mode, returning mock alerts");
        return Ok(AlertsResponse {
            critical: vec![],
            warning: vec![],
            info: vec![],
            total: 0,
            firing: 0,
            pending: 0,
        });
    }

    // 1. Try to get from cache
    {
        let cache = get_alerts_cache().alerts.read().await;
        if let Some((ref alerts, timestamp)) = *cache {
            if timestamp.elapsed() < Duration::from_secs(120) {
                // Augmenté de 60s à 120s
                return Ok(alerts.clone());
            }
        }
    }

    // 2. If cache miss or expired, fetch live
    let alerts = get_active_alerts().await?;

    // 3. Update cache
    let mut cache = get_alerts_cache().alerts.write().await;
    *cache = Some((alerts.clone(), Instant::now()));

    Ok(alerts)
}

/// Background task to refresh Alertmanager cache
pub async fn start_background_refresh() {
    tracing::info!("🚀 Starting Alertmanager background refresh task");

    let mut interval = tokio::time::interval(Duration::from_secs(120)); // Augmenté de 60s à 120s

    loop {
        interval.tick().await;
        tracing::debug!("🔄 Refreshing Alertmanager cache...");

        match get_active_alerts().await {
            Ok(alerts) => {
                let mut cache = get_alerts_cache().alerts.write().await;
                *cache = Some((alerts, Instant::now()));
                tracing::debug!("✅ Updated Alertmanager cache");
            }
            Err(e) => {
                tracing::error!("❌ Failed to refresh alerts: {}", e);
            }
        }
    }
}

/// Get all active alerts from Alertmanager
pub async fn get_active_alerts() -> Result<AlertsResponse, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v2/alerts", get_alertmanager_url());

    let mut request = client.get(&url);
    request = request.query(&[
        ("active", "true"),
        ("silenced", "false"),
        ("inhibited", "false"),
    ]);

    // Add Basic Auth if credentials are provided
    if let (Ok(username), Ok(password)) = (
        std::env::var("ALERTMANAGER_USERNAME"),
        std::env::var("ALERTMANAGER_PASSWORD"),
    ) {
        tracing::debug!("🔑 Using Alertmanager Basic Auth (User: {})", username);
        request = request.basic_auth(username, Some(password));
    } else {
        tracing::warn!("⚠️ No Alertmanager credentials found in env vars (ALERTMANAGER_USERNAME/PASSWORD)");
    }

    let response = request
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Alertmanager request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Alertmanager returned status: {}",
            response.status()
        ));
    }

    let am_alerts: Vec<AmAlert> = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Alertmanager response: {}", e))?;

    let mut critical = Vec::new();
    let mut warning = Vec::new();
    let mut info = Vec::new();
    let mut firing = 0;
    let mut pending = 0;

    for am_alert in am_alerts {
        let severity = am_alert
            .labels
            .get("severity")
            .cloned()
            .unwrap_or_else(|| "info".to_string());

        let alert = Alert {
            name: am_alert
                .labels
                .get("alertname")
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string()),
            severity: severity.clone(),
            state: am_alert.status.state.clone(),
            summary: am_alert
                .annotations
                .get("summary")
                .cloned()
                .unwrap_or_else(|| "No summary".to_string()),
            description: am_alert.annotations.get("description").cloned(),
            namespace: am_alert.labels.get("namespace").cloned(),
            pod: am_alert.labels.get("pod").cloned(),
            started_at: am_alert
                .starts_at
                .parse::<DateTime<Utc>>()
                .unwrap_or_else(|_| Utc::now()),
            fingerprint: am_alert.fingerprint,
        };

        if am_alert.status.state == "firing" {
            firing += 1;
        } else {
            pending += 1;
        }

        match severity.as_str() {
            "critical" => critical.push(alert),
            "warning" => warning.push(alert),
            _ => info.push(alert),
        }
    }

    // Sort by started_at (most recent first)
    critical.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    warning.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    info.sort_by(|a, b| b.started_at.cmp(&a.started_at));

    let total = critical.len() + warning.len() + info.len();

    Ok(AlertsResponse {
        critical,
        warning,
        info,
        total: total as i32,
        firing,
        pending,
    })
}
