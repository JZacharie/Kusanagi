use async_trait::async_trait;
use crate::domain::ports::{IntegrationRepository, SystemRepository, DatabaseRepository};
use crate::error::{Result, KusanagiError};
use crate::legacy;

/// Implementation of IntegrationRepository using legacy modules
pub struct LegacyIntegrationRepository;

#[async_trait]
impl IntegrationRepository for LegacyIntegrationRepository {
    async fn publish_mqtt_message(&self, topic: &str, payload: &str) -> Result<()> {
        // Delegate to legacy MQTT module
        legacy::mqtt::publish_message(topic, payload).await
            .map_err(|e| KusanagiError::external_api("MQTT", &e.to_string()))
    }

    async fn get_mqtt_stats(&self) -> Result<serde_json::Value> {
        legacy::mqtt::get_mqtt_stats().await
            .map_err(|e| KusanagiError::external_api("MQTT", &e.to_string()))
    }

    async fn get_ha_sensors(&self) -> Result<serde_json::Value> {
        legacy::homeassistant::get_sensors().await
            .map_err(|e| KusanagiError::external_api("HomeAssistant", &e.to_string()))
    }

    async fn get_ha_devices(&self) -> Result<serde_json::Value> {
        legacy::homeassistant::get_devices().await
            .map_err(|e| KusanagiError::external_api("HomeAssistant", &e.to_string()))
    }

    async fn get_proxmox_vms(&self) -> Result<serde_json::Value> {
        legacy::proxmox::get_vms().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }

    async fn get_proxmox_containers(&self) -> Result<serde_json::Value> {
        legacy::proxmox::get_containers().await
            .map_err(|e| KusanagiError::external_api("Proxmox", &e.to_string()))
    }

    async fn get_weather_data(&self) -> Result<serde_json::Value> {
        legacy::weather::get_multi_city_weather().await
            .map_err(|e| KusanagiError::external_api("Weather", &e.to_string()))
    }

    async fn get_calendar_events(&self) -> Result<serde_json::Value> {
        legacy::calendar::get_upcoming_events().await
            .map_err(|e| KusanagiError::external_api("Calendar", &e.to_string()))
    }
}

/// Implementation of SystemRepository using legacy modules
pub struct LegacySystemRepository;

#[async_trait]
impl SystemRepository for LegacySystemRepository {
    async fn get_system_status(&self) -> Result<serde_json::Value> {
        legacy::system::get_status().await
            .map_err(|e| KusanagiError::internal(&e.to_string()))
    }

    async fn trigger_rollout(&self, deployment: &str) -> Result<()> {
        legacy::system::trigger_rollout(deployment).await
            .map_err(|e| KusanagiError::internal(&e.to_string()))
    }

    async fn get_system_logs(&self, lines: Option<u32>) -> Result<Vec<String>> {
        // This would need to be implemented in legacy::system
        // For now, return empty
        Ok(vec![])
    }

    async fn check_health(&self) -> Result<serde_json::Value> {
        legacy::health::check_health().await
            .map_err(|e| KusanagiError::internal(&e.to_string()))
    }
}

/// Implementation of DatabaseRepository using legacy modules
pub struct LegacyDatabaseRepository;

#[async_trait]
impl DatabaseRepository for LegacyDatabaseRepository {
    async fn check_health(&self) -> Result<serde_json::Value> {
        legacy::database::check_health().await
            .map_err(|e| KusanagiError::internal(&e.to_string()))
    }

    async fn get_stats(&self) -> Result<serde_json::Value> {
        // This would need to be implemented in legacy::database
        Ok(serde_json::json!({"status": "not_implemented"}))
    }

    async fn execute_query(&self, _query: &str) -> Result<serde_json::Value> {
        // This would need to be implemented in legacy::database
        Err(KusanagiError::not_implemented("Database query execution"))
    }
}
