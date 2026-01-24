use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sensor {
    pub entity_id: String,
    pub state: String,
    pub attributes: serde_json::Value,
    pub last_changed: String,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sensor {
    pub entity_id: String,
    pub state: String,
    pub attributes: serde_json::Value,
    pub last_changed: String,
    pub last_updated: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Automation {
    pub entity_id: String,
    pub state: String,
    pub attributes: AutomationAttributes,
    pub last_changed: String,
    pub last_triggered: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AutomationAttributes {
    #[serde(default)]
    pub friendly_name: String,
    #[serde(default)]
    pub last_triggered: String,
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub current: u32,
    #[serde(default)]
    pub max: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Device {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub area_id: String,
    #[serde(default)]
    pub manufacturer: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub sw_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HAState {
    pub entity_id: String,
    pub state: String,
    pub attributes: serde_json::Value,
    pub last_changed: String,
    pub last_updated: String,
}

pub struct HomeAssistantClient {
    base_url: String,
    client: reqwest::Client,
    token: String,
}

impl HomeAssistantClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut base_url = env::var("HOME_ASSISTANT_URL")
            .unwrap_or_else(|_| "http://homeassistant.local:8123".to_string());
        if base_url.ends_with('/') {
            base_url.pop();
        }
        let token = env::var("HOME_ASSISTANT_TOKEN")
            .unwrap_or_else(|_| "".to_string());

        if token.is_empty() {
            let user = env::var("HOME_ASSISTANT_USER").is_ok();
            let pass = env::var("HOME_ASSISTANT_PASSWORD").is_ok();
            if user && pass {
                info!("HOME_ASSISTANT_USER and PASSWORD found, but TOKEN is missing. HA API typically requires a Long-Lived Access Token.");
            } else {
                warn!("HOME_ASSISTANT_TOKEN not set, API calls will fail");
            }
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        Ok(Self {
            base_url,
            client,
            token,
        })
    }

    async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let url = format!("{}/api{}", self.base_url, path);
        
        info!("Home Assistant API request: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Home Assistant API error: {} - {}", status, body);
            return Err(format!("Home Assistant API error: {}", status).into());
        }

        let data: T = response.json().await?;
        Ok(data)
    }

    pub async fn get_states(&self) -> Result<Vec<HAState>, Box<dyn std::error::Error>> {
        self.get("/states").await
    }

    pub async fn get_sensors(&self) -> Result<Vec<Sensor>, Box<dyn std::error::Error>> {
        let states: Vec<HAState> = self.get_states().await?;
        
        let sensors: Vec<Sensor> = states
            .into_iter()
            .filter(|s| {
                s.entity_id.starts_with("sensor.") || 
                s.entity_id.starts_with("binary_sensor.") ||
                s.entity_id.starts_with("update.") ||
                s.entity_id.starts_with("number.") ||
                s.entity_id.starts_with("input_number.")
            })
            .map(|s| {
                Sensor {
                    entity_id: s.entity_id,
                    state: s.state,
                    attributes: s.attributes,
                    last_changed: s.last_changed,
                    last_updated: s.last_updated,
                }
            })
            .collect();

        Ok(sensors)
    }

    pub async fn get_automations(&self) -> Result<Vec<Automation>, Box<dyn std::error::Error>> {
        let states: Vec<HAState> = self.get_states().await?;
        
        let automations: Vec<Automation> = states
            .into_iter()
            .filter(|s| s.entity_id.starts_with("automation."))
            .map(|s| {
                let attributes = serde_json::from_value::<AutomationAttributes>(s.attributes.clone())
                    .unwrap_or_else(|_| AutomationAttributes {
                        friendly_name: s.entity_id.clone(),
                        last_triggered: String::new(),
                        mode: String::new(),
                        current: 0,
                        max: 0,
                    });

                let last_triggered = if !attributes.last_triggered.is_empty() {
                    Some(attributes.last_triggered.clone())
                } else {
                    None
                };

                Automation {
                    entity_id: s.entity_id,
                    state: s.state,
                    attributes,
                    last_changed: s.last_changed,
                    last_triggered,
                }
            })
            .collect();

        Ok(automations)
    }

    pub async fn get_devices(&self) -> Result<Vec<Device>, Box<dyn std::error::Error>> {
        // Note: Device registry requires WebSocket API or config/device_registry endpoint
        // For now, we'll return device trackers from states
        let states: Vec<HAState> = self.get_states().await?;
        
        let devices: Vec<Device> = states
            .into_iter()
            .filter(|s| s.entity_id.starts_with("device_tracker.") || s.entity_id.starts_with("person."))
            .enumerate()
            .map(|(idx, s)| {
                let name = s.attributes
                    .get("friendly_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&s.entity_id)
                    .to_string();

                Device {
                    id: format!("device_{}", idx),
                    name,
                    area_id: String::new(),
                    manufacturer: String::new(),
                    model: String::new(),
                    sw_version: String::new(),
                }
            })
            .collect();

        Ok(devices)
    }

    pub async fn get_config(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        self.get("/config").await
    }
}

// API Handlers
pub async fn get_sensors_handler() -> Result<HttpResponse> {
    match HomeAssistantClient::new() {
        Ok(client) => match client.get_sensors().await {
            Ok(sensors) => {
                info!("Retrieved {} sensors from Home Assistant", sensors.len());
                Ok(HttpResponse::Ok().json(sensors))
            }
            Err(e) => {
                error!("Failed to get sensors: {}", e);
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": format!("Failed to fetch sensors: {}", e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Home Assistant client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Home Assistant not configured: {}", e)
            })))
        }
    }
}

pub async fn get_automations_handler() -> Result<HttpResponse> {
    match HomeAssistantClient::new() {
        Ok(client) => match client.get_automations().await {
            Ok(automations) => {
                info!("Retrieved {} automations from Home Assistant", automations.len());
                Ok(HttpResponse::Ok().json(automations))
            }
            Err(e) => {
                error!("Failed to get automations: {}", e);
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": format!("Failed to fetch automations: {}", e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Home Assistant client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Home Assistant not configured: {}", e)
            })))
        }
    }
}

pub async fn get_devices_handler() -> Result<HttpResponse> {
    match HomeAssistantClient::new() {
        Ok(client) => match client.get_devices().await {
            Ok(devices) => {
                info!("Retrieved {} devices from Home Assistant", devices.len());
                Ok(HttpResponse::Ok().json(devices))
            }
            Err(e) => {
                error!("Failed to get devices: {}", e);
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": format!("Failed to fetch devices: {}", e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Home Assistant client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Home Assistant not configured: {}", e)
            })))
        }
    }
}

pub async fn get_config_handler() -> Result<HttpResponse> {
    match HomeAssistantClient::new() {
        Ok(client) => match client.get_config().await {
            Ok(config) => {
                info!("Retrieved config from Home Assistant");
                Ok(HttpResponse::Ok().json(config))
            }
            Err(e) => {
                error!("Failed to get config: {}", e);
                Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                    "error": format!("Failed to fetch config: {}", e)
                })))
            }
        },
        Err(e) => {
            error!("Failed to create Home Assistant client: {}", e);
            Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
                "error": format!("Home Assistant not configured: {}", e)
            })))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/ha")
            .route("/sensors", web::get().to(get_sensors_handler))
            .route("/automations", web::get().to(get_automations_handler))
            .route("/devices", web::get().to(get_devices_handler))
            .route("/config", web::get().to(get_config_handler)),
    );
}
