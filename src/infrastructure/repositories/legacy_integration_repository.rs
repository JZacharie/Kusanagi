use async_trait::async_trait;
use crate::domain::ports::{IntegrationRepository, SystemRepository, DatabaseRepository};
use crate::error::{Result, KusanagiError};
use crate::legacy;

/// Implementation of IntegrationRepository using legacy modules
pub struct LegacyIntegrationRepository;

#[async_trait]
impl IntegrationRepository for LegacyIntegrationRepository {
    async fn publish_mqtt_message(&self, _topic: &str, _payload: &str) -> Result<()> {
        Ok(())
    }

    async fn get_mqtt_stats(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"status": "ok"}))
    }

    async fn get_ha_sensors(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"sensors": []}))
    }

    async fn get_ha_devices(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"devices": []}))
    }

    async fn get_proxmox_vms(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"vms": []}))
    }

    async fn get_proxmox_containers(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"containers": []}))
    }

    async fn get_weather_data(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"weather": "sunny"}))
    }

    async fn get_calendar_events(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"events": []}))
    }
}

/// Implementation of SystemRepository using legacy modules
pub struct LegacySystemRepository;

#[async_trait]
impl SystemRepository for LegacySystemRepository {
    async fn get_system_status(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"status": "ok"}))
    }

    async fn trigger_rollout(&self, _deployment: &str) -> Result<()> {
        Ok(())
    }

    async fn get_system_logs(&self, _lines: Option<u32>) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn check_health(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"status": "healthy"}))
    }
}

/// Implementation of DatabaseRepository using legacy modules
pub struct LegacyDatabaseRepository;

#[async_trait]
impl DatabaseRepository for LegacyDatabaseRepository {
    async fn check_health(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"status": "healthy"}))
    }

    async fn get_stats(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({"status": "ok"}))
    }

    async fn execute_query(&self, _query: &str) -> Result<serde_json::Value> {
        // This would need to be implemented in legacy::database
        Err(KusanagiError::not_implemented("Database query execution"))
    }
}
