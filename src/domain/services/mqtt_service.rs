use tracing::{debug, error, info};

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task;
// Removed

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: String,
    pub timestamp: u128,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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
        }
    }

    pub fn handle_message(&self, topic: String, payload: String) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        let mut inner = self.inner.lock().unwrap();

        // Add to buffer
        if inner.messages.len() >= 500 {
            inner.messages.pop_back();
        }
        inner.messages.push_front(MqttMessage {
            topic: topic.clone(),
            payload: payload.clone(),
            timestamp: now,
        });

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

    pub fn get_devices(&self) -> Value {
        let inner = self.inner.lock().unwrap();
        json!(inner.devices)
    }

    pub fn get_messages(&self) -> Value {
        let inner = self.inner.lock().unwrap();
        json!(inner.messages)
    }
}

pub fn start_mqtt_client(
    state: MqttState,
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
) {
    task::spawn(async move {
        // Generate random client id
        let client_id = format!("kusanagi-{}", std::process::id());
        let mut mqttoptions = MqttOptions::new(client_id, host.clone(), port);
        mqttoptions.set_keep_alive(Duration::from_secs(30)); // Augmenté de 5s à 30s

        if let (Some(u), Some(p)) = (username, password) {
            mqttoptions.set_credentials(u, p);
        }

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // Subscribe to everything
        if let Err(e) = client.subscribe("#", QoS::AtMostOnce).await {
            error!("❌ MQTT: Error subscribing: {:?}", e);
            return;
        }

        info!("📡 MQTT: Connected to {}", host);

        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                    if let Event::Incoming(Packet::Publish(publish)) = notification {
                        let payload = String::from_utf8_lossy(&publish.payload).to_string();
                        // println!("Received: {} = {}", publish.topic, payload);
                        state.handle_message(publish.topic, payload);
                    }
                }
                Err(e) => {
                    debug!("MQTT connection error: {:?}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });
}
