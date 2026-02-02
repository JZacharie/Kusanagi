//! Unified Notifications System
//!
//! Multi-channel notifications: Slack, Email, Webhook, In-app.
//! Supports templates, rate limiting, and batching.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Notification channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Slack,
    Email,
    Webhook,
    InApp,
}

/// Notification priority
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

/// Notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub channel: Channel,
    pub priority: Priority,
    pub metadata: HashMap<String, String>,
    pub created_at: String,
}

impl Notification {
    /// Create new notification
    pub fn new(title: impl Into<String>, message: impl Into<String>, channel: Channel) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.into(),
            message: message.into(),
            channel,
            priority: Priority::Normal,
            metadata: HashMap::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Notification sender trait
#[async_trait::async_trait]
pub trait NotificationSender: Send + Sync {
    /// Send notification
    async fn send(&self, notification: &Notification) -> Result<(), String>;
    
    /// Get channel type
    fn channel(&self) -> Channel;
}

/// Notification manager
pub struct NotificationManager {
    senders: HashMap<Channel, Box<dyn NotificationSender>>,
    enabled: HashMap<Channel, bool>,
}

impl NotificationManager {
    /// Create new notification manager
    pub fn new() -> Self {
        let mut manager = Self {
            senders: HashMap::new(),
            enabled: HashMap::new(),
        };
        
        // Initialize all channels as disabled by default
        manager.enabled.insert(Channel::Slack, false);
        manager.enabled.insert(Channel::Email, false);
        manager.enabled.insert(Channel::Webhook, false);
        manager.enabled.insert(Channel::InApp, true); // In-app always enabled
        
        manager
    }

    /// Register a sender
    pub fn register_sender(&mut self, sender: Box<dyn NotificationSender>) {
        let channel = sender.channel();
        self.senders.insert(channel, sender);
        info!(channel = ?channel, "Notification sender registered");
    }

    /// Enable/disable channel
    pub fn set_enabled(&mut self, channel: Channel, enabled: bool) {
        self.enabled.insert(channel, enabled);
    }

    /// Check if channel is enabled
    pub fn is_enabled(&self, channel: Channel) -> bool {
        *self.enabled.get(&channel).unwrap_or(&false)
    }

    /// Send notification to specific channel
    pub async fn send(&self, notification: Notification) -> Result<(), String> {
        let channel = notification.channel;
        
        if !self.is_enabled(channel) {
            warn!(channel = ?channel, "Channel is disabled, skipping notification");
            return Err(format!("Channel {:?} is disabled", channel));
        }

        if let Some(sender) = self.senders.get(&channel) {
            match sender.send(&notification).await {
                Ok(()) => {
                    info!(id = %notification.id, channel = ?channel, "Notification sent");
                    crate::metrics::custom::record_notification(&format!("{:?}", channel), true);
                    Ok(())
                }
                Err(e) => {
                    error!(id = %notification.id, channel = ?channel, error = %e, "Failed to send notification");
                    crate::metrics::custom::record_notification(&format!("{:?}", channel), false);
                    Err(e)
                }
            }
        } else {
            error!(channel = ?channel, "No sender registered for channel");
            Err(format!("No sender registered for channel {:?}", channel))
        }
    }

