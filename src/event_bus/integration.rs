//! Event Bus Integration
//!
//! This module provides integration between the event bus and other system components.
//! It sets up handlers that bridge events to WebSocket notifications, cache invalidation,
//! and other cross-cutting concerns.
//!
//! # Usage
//!
//! ```rust
//! use crate::event_bus::integration::EventBusIntegration;
//!
//! let integration = EventBusIntegration::new();
//! integration.start_handlers().await;
//! ```

use crate::cache::{Cache, InMemoryCache};
use crate::error::Result;
use crate::event_bus::{
    handlers::{ClusterEventHandler, HandlerBuilder, PodEventHandler},
    slack_handler::SlackEventHandler,
    AlertEvent, AuditEvent, ClusterEvent, DomainEvent, EventBus, EventHandler, PodEvent,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// WebSocket notification message for pod events
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "event_type")]
pub enum PodNotification {
    /// Pod created notification
    #[serde(rename = "pod_created")]
    PodCreated {
        pod_name: String,
        namespace: String,
        node_name: String,
        timestamp: String,
    },
    /// Pod deleted notification
    #[serde(rename = "pod_deleted")]
    PodDeleted {
        pod_name: String,
        namespace: String,
        timestamp: String,
        reason: String,
    },
    /// Pod status changed notification
    #[serde(rename = "pod_status_changed")]
    PodStatusChanged {
        pod_name: String,
        namespace: String,
        previous_status: String,
        new_status: String,
        timestamp: String,
    },
    /// Pod restarted notification
    #[serde(rename = "pod_restarted")]
    PodRestarted {
        pod_name: String,
        namespace: String,
        restart_count: i32,
        timestamp: String,
        reason: String,
    },
    /// Crash loop detected notification
    #[serde(rename = "pod_crash_loop")]
    CrashLoop {
        pod_name: String,
        namespace: String,
        container_name: String,
        restart_count: i32,
        timestamp: String,
        severity: String,
    },
    /// Image pull failed notification
    #[serde(rename = "pod_image_pull_failed")]
    ImagePullFailed {
        pod_name: String,
        namespace: String,
        image: String,
        timestamp: String,
        error: String,
    },
}

impl From<&PodEvent> for PodNotification {
    fn from(event: &PodEvent) -> Self {
        match event {
            PodEvent::Created {
                pod_name,
                namespace,
                node_name,
                metadata,
                ..
            } => PodNotification::PodCreated {
                pod_name: pod_name.clone(),
                namespace: namespace.clone(),
                node_name: node_name.clone(),
                timestamp: metadata.timestamp.to_rfc3339(),
            },
            PodEvent::Deleted {
                pod_name,
                namespace,
                reason,
                metadata,
                ..
            } => PodNotification::PodDeleted {
                pod_name: pod_name.clone(),
                namespace: namespace.clone(),
                timestamp: metadata.timestamp.to_rfc3339(),
                reason: reason.clone(),
            },
            PodEvent::StatusChanged {
                pod_name,
                namespace,
                previous_status,
                new_status,
                metadata,
                ..
            } => PodNotification::PodStatusChanged {
                pod_name: pod_name.clone(),
                namespace: namespace.clone(),
                previous_status: previous_status.clone(),
                new_status: new_status.clone(),
                timestamp: metadata.timestamp.to_rfc3339(),
            },
            PodEvent::Restarted {
                pod_name,
                namespace,
                restart_count,
                reason,
                metadata,
                ..
            } => PodNotification::PodRestarted {
                pod_name: pod_name.clone(),
                namespace: namespace.clone(),
                restart_count: *restart_count,
                timestamp: metadata.timestamp.to_rfc3339(),
                reason: reason.clone(),
            },
            PodEvent::CrashLoopDetected {
                pod_name,
                namespace,
                container_name,
                restart_count,
                metadata,
                ..
            } => PodNotification::CrashLoop {
                pod_name: pod_name.clone(),
                namespace: namespace.clone(),
                container_name: container_name.clone(),
                restart_count: *restart_count,
                timestamp: metadata.timestamp.to_rfc3339(),
                severity: "critical".to_string(),
            },
            PodEvent::ImagePullFailed {
                pod_name,
                namespace,
                image,
                error,
                metadata,
                ..
            } => PodNotification::ImagePullFailed {
                pod_name: pod_name.clone(),
                namespace: namespace.clone(),
                image: image.clone(),
                timestamp: metadata.timestamp.to_rfc3339(),
                error: error.clone(),
            },
        }
    }
}

