use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TailscaleDevice {
    pub addresses: Vec<String>,
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub hostname: String,
    #[serde(rename = "nodeKey", default)]
    pub nodekey: String,
    pub os: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub last_seen: String,
    #[serde(rename = "connectedToControl", default)]
    pub online: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip, default)]
    pub is_exit_node: bool,
    pub user: Option<String>,
    pub tailscale_ips: Option<Vec<String>>,
    pub machine_key: Option<String>,
    #[serde(rename = "expires", default)]
    pub expiry: String,
    pub blocked: Option<bool>,
    pub client_version: Option<String>,
    pub created: Option<String>,
    pub last_modified: Option<String>,
    pub engine: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TailscaleDevicesResponse {
    pub devices: Vec<TailscaleDevice>,
}

pub async fn fetch_tailscale_devices(
    client: &reqwest::Client,
) -> Result<TailscaleDevicesResponse, String> {
    let api_key = std::env::var("TAILSCALE_API_KEY").unwrap_or_else(|_| {
        "tskey-api-kWTqDi8bTp11CNTRL-9VWrtw1aZSEbNRphMu2zREw6dBX3UNyQ1".to_string()
    });
    let tailnet = std::env::var("TAILSCALE_TAILNET").unwrap_or_else(|_| "zacharie.org".to_string());

    let url = format!(
        "https://api.tailscale.com/api/v2/tailnet/{}/devices",
        tailnet
    );

    let response = client
        .get(&url)
        .basic_auth(api_key, Some(""))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Tailscale API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Tailscale API returned {}: {}", status, body));
    }

    let data: TailscaleDevicesResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Tailscale response: {}", e))?;

    Ok(data)
}

pub async fn get_tailscale_devices_json(
    client: &reqwest::Client,
    cache: &crate::AdvancedCache<String>,
) -> serde_json::Value {
    const CACHE_KEY: &str = "tailscale_devices";

    if let Some(cached) = cache.get(CACHE_KEY).await {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&cached) {
            return value;
        }
    }

    match fetch_tailscale_devices(client).await {
        Ok(response) => {
            let enriched: Vec<serde_json::Value> = response
                .devices
                .into_iter()
                .map(|d| {
                    let machine_name = if d.name.is_empty() {
                        &d.hostname
                    } else {
                        &d.name
                    };

                    let is_exit =
                        machine_name.contains("exit-node") || d.hostname.contains("exit-node");

                    json!({
                        "id": d.id,
                        "name": machine_name,
                        "hostname": d.hostname,
                        "addresses": d.addresses,
                        "tailscale_ips": d.tailscale_ips.unwrap_or_default(),
                        "os": d.os.unwrap_or_default(),
                        "version": d.version.unwrap_or_default(),
                        "last_seen": d.last_seen,
                        "online": d.online,
                        "tags": d.tags,
                        "is_exit_node": is_exit,
                        "user": d.user.unwrap_or_default(),
                        "expiry": d.expiry,
                        "blocked": d.blocked.unwrap_or(false),
                        "engine": d.engine.unwrap_or_default(),
                    })
                })
                .collect();

            let result = json!({
                "devices": enriched,
                "total": enriched.len(),
                "online": enriched.iter().filter(|d| d["online"].as_bool().unwrap_or(false)).count(),
            });

            if let Ok(json_str) = serde_json::to_string(&result) {
                cache
                    .set(
                        CACHE_KEY.to_string(),
                        json_str,
                        Some(std::time::Duration::from_secs(60)),
                    )
                    .await;
            }

            result
        }
        Err(e) => json!({
            "error": e,
            "devices": [],
            "total": 0,
            "online": 0
        }),
    }
}
