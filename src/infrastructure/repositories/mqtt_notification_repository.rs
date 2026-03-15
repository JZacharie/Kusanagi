//! MQTT Notification Repository Implementation
//!
//! Infrastructure adapter for sending notifications via MQTT.

use crate::domain::ports::NotificationRepository;
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tracing::{info, error};

pub struct MqttNotificationRepository {
    client: AsyncClient,
}

impl MqttNotificationRepository {
    pub async fn new(host: String, port: u16, username: Option<String>, password: Option<String>) -> Self {
        let mut mqttoptions = MqttOptions::new("kusanagi-notifier", host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));
        
        if let (Some(u), Some(p)) = (username, password) {
            mqttoptions.set_credentials(u, p);
        }

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
        
        // Start event loop in background
        tokio::spawn(async move {
            loop {
                if let Err(e) = eventloop.poll().await {
                    error!("MQTT Notification event loop error: {:?}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        });

        Self { client }
    }
}

#[async_trait]
impl NotificationRepository for MqttNotificationRepository {
    async fn send_message(&self, topic: &str, message: &str) -> Result<()> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, message.as_bytes().to_vec())
            .await
            .map_err(|e| {
                error!("Failed to publish MQTT notification: {}", e);
                KusanagiError::external_service(format!("MQTT publish error: {}", e))
            })?;

        info!("Successfully sent MQTT notification to topic: {}", topic);
        Ok(())
    }
}
