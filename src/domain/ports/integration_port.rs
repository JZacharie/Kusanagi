use async_trait::async_trait;
use crate::error::Result;

/// Port for external integrations (MQTT, Home Assistant, Proxmox, etc.)
#[async_trait]
pub trait IntegrationRepository: Send + Sync {
    // MQTT
    async fn publish_mqtt_message(&self, topic: &str, payload: &str) -> Result<()>;
    async fn get_mqtt_stats(&self) -> Result<serde_json::Value>;
    
    // Home Assistant
    async fn get_ha_sensors(&self) -> Result<serde_json::Value>;
    async fn get_ha_devices(&self) -> Result<serde_json::Value>;
    
    // Proxmox
    async fn get_proxmox_vms(&self) -> Result<serde_json::Value>;
    async fn get_proxmox_containers(&self) -> Result<serde_json::Value>;
    
    // Weather
    async fn get_weather_data(&self) -> Result<serde_json::Value>;
    
    // Calendar
    async fn get_calendar_events(&self) -> Result<serde_json::Value>;
}