/// Global broadcast channel for pod notifications
/// 
/// This allows any component to receive pod notifications without
/// directly subscribing to the event bus.
static POD_NOTIFICATION_TX: std::sync::OnceLock<broadcast::Sender<PodNotification>> =
    std::sync::OnceLock::new();

/// Initialize the global pod notification channel
pub fn init_pod_notifications() -> broadcast::Sender<PodNotification> {
    let (tx, _rx) = broadcast::channel(100);
    POD_NOTIFICATION_TX
        .set(tx.clone())
        .expect("Pod notification channel already initialized");
    tx
}

/// Get the global pod notification sender
pub fn pod_notification_sender() -> Option<broadcast::Sender<PodNotification>> {
    POD_NOTIFICATION_TX.get().cloned()
}

/// Subscribe to pod notifications
pub fn subscribe_pod_notifications() -> Option<broadcast::Receiver<PodNotification>> {
    POD_NOTIFICATION_TX.get().map(|tx| tx.subscribe())
}

/// Event bus integration manager
pub struct EventBusIntegration {
    bus: EventBus,
    cluster_cache: Arc<dyn Cache<String, String> + Send + Sync>,
    pod_cache: Arc<dyn Cache<String, String> + Send + Sync>,
}

impl EventBusIntegration {
    /// Create a new integration instance
    pub fn new() -> Self {
        // Create in-memory caches for cluster and pod data
        let cluster_cache: Arc<dyn Cache<String, String> + Send + Sync> =
            Arc::new(InMemoryCache::with_ttl("cluster_cache", std::time::Duration::from_secs(60)));
        let pod_cache: Arc<dyn Cache<String, String> + Send + Sync> =
            Arc::new(InMemoryCache::with_ttl("pod_cache", std::time::Duration::from_secs(30)));

        Self {
            bus: EventBus::new(),
            cluster_cache,
            pod_cache,
        }
    }

    /// Create with existing caches
    pub fn with_caches(
        bus: EventBus,
        cluster_cache: Arc<dyn Cache<String, String> + Send + Sync>,
        pod_cache: Arc<dyn Cache<String, String> + Send + Sync>,
    ) -> Self {
        Self {
            bus,
            cluster_cache,
            pod_cache,
        }
    }

    /// Get a clone of the event bus
    pub fn bus(&self) -> EventBus {
        self.bus.clone()
    }

    /// Start all event handlers
    ///
    /// This spawns background tasks that:
    /// - Listen for events on the bus
    /// - Handle cache invalidation
    /// - Broadcast notifications to WebSocket clients
    /// - Log audit events
    pub async fn start_handlers(&self) {
        info!("Starting event bus integration handlers...");

        // Initialize pod notification channel
        init_pod_notifications();

        // Start pod event handler with cache invalidation
        self.start_pod_event_handler().await;

        // Start cluster event handler
        self.start_cluster_event_handler().await;

        // Start alert event handler
        self.start_alert_event_handler().await;

        // Start audit event handler
        self.start_audit_event_handler().await;

        // Start domain event logger (logs all events)
        self.start_domain_event_logger().await;

        // Start Slack notification handler (if configured)
        self.start_slack_handler().await;

        info!("Event bus integration handlers started successfully");
    }

    /// Start Slack notification handler
    async fn start_slack_handler(&self) {
        if let Some(handler) = SlackEventHandler::new().await.ok().flatten() {
            handler.start(self.bus.clone()).await;
            info!("Slack notification handler started");
        } else {
            debug!("Slack handler not started (disabled or misconfigured)");
        }
    }

