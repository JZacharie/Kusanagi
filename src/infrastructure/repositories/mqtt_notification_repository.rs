//! MQTT Notification Repository Implementation
//!
//! Infrastructure adapter for sending notifications via MQTT.

use crate::domain::ports::NotificationRepository;
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tracing::{error, info, warn};

pub struct MqttNotificationRepository {
    client: AsyncClient,
    namespace: String,
}

impl MqttNotificationRepository {
    pub async fn new(
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
        namespace: String,
    ) -> Self {
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "kusanagi".to_string());

        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        hostname.hash(&mut hasher);
        let host_hash = format!("{:04x}", hasher.finish() & 0xFFFF);
        let client_id = format!("kn-{}-{}", host_hash, std::process::id() % 1000);

        info!(
            "🔌 MQTT Notification: Connecting to {}:{} with client_id: {}",
            host, port, client_id
        );

        let mut mqttoptions = MqttOptions::new(client_id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(60));
        mqttoptions.set_clean_session(true);

        if let (Some(u), Some(p)) = (username, password) {
            mqttoptions.set_credentials(u, p);
        }

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // Start event loop in background
        tokio::spawn(async move {
            loop {
                match eventloop.poll().await {
                    Ok(notification) => match notification {
                        rumqttc::Event::Incoming(rumqttc::Packet::ConnAck(connack)) => {
                            info!("✅ MQTT Notification: Connected! ConnAck: {:?}", connack);
                        }
                        rumqttc::Event::Incoming(rumqttc::Packet::Disconnect) => {
                            warn!("⚠️ MQTT Notification: Received Disconnect from broker");
                        }
                        _ => {}
                    },
                    Err(e) => {
                        error!("❌ MQTT Notification event loop error: {:?}", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Self { client, namespace }
    }
}

#[async_trait]
impl NotificationRepository for MqttNotificationRepository {
    async fn send_message(&self, topic: &str, message: &str) -> Result<()> {
        let prefixed_topic = if topic.starts_with(&self.namespace) {
            topic.to_string()
        } else {
            format!("{}/{}", self.namespace, topic)
        };

        self.client
            .publish(
                prefixed_topic.clone(),
                QoS::AtLeastOnce,
                false,
                message.as_bytes().to_vec(),
            )
            .await
            .map_err(|e| {
                error!("Failed to publish MQTT notification: {}", e);
                KusanagiError::external_service(format!("MQTT publish error: {}", e))
            })?;

        info!(
            "Successfully sent MQTT notification to topic: {}",
            prefixed_topic
        );
        Ok(())
    }
}