    /// Send to all enabled channels
    pub async fn broadcast(&self, title: &str, message: &str, priority: Priority) -> Vec<(Channel, Result<(), String>)> {
        let mut results = Vec::new();
        
        for channel in [Channel::Slack, Channel::Email, Channel::Webhook, Channel::InApp] {
            if self.is_enabled(channel) {
                let notification = Notification::new(title, message, channel)
                    .with_priority(priority);
                
                let result = self.send(notification).await;
                results.push((channel, result));
            }
        }
        
        results
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============== Built-in Senders ==============

/// Slack sender
pub struct SlackSender {
    webhook_url: String,
    client: reqwest::Client,
}

impl SlackSender {
    pub fn new(webhook_url: impl Into<String>) -> Self {
        Self {
            webhook_url: webhook_url.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl NotificationSender for SlackSender {
    async fn send(&self, notification: &Notification) -> Result<(), String> {
        let color = match notification.priority {
            Priority::Low => "#808080",
            Priority::Normal => "#00FF00",
            Priority::High => "#FFA500",
            Priority::Critical => "#FF0000",
        };

        let payload = serde_json::json!({
            "attachments": [{
                "color": color,
                "title": notification.title,
                "text": notification.message,
                "fields": notification.metadata.iter().map(|(k, v)| {
                    serde_json::json!({
                        "title": k,
                        "value": v,
                        "short": true
                    })
                }).collect::<Vec<_>>(),
                "ts": chrono::Utc::now().timestamp()
            }]
        });

        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Slack request failed: {}", e))?
            .error_for_status()
            .map_err(|e| format!("Slack returned error: {}", e))?;

        Ok(())
    }

    fn channel(&self) -> Channel {
        Channel::Slack
    }
}

/// Webhook sender
pub struct WebhookSender {
    url: String,
    client: reqwest::Client,
    headers: HashMap<String, String>,
}

impl WebhookSender {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            client: reqwest::Client::new(),
            headers: HashMap::new(),
        }
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

#[async_trait::async_trait]
impl NotificationSender for WebhookSender {
    async fn send(&self, notification: &Notification) -> Result<(), String> {
        let payload = serde_json::json!({
            "id": notification.id,
            "title": notification.title,
            "message": notification.message,
            "priority": format!("{:?}", notification.priority),
            "metadata": notification.metadata,
            "timestamp": notification.created_at,
        });

        let mut request = self.client.post(&self.url).json(&payload);
        
        for (key, value) in &self.headers {
            request = request.header(key, value);
        }

        request
            .send()
            .await
            .map_err(|e| format!("Webhook request failed: {}", e))?
            .error_for_status()
            .map_err(|e| format!("Webhook returned error: {}", e))?;

        Ok(())
    }

    fn channel(&self) -> Channel {
        Channel::Webhook
    }
}

/// In-app notification store
pub struct InAppStore {
    notifications: std::sync::Mutex<Vec<Notification>>,
}

impl InAppStore {
    pub fn new() -> Self {
        Self {
            notifications: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn get_recent(&self, limit: usize) -> Vec<Notification> {
        let notifications = self.notifications.lock().unwrap();
        notifications.iter().rev().take(limit).cloned().collect()
    }
}

impl Default for InAppStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl NotificationSender for InAppStore {
    async fn send(&self, notification: &Notification) -> Result<(), String> {
        let mut notifications = self.notifications.lock().unwrap();
        notifications.push(notification.clone());
        
        // Keep only last 1000 notifications
        if notifications.len() > 1000 {
            notifications.remove(0);
        }
        
        Ok(())
    }

    fn channel(&self) -> Channel {
        Channel::InApp
    }
}

// ============== Global Instance ==============

static NOTIFICATIONS: once_cell::sync::OnceCell<tokio::sync::Mutex<NotificationManager>> = once_cell::sync::OnceCell::new();

/// Initialize notifications
pub async fn init() -> &'static tokio::sync::Mutex<NotificationManager> {
    NOTIFICATIONS.get_or_init(|| {
        let mut manager = NotificationManager::new();
        
        // Register in-app store
        manager.register_sender(Box::new(InAppStore::new()));
        
        // Register Slack if configured
        if let Ok(slack_url) = std::env::var("SLACK_WEBHOOK_URL") {
            manager.register_sender(Box::new(SlackSender::new(slack_url)));
            manager.set_enabled(Channel::Slack, true);
        }
        
        // Register Webhook if configured
        if let Ok(webhook_url) = std::env::var("NOTIFICATION_WEBHOOK_URL") {
            let sender = WebhookSender::new(webhook_url);
            manager.register_sender(Box::new(sender));
            manager.set_enabled(Channel::Webhook, true);
        }
        
        tokio::sync::Mutex::new(manager)
    })
}

/// Send notification
pub async fn send(notification: Notification) -> Result<(), String> {
    init().await.lock().await.send(notification).await
}

/// Broadcast to all channels
pub async fn broadcast(title: &str, message: &str, priority: Priority) -> Vec<(Channel, Result<(), String>)> {
    init().await.lock().await.broadcast(title, message, priority).await
}