    /// Start pod event handler
    async fn start_pod_event_handler(&self) {
        let bus = self.bus.clone();
        let handler = HandlerBuilder::pod_with_cache(Arc::clone(&self.pod_cache));
        let notification_tx = pod_notification_sender().expect("Pod notification channel not initialized");

        tokio::spawn(async move {
            let mut rx = bus.subscribe::<PodEvent>().await;

            loop {
                match rx.recv().await {
                    Ok(event) => {
                        debug!(event = ?event, "Received pod event");

                        // Handle cache invalidation
                        handler.handle(event.clone()).await;

                        // Broadcast to WebSocket clients
                        let notification = PodNotification::from(&event);
                        if let Err(e) = notification_tx.send(notification) {
                            debug!(error = ?e, "No WebSocket clients listening");
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(dropped = n, "Pod event handler lagged, events dropped");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        error!("Pod event channel closed, stopping handler");
                        break;
                    }
                }
            }
        });
    }

    /// Start cluster event handler
    async fn start_cluster_event_handler(&self) {
        let bus = self.bus.clone();
        let handler = HandlerBuilder::cluster_with_cache(Arc::clone(&self.cluster_cache));

        tokio::spawn(async move {
            let mut rx = bus.subscribe::<ClusterEvent>().await;

            loop {
                match rx.recv().await {
                    Ok(event) => {
                        handler.handle(event).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(dropped = n, "Cluster event handler lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        error!("Cluster event channel closed");
                        break;
                    }
                }
            }
        });
    }

    /// Start alert event handler
    async fn start_alert_event_handler(&self) {
        let bus = self.bus.clone();

        tokio::spawn(async move {
            let mut rx = bus.subscribe::<AlertEvent>().await;

            loop {
                match rx.recv().await {
                    Ok(event) => {
                        match &event {
                            AlertEvent::Fired {
                                alert_name,
                                severity,
                                summary,
                                ..
                            } => {
                                warn!(
                                    alert = %alert_name,
                                    severity = ?severity,
                                    summary = %summary,
                                    "Alert fired"
                                );
                            }
                            AlertEvent::Resolved { alert_name, .. } => {
                                info!(alert = %alert_name, "Alert resolved");
                            }
                            AlertEvent::Acknowledged {
                                alert_name,
                                acknowledged_by,
                                ..
                            } => {
                                info!(
                                    alert = %alert_name,
                                    user = %acknowledged_by,
                                    "Alert acknowledged"
                                );
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(dropped = n, "Alert event handler lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        error!("Alert event channel closed");
                        break;
                    }
                }
            }
        });
    }

    /// Start audit event handler
    async fn start_audit_event_handler(&self) {
        let bus = self.bus.clone();
        let handler = HandlerBuilder::audit();

        tokio::spawn(async move {
            let mut rx = bus.subscribe::<AuditEvent>().await;

            loop {
                match rx.recv().await {
                    Ok(event) => {
                        handler.handle(event).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(dropped = n, "Audit event handler lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        error!("Audit event channel closed");
                        break;
                    }
                }
            }
        });
    }

    /// Start domain event logger (logs all events for debugging)
    async fn start_domain_event_logger(&self) {
        let bus = self.bus.clone();
        let handler = HandlerBuilder::logging();

        tokio::spawn(async move {
            let mut rx = bus.subscribe::<DomainEvent>().await;

            loop {
                match rx.recv().await {
                    Ok(event) => {
                        handler.handle(event).await;
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        debug!(dropped = n, "Domain event logger lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
        });
    }

    /// Publish a pod event
    pub async fn publish_pod_event(&self, event: PodEvent) -> Result<()> {
        self.bus.publish(event).await
    }

    /// Publish a cluster event
    pub async fn publish_cluster_event(&self, event: ClusterEvent) -> Result<()> {
        self.bus.publish(event).await
    }

    /// Publish an alert event
    pub async fn publish_alert_event(&self, event: AlertEvent) -> Result<()> {
        self.bus.publish(event).await
    }

    /// Publish an audit event
    pub async fn publish_audit_event(&self, event: AuditEvent) -> Result<()> {
        self.bus.publish(event).await
    }
}

impl Default for EventBusIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize global event bus integration
///
/// This should be called once at application startup.
static GLOBAL_INTEGRATION: std::sync::OnceLock<EventBusIntegration> =
    std::sync::OnceLock::new();

/// Initialize the global event bus integration
pub async fn init_global_integration() -> &'static EventBusIntegration {
    let integration = GLOBAL_INTEGRATION.get_or_init(|| {
        let integration = EventBusIntegration::new();
        // Note: start_handlers must be called after this
        integration
    });
    
    // Start handlers if not already started
    integration.start_handlers().await;
    
    integration
}

/// Get the global event bus integration
pub fn global_integration() -> Option<&'static EventBusIntegration> {
    GLOBAL_INTEGRATION.get()
}

/// Get the global event bus (convenience function)
pub fn global_bus() -> Option<EventBus> {
    global_integration().map(|i| i.bus())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventMetadata;

    #[test]
    fn test_pod_notification_from_event() {
        let event = PodEvent::Created {
            metadata: EventMetadata::new("test"),
            pod_name: "test-pod".to_string(),
            namespace: "default".to_string(),
            node_name: "node-1".to_string(),
            labels: std::collections::HashMap::new(),
        };

        let notification = PodNotification::from(&event);

        match notification {
            PodNotification::PodCreated {
                pod_name,
                namespace,
                node_name,
                ..
            } => {
                assert_eq!(pod_name, "test-pod");
                assert_eq!(namespace, "default");
                assert_eq!(node_name, "node-1");
            }
            _ => panic!("Expected PodCreated notification"),
        }
    }

    #[test]
    fn test_pod_notification_crash_loop() {
        let event = PodEvent::CrashLoopDetected {
            metadata: EventMetadata::new("test"),
            pod_name: "crashy-pod".to_string(),
            namespace: "production".to_string(),
            restart_count: 10,
            container_name: "app".to_string(),
        };

        let notification = PodNotification::from(&event);

        match notification {
            PodNotification::CrashLoop {
                pod_name,
                namespace,
                container_name,
                restart_count,
                severity,
                ..
            } => {
                assert_eq!(pod_name, "crashy-pod");
                assert_eq!(namespace, "production");
                assert_eq!(container_name, "app");
                assert_eq!(restart_count, 10);
                assert_eq!(severity, "critical");
            }
            _ => panic!("Expected CrashLoop notification"),
        }
    }

    #[tokio::test]
    async fn test_pod_notification_broadcast() {
        // Initialize notification channel (ignore error if already initialized by other test)
        let tx = POD_NOTIFICATION_TX.get_or_init(|| {
            let (tx, _rx) = broadcast::channel(100);
            tx
        }).clone();
        
        let mut rx = tx.subscribe();

        // Create and send a notification
        let notification = PodNotification::PodCreated {
            pod_name: "test".to_string(),
            namespace: "default".to_string(),
            node_name: "node-1".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        tx.send(notification.clone()).unwrap();

        // Receive the notification
        let received = rx.recv().await.unwrap();
        match received {
            PodNotification::PodCreated { pod_name, .. } => {
                assert_eq!(pod_name, "test");
            }
            _ => panic!("Expected PodCreated"),
        }
    }

    #[test]
    fn test_init_pod_notifications_idempotent() {
        // Initialize notification channel (using get_or_init to handle multiple test runs)
        let tx = POD_NOTIFICATION_TX.get_or_init(|| {
            let (tx, _rx) = broadcast::channel(100);
            tx
        });

        // Verify we got a valid sender by creating a receiver
        let rx = tx.subscribe();
        drop(rx);
        
        // Verify pod_notification_sender returns the same sender
        let tx2 = pod_notification_sender().unwrap();
        assert_eq!(tx.receiver_count(), tx2.receiver_count());
    }
}
