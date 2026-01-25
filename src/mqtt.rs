use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info};
use actix_web::{web, HttpResponse, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttDevice {
    pub id: String,
    pub name: String,
    pub last_seen: String,
    pub last_topic: String,
    pub message_count: usize,
}

pub struct MqttState {
    pub recent_messages: VecDeque<MqttMessage>,
    pub devices: HashMap<String, MqttDevice>,
    pub tx: broadcast::Sender<MqttMessage>,
    pub client: Option<AsyncClient>,
}

lazy_static::lazy_static! {
    pub static ref MQTT_STATE: Arc<Mutex<MqttState>> = {
        let (tx, _) = broadcast::channel(100);
        Arc::new(Mutex::new(MqttState {
            recent_messages: VecDeque::with_capacity(100),
            devices: HashMap::new(),
            tx,
            client: None,
        }))
    };
}

pub async fn init_mqtt() {
    let host = env::var("MQTT_ENDPOINT").unwrap_or_else(|_| "localhost".to_string());
    let user = env::var("MQTT_USER").ok();
    let pass = env::var("MQTT_PASSWORD").ok();

    info!("Initializing MQTT client for {}", host);

    let client_id = format!("kusanagi-backend-{}", chrono::Utc::now().timestamp() % 1000);
    let mut mqttoptions = MqttOptions::new(client_id, host, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    if let (Some(u), Some(p)) = (user, pass) {
        mqttoptions.set_credentials(u, p);
    }

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    
    // Store client in state
    {
        let mut state = MQTT_STATE.lock().unwrap();
        state.client = Some(client.clone());
    }

    // Subscribe to all topics
    tokio::spawn(async move {
        if let Err(e) = client.subscribe("#", QoS::AtMostOnce).await {
            error!("Failed to subscribe to MQTT topics: {}", e);
            return;
        }

        loop {
            match eventloop.poll().await {
                Ok(notification) => {
                    if let Event::Incoming(Packet::Publish(publish)) = notification {
                        let topic = publish.topic;
                        let payload = String::from_utf8_lossy(&publish.payload).to_string();
                        let timestamp = chrono::Utc::now().to_rfc3339();

                        let msg = MqttMessage {
                            topic: topic.clone(),
                            payload,
                            timestamp,
                        };

                        handle_incoming_message(msg);
                    }
                }
                Err(e) => {
                    error!("MQTT EventLoop error: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });
}

fn handle_incoming_message(msg: MqttMessage) {
    let mut state = MQTT_STATE.lock().unwrap();
    
    // Broadcast to WebSocket listeners
    let _ = state.tx.send(msg.clone());

    // Update recent messages
    state.recent_messages.push_back(msg.clone());
    if state.recent_messages.len() > 100 {
        state.recent_messages.pop_front();
    }

    // Detect/Update device (naive detection based on first part of topic)
    let device_id = msg.topic.split('/').next().unwrap_or("unknown").to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let device = state.devices.entry(device_id.clone()).or_insert(MqttDevice {
        id: device_id,
        name: msg.topic.split('/').next().unwrap_or("Unknown Device").to_string(),
        last_seen: now.clone(),
        last_topic: msg.topic.clone(),
        message_count: 0,
    });

    device.last_seen = now;
    device.last_topic = msg.topic.clone();
    device.message_count += 1;

    // Bridge to Slack
    debug!("Bridging MQTT message from topic `{}` to Slack", msg.topic);
    let slack_msg = format!("*MQTT Message*\n*Topic*: `{}`\n*Payload*: `{}`", msg.topic, msg.payload);
    tokio::spawn(async move {
        if let Ok(slack) = crate::slack::SlackClient::new() {
            let _ = slack.send_message(&slack_msg).await;
        }
    });
}

pub async fn publish_message(topic: &str, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = {
        let state = MQTT_STATE.lock().unwrap();
        state.client.clone()
    };

    if let Some(client) = client {
        client.publish(topic, QoS::AtMostOnce, false, payload).await?;
        Ok(())
    } else {
        Err("MQTT client not initialized".into())
    }
}

// API Handlers
pub async fn get_mqtt_messages() -> Result<HttpResponse> {
    let state = MQTT_STATE.lock().unwrap();
    let messages: Vec<MqttMessage> = state.recent_messages.iter().cloned().collect();
    Ok(HttpResponse::Ok().json(messages))
}

pub async fn get_mqtt_devices() -> Result<HttpResponse> {
    let state = MQTT_STATE.lock().unwrap();
    let devices: Vec<MqttDevice> = state.devices.values().cloned().collect();
    Ok(HttpResponse::Ok().json(devices))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/mqtt")
            .route("/messages", web::get().to(get_mqtt_messages))
            .route("/devices", web::get().to(get_mqtt_devices)),
    );
}
