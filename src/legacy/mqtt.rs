use rumqttc::{AsyncClient, MqttOptions, QoS, Event, Packet};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::env;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use tokio::time::interval;
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
    pub first_seen: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttTopic {
    pub name: String,
    pub message_count: usize,
    pub last_message: Option<MqttMessage>,
    pub sub_topics: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttStats {
    pub total_messages: usize,
    pub total_devices: usize,
    pub total_topics: usize,
    pub messages_per_minute: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttHealth {
    pub status: String,
    pub connected: bool,
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
    pub last_error: Option<String>,
    pub connection_time: Option<String>,
}

pub struct MqttState {
    pub recent_messages: VecDeque<MqttMessage>,
    pub devices: HashMap<String, MqttDevice>,
    pub topics: HashMap<String, MqttTopic>,
    pub tx: broadcast::Sender<MqttMessage>,
    pub client: Option<AsyncClient>,
    pub connected: bool,
    pub connection_time: Option<Instant>,
    pub message_count: usize,
    pub last_error: Option<String>,
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
}

lazy_static::lazy_static! {
    pub static ref MQTT_STATE: Arc<Mutex<MqttState>> = {
        let (tx, _) = broadcast::channel(100);
        Arc::new(Mutex::new(MqttState {
            recent_messages: VecDeque::with_capacity(100),
            devices: HashMap::new(),
            topics: HashMap::new(),
            tx,
            client: None,
            connected: false,
            connection_time: None,
            message_count: 0,
            last_error: None,
            broker_host: "localhost".to_string(),
            broker_port: 1883,
            client_id: String::new(),
        }))
    };
}

pub async fn init_mqtt() {
    let host = env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("MQTT_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(1883);
    let user = env::var("MQTT_USER").ok();
    let pass = env::var("MQTT_PASSWORD").ok();

    info!("Initializing MQTT client for {}:{}", host, port);

    // Update state with broker info
    {
        let mut state = MQTT_STATE.lock().unwrap();
        state.broker_host = host.clone();
        state.broker_port = port;
    }

    let client_id = format!("kusanagi-backend-{}", chrono::Utc::now().timestamp() % 1000);
    
    {
        let mut state = MQTT_STATE.lock().unwrap();
        state.client_id = client_id.clone();
    }

    let mut mqttoptions = MqttOptions::new(client_id, &host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(30)); // Augmenté de 5s à 30s

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
            {
                let mut state = MQTT_STATE.lock().unwrap();
                state.last_error = Some(format!("Subscribe failed: {}", e));
            }
            return;
        }

        // Mark as connected
        {
            let mut state = MQTT_STATE.lock().unwrap();
            state.connected = true;
            state.connection_time = Some(Instant::now());
        }
        info!("MQTT connected and subscribed to all topics");

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
                    {
                        let mut state = MQTT_STATE.lock().unwrap();
                        state.connected = false;
                        state.last_error = Some(e.to_string());
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    // Start stats calculation task
    tokio::spawn(start_stats_task());
}

fn handle_incoming_message(msg: MqttMessage) {
    let mut state = MQTT_STATE.lock().unwrap();
    
    // Update message count
    state.message_count += 1;
    
    // Broadcast to WebSocket listeners
    let _ = state.tx.send(msg.clone());

    // Update recent messages
    debug!("Incoming MQTT message topic: {}, payload: {}", msg.topic, msg.payload);
    state.recent_messages.push_back(msg.clone());
    if state.recent_messages.len() > 100 {
        state.recent_messages.pop_front();
    }

    // Update topic stats
    let topic_entry = state.topics.entry(msg.topic.clone()).or_insert_with(|| {
        MqttTopic {
            name: msg.topic.clone(),
            message_count: 0,
            last_message: None,
            sub_topics: vec![],
        }
    });
    topic_entry.message_count += 1;
    topic_entry.last_message = Some(msg.clone());

    // Detect/Update device (naive detection based on first part of topic)
    let device_id = msg.topic.split('/').next().unwrap_or("unknown").to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let device = state.devices.entry(device_id.clone()).or_insert_with(|| {
        info!("New MQTT device detected: {}", device_id);
        MqttDevice {
            id: device_id.clone(),
            name: msg.topic.split('/').next().unwrap_or("Unknown Device").to_string(),
            last_seen: now.clone(),
            last_topic: msg.topic.clone(),
            message_count: 0,
            first_seen: now.clone(),
        }
    });

    device.last_seen = now;
    device.last_topic = msg.topic.clone();
    device.message_count += 1;

    // Bridge to Slack (only for important topics to avoid spam)
    if should_bridge_to_slack(&msg.topic) {
        let slack_msg = format!("*MQTT Message*\n*Topic*: `{}`\n*Payload*: `{}`", msg.topic, msg.payload);
        tokio::spawn(async move {
            if let Ok(slack) = crate::legacy::slack::SlackClient::new() {
                let _ = slack.send_message(&slack_msg).await;
            }
        });
    }
}

fn should_bridge_to_slack(topic: &str) -> bool {
    // Only bridge specific important topics to avoid Slack spam
    let important_patterns = ["alert", "error", "critical", "security", "motion"];
    let topic_lower = topic.to_lowercase();
    important_patterns.iter().any(|p| topic_lower.contains(p))
}

async fn start_stats_task() {
    let mut interval = interval(Duration::from_secs(60));
    let mut last_count = 0;
    
    loop {
        interval.tick().await;
        
        let state = MQTT_STATE.lock().unwrap();
        let current_count = state.message_count;
        let rate = (current_count - last_count) as f64;
        last_count = current_count;
        
        debug!("MQTT stats: {} total messages, {:.1} msg/min", current_count, rate);
    }
}

pub async fn publish_message(topic: &str, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = {
        let state = MQTT_STATE.lock().unwrap();
        state.client.clone()
    };

    if let Some(client) = client {
        client.publish(topic, QoS::AtMostOnce, false, payload).await?;
        info!("Published message to topic: {}", topic);
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

pub async fn get_mqtt_topics() -> Result<HttpResponse> {
    let state = MQTT_STATE.lock().unwrap();
    
    // Build topic tree
    let mut topic_tree: HashMap<String, MqttTopic> = HashMap::new();
    
    for (topic_name, topic) in state.topics.iter() {
        let parts: Vec<&str> = topic_name.split('/').collect();
        if parts.len() > 1 {
            let parent = parts[0..parts.len()-1].join("/");
            if let Some(parent_topic) = topic_tree.get_mut(&parent) {
                if !parent_topic.sub_topics.contains(topic_name) {
                    parent_topic.sub_topics.push(topic_name.clone());
                }
            }
        }
        topic_tree.insert(topic_name.clone(), topic.clone());
    }
    
    let topics: Vec<MqttTopic> = topic_tree.values().cloned().collect();
    Ok(HttpResponse::Ok().json(topics))
}

pub async fn get_mqtt_stats() -> Result<HttpResponse> {
    let state = MQTT_STATE.lock().unwrap();
    
    let uptime = state.connection_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
    let messages_per_minute = if uptime > 0 {
        (state.message_count as f64 / uptime as f64) * 60.0
    } else {
        0.0
    };
    
    let stats = MqttStats {
        total_messages: state.message_count,
        total_devices: state.devices.len(),
        total_topics: state.topics.len(),
        messages_per_minute,
        uptime_seconds: uptime,
    };
    
    Ok(HttpResponse::Ok().json(stats))
}

pub async fn get_mqtt_health() -> Result<HttpResponse> {
    let state = MQTT_STATE.lock().unwrap();
    
    let health = MqttHealth {
        status: if state.connected { "healthy".to_string() } else { "unhealthy".to_string() },
        connected: state.connected,
        broker_host: state.broker_host.clone(),
        broker_port: state.broker_port,
        client_id: state.client_id.clone(),
        last_error: state.last_error.clone(),
        connection_time: state.connection_time.map(|t| {
            let since = t.elapsed();
            format!("{:?} ago", since)
        }),
    };
    
    if state.connected {
        Ok(HttpResponse::Ok().json(health))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(health))
    }
}

#[derive(Deserialize)]
pub struct PublishRequest {
    topic: String,
    payload: String,
}

pub async fn publish_handler(body: web::Json<PublishRequest>) -> Result<HttpResponse> {
    match publish_message(&body.topic, &body.payload).await {
        Ok(()) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Message published"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/mqtt")
            .route("/messages", web::get().to(get_mqtt_messages))
            .route("/devices", web::get().to(get_mqtt_devices))
            .route("/topics", web::get().to(get_mqtt_topics))
            .route("/stats", web::get().to(get_mqtt_stats))
            .route("/health", web::get().to(get_mqtt_health))
            .route("/publish", web::post().to(publish_handler)),
    );
}
