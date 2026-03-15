use tracing::{error, info, warn};

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;
// Removed

#[derive(Clone, Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: String,
    pub timestamp: u128,
}

#[derive(Clone, Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct MqttDevice {
    pub id: String,
    pub name: String,
    pub last_seen: u128,
    pub last_topic: String,
    pub message_count: u64,
}

struct InnerState {
    devices: Vec<MqttDevice>,
    messages: VecDeque<MqttMessage>,
}

#[derive(Clone)]
pub struct MqttState {
    inner: Arc<Mutex<InnerState>>,
    pub process_audio_use_case: Option<Arc<crate::application::use_cases::ProcessAudioUseCase>>,
    pub broadcast_sender: Option<tokio::sync::broadcast::Sender<crate::interfaces::http::handlers::core::websocket::NotificationMessage>>,
    pub namespace: String,
}

impl Default for MqttState {
    fn default() -> Self {
        Self::new()
    }
}

impl MqttState {
    pub fn new() -> Self {
        MqttState {
            inner: Arc::new(Mutex::new(InnerState {
                devices: Vec::new(),
                messages: VecDeque::with_capacity(500),
            })),
            process_audio_use_case: None,
            broadcast_sender: None,
            namespace: "unknown".to_string(),
        }
    }

    pub fn with_process_audio(mut self, use_case: Arc<crate::application::use_cases::ProcessAudioUseCase>) -> Self {
        self.process_audio_use_case = Some(use_case);
        self
    }

    pub fn with_broadcast(mut self, sender: tokio::sync::broadcast::Sender<crate::interfaces::http::handlers::core::websocket::NotificationMessage>) -> Self {
        self.broadcast_sender = Some(sender);
        self
    }

    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.namespace = namespace;
        self
    }

    pub fn handle_message(&self, topic: String, payload: String) {
        use crate::utils::MutexExt;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let mut inner = self.inner.lock_safe();

        // Add to buffer
        if inner.messages.len() >= 500 {
            inner.messages.pop_back();
        }
        inner.messages.push_front(MqttMessage {
            topic: topic.clone(),
            payload: payload.clone(),
            timestamp: now,
        });

        // Broadcast to WebSockets
        if let Some(sender) = &self.broadcast_sender {
            let msg = crate::interfaces::http::handlers::core::websocket::NotificationMessage::Mqtt {
                topic: topic.clone(),
                payload: payload.clone(),
                timestamp: now,
            };
            let _ = sender.send(msg);
        }

        // Update Device
        // Assumption: topic format is "deviceId/..." or just match first part
        let device_id = topic.split('/').next().unwrap_or("unknown").to_string();

        if let Some(device) = inner.devices.iter_mut().find(|d| d.id == device_id) {
            device.last_seen = now;
            device.last_topic = topic;
            device.message_count += 1;
        } else {
            inner.devices.push(MqttDevice {
                id: device_id.clone(),
                name: device_id,
                last_seen: now,
                last_topic: topic.clone(),
                message_count: 1,
            });

            // Limit devices to 100 to prevent memory leak
            if inner.devices.len() > 100 {
                // Remove oldest device (least recently seen)
                if let Some(oldest_idx) = inner
                    .devices
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, d)| d.last_seen)
                    .map(|(idx, _)| idx)
                {
                    inner.devices.remove(oldest_idx);
                }
            }
        }
    }

    pub async fn handle_publish(&self, topic: String, payload_bytes: Vec<u8>) {
        let payload_str = String::from_utf8_lossy(&payload_bytes).to_string();
        
        // Standard message handling
        self.handle_message(topic.clone(), payload_str);

        // Special handling for "kitt" topic (Audio ASR)
        if topic == "kitt" {
            if let Some(use_case) = &self.process_audio_use_case {
                let filename = format!("mqtt_audio_{}", 
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
                );
                
                let state_clone = self.clone();
                let use_case = use_case.clone();
                tokio::spawn(async move {
                    match use_case.execute(payload_bytes, &filename).await {
                        Ok(text) => {
                            let topic = format!("{}/kitt/transcription", state_clone.namespace);
                            state_clone.handle_message(topic, text);
                        }
                        Err(e) => {
                            error!("Failed to process MQTT audio: {}", e);
                        }
                    }
                });
            } else {
                warn!("Received audio on 'kitt' but ProcessAudioUseCase is not configured");
            }
        }
    }

    pub fn get_devices(&self) -> Value {
        use crate::utils::MutexExt;
        let inner = self.inner.lock_safe();
        json!(inner.devices)
    }

    pub fn get_messages(&self) -> Value {
        use crate::utils::MutexExt;
        let inner = self.inner.lock_safe();
        json!(inner.messages)
    }
}

pub fn start_mqtt_client(
    state: MqttState,
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    namespace: String,
) {
    task::spawn(async move {
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "kusanagi".to_string());
        let client_id = format!("{}-{}-{}", namespace, hostname, std::process::id());
        let mut mqttoptions = MqttOptions::new(client_id, host.clone(), port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));
        mqttoptions.set_clean_session(true);

        if let (Some(u), Some(p)) = (username, password) {
            mqttoptions.set_credentials(u, p);
        }

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // Subscribe to everything
        match client.subscribe("#", QoS::AtMostOnce).await {
            Ok(_) => info!("📡 MQTT: Subscribed to '#' on {}", host),
            Err(e) => {
                error!("❌ MQTT: Error subscribing to '#': {:?}", e);
                return;
            }
        }

        info!("📡 MQTT: Connected to {}", host);

        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                    if let Event::Incoming(Packet::Publish(publish)) = notification {
                        state.handle_publish(publish.topic, publish.payload.to_vec()).await;
                    }
                }
                Err(e) => {
                    warn!("⚠️ MQTT connection error: {:?} — retrying in 5s", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });
}
