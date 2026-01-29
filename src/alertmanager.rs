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
#[derive(Debug, Serialize, Deserialize)]
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
    std::env::var("ALERTMANAGER_URL")
        .unwrap_or_else(|_| {
            tracing::warn!("ALERTMANAGER_URL not set, using default local K8s service URL");
            "http://kube-prometheus-stack-alertmanager.kube-prometheus-stack.svc:9093".to_string()
        })
}

/// Get all active alerts from Alertmanager
pub async fn get_active_alerts() -> Result<AlertsResponse, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v2/alerts", get_alertmanager_url());
    
    let response = client
        .get(&url)
        .query(&[("active", "true"), ("silenced", "false"), ("inhibited", "false")])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Alertmanager request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Alertmanager returned status: {}", response.status()));
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
        let severity = am_alert.labels.get("severity")
            .cloned()
            .unwrap_or_else(|| "info".to_string());
        
        let alert = Alert {
            name: am_alert.labels.get("alertname")
                .cloned()
                .unwrap_or_else(|| "Unknown".to_string()),
            severity: severity.clone(),
            state: am_alert.status.state.clone(),
            summary: am_alert.annotations.get("summary")
                .cloned()
                .unwrap_or_else(|| "No summary".to_string()),
            description: am_alert.annotations.get("description").cloned(),
            namespace: am_alert.labels.get("namespace").cloned(),
            pod: am_alert.labels.get("pod").cloned(),
            started_at: am_alert.starts_at.parse::<DateTime<Utc>>()
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

