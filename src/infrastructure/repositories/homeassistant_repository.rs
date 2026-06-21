//! HomeAssistant Repository Implementation
//!
//! Infrastructure adapter implementing the HomeAssistantRepository port.
//! Handles Home Assistant REST API calls.

use crate::domain::entities::{
    HomeAssistantDevice, HomeAssistantDevicesResponse, HomeAssistantSensor,
    HomeAssistantSensorsResponse, HomeAssistantState,
};
use crate::domain::ports::HomeAssistantRepository;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::env;
use std::time::Duration;
use tracing::{error, info, warn};

const DEFAULT_HA_URL: &str = "http://homeassistant.local:8123";
const DEFAULT_TIMEOUT_SECONDS: u64 = 10;

/// HomeAssistant repository implementation
pub struct HomeAssistantRepositoryImpl {
    client: reqwest::Client,
    base_url: String,
    token: String,
}

impl HomeAssistantRepositoryImpl {
    /// Create a new repository instance
    pub fn new() -> Result<Self> {
        let mut base_url = env::var("HOME_ASSISTANT_URL").unwrap_or_else(|_| {
            warn!("HOME_ASSISTANT_URL not set, using default");
            DEFAULT_HA_URL.to_string()
        });

        if base_url.ends_with('/') {
            base_url.pop();
        }

        let token = env::var("HOME_ASSISTANT_TOKEN").unwrap_or_default();

        if token.is_empty() {
            let user = env::var("HOME_ASSISTANT_USER").is_ok();
            let pass = env::var("HOME_ASSISTANT_PASSWORD").is_ok();
            if user && pass {
                warn!("HOME_ASSISTANT_USER and PASSWORD found, but TOKEN is missing. HA API requires a Long-Lived Access Token.");
            } else {
                warn!("HOME_ASSISTANT_TOKEN not set, API calls will fail");
            }
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECONDS))
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| {
                AppError::external_service(format!("Failed to build HTTP client: {}", e))
            })?;

        Ok(Self {
            client,
            base_url,
            token,
        })
    }

    /// Check if the repository is configured (has a token)
    pub fn is_configured(&self) -> bool {
        !self.token.is_empty()
    }

    /// Generic GET request to Home Assistant API
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let url = format!("{}/api{}", self.base_url, path);

        info!("Home Assistant API request: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| {
                AppError::external_service(format!("Home Assistant request failed: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            if status == 401 {
                error!(
                    "Home Assistant API error: 401 Unauthorized (Token missing: {})",
                    self.token.is_empty()
                );
            } else {
                error!("Home Assistant API error: {} - {}", status, body);
            }
            return Err(AppError::external_service(format!(
                "Home Assistant API error: {}",
                status
            )));
        }

        let data: T = response
            .json()
            .await
            .map_err(|e| AppError::serialization(format!("Failed to parse HA response: {}", e)))?;

        Ok(data)
    }

    /// Get all states from Home Assistant
    async fn get_states(&self) -> Result<Vec<HomeAssistantState>> {
        self.get("/states").await
    }

    /// Get all sensors (legacy method, used internally)
    async fn get_sensors_internal(&self) -> Result<Vec<HomeAssistantSensor>> {
        let states = self.get_states().await?;

        let sensors: Vec<HomeAssistantSensor> = states
            .into_iter()
            .filter(|s| {
                (s.entity_id.starts_with("sensor.")
                    || s.entity_id.starts_with("binary_sensor.")
                    || s.entity_id.starts_with("update.")
                    || s.entity_id.starts_with("number.")
                    || s.entity_id.starts_with("input_number.")
                    || s.entity_id.starts_with("switch.")
                    || s.entity_id.starts_with("light.")
                    || s.entity_id.starts_with("fan.")
                    || s.entity_id.starts_with("lock."))
                    && s.state != "unavailable"
                    && s.state != "unknown"
            })
            .map(|s| HomeAssistantSensor {
                entity_id: s.entity_id,
                state: s.state,
                attributes: s.attributes,
                last_changed: s.last_changed,
                last_updated: s.last_updated,
            })
            .collect();

        Ok(sensors)
    }

    /// Get all devices (legacy method, used internally)
    async fn get_devices_internal(&self) -> Result<Vec<HomeAssistantDevice>> {
        let states = self.get_states().await?;

        let devices: Vec<HomeAssistantDevice> = states
            .into_iter()
            .filter(|s| {
                s.entity_id.starts_with("device_tracker.") || s.entity_id.starts_with("person.")
            })
            .enumerate()
            .map(|(idx, s)| {
                let name = s
                    .attributes
                    .get("friendly_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&s.entity_id)
                    .to_string();

                HomeAssistantDevice {
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
}

#[async_trait]
impl HomeAssistantRepository for HomeAssistantRepositoryImpl {
    async fn get_sensors(&self) -> Result<HomeAssistantSensorsResponse> {
        if !self.is_configured() {
            warn!("Home Assistant not configured: HOME_ASSISTANT_TOKEN not set, returning empty sensors");
            return Ok(HomeAssistantSensorsResponse {
                sensors: vec![],
                count: 0,
            });
        }

        match self.get_sensors_internal().await {
            Ok(sensors) => {
                let count = sensors.len();
                info!("Retrieved {} sensors from Home Assistant", count);
                Ok(HomeAssistantSensorsResponse { sensors, count })
            }
            Err(e) => {
                warn!("Failed to fetch sensors from Home Assistant: {}, returning empty", e);
                Ok(HomeAssistantSensorsResponse {
                    sensors: vec![],
                    count: 0,
                })
            }
        }
    }

    async fn get_devices(&self) -> Result<HomeAssistantDevicesResponse> {
        if !self.is_configured() {
            warn!("Home Assistant not configured: HOME_ASSISTANT_TOKEN not set, returning empty devices");
            return Ok(HomeAssistantDevicesResponse {
                devices: vec![],
                count: 0,
            });
        }

        match self.get_devices_internal().await {
            Ok(devices) => {
                let count = devices.len();
                info!("Retrieved {} devices from Home Assistant", count);
                Ok(HomeAssistantDevicesResponse { devices, count })
            }
            Err(e) => {
                warn!("Failed to fetch devices from Home Assistant: {}, returning empty", e);
                Ok(HomeAssistantDevicesResponse {
                    devices: vec![],
                    count: 0,
                })
            }
        }
    }
}

impl Default for HomeAssistantRepositoryImpl {
    fn default() -> Self {
        // Note: This will panic if environment variables are not set
        // Use HomeAssistantRepositoryImpl::new() for proper error handling
        Self::new().expect("Failed to create HomeAssistantRepositoryImpl")
    }
}

/// Factory function for creating the repository
pub fn create_homeassistant_repository() -> Result<HomeAssistantRepositoryImpl> {
    HomeAssistantRepositoryImpl::new()
}
