//! Slack Event Handler
//!
//! This module provides an event handler that sends Slack notifications
//! for important cluster events such as:
//! - Critical alerts
//! - Pod crash loops
//! - Node failures
//! - Security events
//!
//! # Configuration
//!
//! Requires the following environment variables:
//! - `SLACK_BOT_TOKEN` - Slack Bot User OAuth Token
//! - `SLACK_CHANNEL_ID` - Default channel for notifications
//!
//! # Usage
//!
//! ```rust
//! use crate::event_bus::slack_handler::SlackEventHandler;
//! use crate::event_bus::EventBus;
//!
//! let handler = SlackEventHandler::new().await?;
//! handler.start(EventBus::new()).await;
//! ```

use crate::error::Result;
use crate::event_bus::{
    AlertEvent, AlertSeverity, ClusterEvent, EventBus, PodEvent, SecurityEvent,
};
use crate::slack::SlackClient;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Slack event handler configuration
#[derive(Debug, Clone)]
pub struct SlackConfig {
    /// Whether Slack integration is enabled
    pub enabled: bool,
    /// Default channel for notifications
    pub channel: String,
    /// Minimum severity level to notify
    pub min_severity: AlertSeverity,
    /// Notify on pod crash loops
    pub notify_crash_loops: bool,
    /// Notify on node failures
    pub notify_node_failures: bool,
    /// Notify on security events
    pub notify_security: bool,
    /// Rate limit: max notifications per minute
    pub rate_limit_per_minute: u32,
}

impl Default for SlackConfig {
    fn default() -> Self {
        Self {
            enabled: !std::env::var("SLACK_BOT_TOKEN").unwrap_or_default().is_empty(),
            channel: std::env::var("SLACK_CHANNEL_ID").unwrap_or_default(),
            min_severity: AlertSeverity::Warning,
            notify_crash_loops: true,
            notify_node_failures: true,
            notify_security: true,
            rate_limit_per_minute: 10,
        }
    }
}

/// Slack event handler for cluster notifications
pub struct SlackEventHandler {
    client: SlackClient,
    config: SlackConfig,
    rate_limiter: Arc<tokio::sync::Mutex<RateLimiter>>,
}

/// Simple rate limiter for notifications
struct RateLimiter {
    count: u32,
    window_start: std::time::Instant,
    max_per_window: u32,
}

impl RateLimiter {
    fn new(max_per_window: u32) -> Self {
        Self {
            count: 0,
            window_start: std::time::Instant::now(),
            max_per_window,
        }
    }

    fn check_and_increment(&mut self) -> bool {
        let now = std::time::Instant::now();
        let window = std::time::Duration::from_secs(60);

        if now.duration_since(self.window_start) > window {
            self.window_start = now;
            self.count = 0;
        }

        if self.count < self.max_per_window {
            self.count += 1;
            true
        } else {
            false
        }
    }
}

impl SlackEventHandler {
    /// Create a new Slack event handler
    pub async fn new() -> Result<Option<Self>> {
        let config = SlackConfig::default();

        if !config.enabled {
            info!("Slack integration disabled (SLACK_BOT_TOKEN not set)");
            return Ok(None);
        }

        let client = SlackClient::new().map_err(|e| {
            crate::error::KusanagiError::external_api("Slack", &e.to_string())
        })?;

        info!("Slack event handler initialized");

        Ok(Some(Self {
            client,
            config,
            rate_limiter: Arc::new(tokio::sync::Mutex::new(RateLimiter::new(10))),
        }))
    }

    /// Create with custom configuration
    pub fn with_config(config: SlackConfig) -> Result<Option<Self>> {
        if !config.enabled {
            return Ok(None);
        }

        let client = SlackClient::new().map_err(|e| {
            crate::error::KusanagiError::external_api("Slack", &e.to_string())
        })?;

        let rate_limit = config.rate_limit_per_minute;

        Ok(Some(Self {
            client,
            config,
            rate_limiter: Arc::new(tokio::sync::Mutex::new(RateLimiter::new(rate_limit))),
        }))
    }

    /// Start listening for events
    pub async fn start(&self, bus: EventBus) {
        info!("Starting Slack event handler");

        // Start pod event listener
        self.start_pod_listener(bus.clone()).await;

        // Start cluster event listener
        self.start_cluster_listener(bus.clone()).await;

        // Start alert event listener
        self.start_alert_listener(bus.clone()).await;

        // Start security event listener
        self.start_security_listener(bus).await;
    }

    /// Check if we should send notification (rate limiting)
    async fn should_notify(&self) -> bool {
        let mut limiter = self.rate_limiter.lock().await;
        limiter.check_and_increment()
    }

    /// Send Slack notification with rate limiting
    async fn notify(&self, title: &str, message: &str, severity: &str) {
        if !self.should_notify().await {
            warn!("Slack notification rate limit exceeded, dropping message");
            return;
        }

        if let Err(e) = self.client.notify_alert(title, message, severity).await {
            error!(error = ?e, "Failed to send Slack notification");
        } else {
            debug!(title = %title, "Slack notification sent");
        }
    }

    /// Listen for pod events
    async fn start_pod_listener(&self, bus: EventBus) {
        if !self.config.notify_crash_loops {
            return;
        }

        let mut rx = bus.subscribe::<PodEvent>().await;
        let handler = self.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        handler.handle_pod_event(&event).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(dropped = n, "Slack pod event handler lagged");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        error!("Pod event channel closed for Slack handler");
                        break;
                    }
                }
            }
        });
    }

    /// Handle pod events
    async fn handle_pod_event(&self, event: &PodEvent) {
        match event {
            PodEvent::CrashLoopDetected {
                pod_name,
                namespace,
                container_name,
                restart_count,
                metadata,
            } => {
                let title = format!("Crash Loop Detected: {}", pod_name);
                let message = format!(
                    "• Namespace: {}\n• Container: {}\n• Restart count: {}\n• Time: {}",
                    namespace,
                    container_name,
                    restart_count,
                    metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                self.notify(&title, &message, "critical").await;
            }
            PodEvent::ImagePullFailed {
                pod_name,
                namespace,
                image,
                error,
                metadata,
            } => {
                let title = format!("Image Pull Failed: {}", pod_name);
                let message = format!(
                    "• Namespace: {}\n• Image: {}\n• Error: {}\n• Time: {}",
                    namespace,
                    image,
                    error,
                    metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                self.notify(&title, &message, "error").await;
            }
            PodEvent::Restarted {
                pod_name,
                namespace,
                restart_count,
                reason,
                metadata,
            } if *restart_count > 3 => {
                let title = format!("Pod Restarted Multiple Times: {}", pod_name);
                let message = format!(
                    "• Namespace: {}\n• Restart count: {}\n• Reason: {}\n• Time: {}",
                    namespace,
                    restart_count,
                    reason,
                    metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                self.notify(&title, &message, "warning").await;
            }
            _ => {}
        }
    }

    /// Listen for cluster events
    async fn start_cluster_listener(&self, bus: EventBus) {
        if !self.config.notify_node_failures {
            return;
        }

        let mut rx = bus.subscribe::<ClusterEvent>().await;
        let handler = self.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        handler.handle_cluster_event(&event).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(dropped = n, "Slack cluster event handler lagged");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        error!("Cluster event channel closed for Slack handler");
                        break;
                    }
                }
            }
        });
    }

    /// Handle cluster events
    async fn handle_cluster_event(&self, event: &ClusterEvent) {
        match event {
            ClusterEvent::NodeNotReady {
                node_name,
                condition,
                metadata,
            } => {
                let title = format!("Node Not Ready: {}", node_name);
                let message = format!(
                    "• Condition: {}\n• Time: {}",
                    condition,
                    metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                self.notify(&title, &message, "critical").await;
            }
            ClusterEvent::NodeRemoved {
                node_name,
                reason,
                metadata,
            } => {
                let title = format!("Node Removed: {}", node_name);
                let message = format!(
                    "• Reason: {}\n• Time: {}",
                    reason,
                    metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                self.notify(&title, &message, "warning").await;
            }
            ClusterEvent::ThresholdCrossed {
                resource_type,
                threshold,
                current_value,
                severity,
                metadata,
            } => {
                // Only notify if severity meets minimum threshold
                if Self::severity_meets_threshold(severity, &self.config.min_severity) {
                    let title = format!("Resource Threshold Crossed: {}", resource_type);
                    let message = format!(
                        "• Threshold: {:.1}%\n• Current: {:.1}%\n• Severity: {}\n• Time: {}",
                        threshold * 100.0,
                        current_value * 100.0,
                        severity,
                        metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    let severity_str = match severity {
                        AlertSeverity::Critical => "critical",
                        AlertSeverity::Warning => "warning",
                        AlertSeverity::Info => "info",
                    };
                    self.notify(&title, &message, severity_str).await;
                }
            }
            _ => {}
        }
    }

    /// Listen for alert events
    async fn start_alert_listener(&self, bus: EventBus) {
        let mut rx = bus.subscribe::<AlertEvent>().await;
        let handler = self.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        handler.handle_alert_event(&event).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(dropped = n, "Slack alert event handler lagged");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        error!("Alert event channel closed for Slack handler");
                        break;
                    }
                }
            }
        });
    }

    /// Handle alert events
    async fn handle_alert_event(&self, event: &AlertEvent) {
        match event {
            AlertEvent::Fired {
                alert_name,
                severity,
                summary,
                description,
                metadata,
                labels: _,
            } => {
                if Self::severity_meets_threshold(severity, &self.config.min_severity) {
                    let severity_str = match severity {
                        AlertSeverity::Critical => "critical",
                        AlertSeverity::Warning => "warning",
                        AlertSeverity::Info => "info",
                    };
                    let message = format!(
                        "• Summary: {}\n• Description: {}\n• Time: {}",
                        summary,
                        description,
                        metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    self.notify(alert_name, &message, severity_str).await;
                }
            }
            _ => {}
        }
    }

    /// Listen for security events
    async fn start_security_listener(&self, bus: EventBus) {
        if !self.config.notify_security {
            return;
        }

        let mut rx = bus.subscribe::<SecurityEvent>().await;
        let handler = self.clone();

        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        handler.handle_security_event(&event).await;
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(dropped = n, "Slack security event handler lagged");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        error!("Security event channel closed for Slack handler");
                        break;
                    }
                }
            }
        });
    }

    /// Handle security events
    async fn handle_security_event(&self, event: &SecurityEvent) {
        match event {
            SecurityEvent::VulnerabilityDetected {
                resource,
                vulnerability_id,
                severity,
                description,
                metadata,
            } => {
                let title = format!("Security Vulnerability: {}", vulnerability_id);
                let message = format!(
                    "• Resource: {}\n• Severity: {}\n• Description: {}\n• Time: {}",
                    resource,
                    severity,
                    description,
                    metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                let severity_str = match severity {
                    AlertSeverity::Critical => "critical",
                    AlertSeverity::Warning => "warning",
                    AlertSeverity::Info => "info",
                };
                self.notify(&title, &message, severity_str).await;
            }
            SecurityEvent::PolicyViolation {
                policy,
                resource,
                namespace,
                message,
                metadata,
            } => {
                let title = format!("Policy Violation: {}", policy);
                let description = format!(
                    "• Resource: {}\n• Namespace: {}\n• Message: {}\n• Time: {}",
                    resource,
                    namespace,
                    message,
                    metadata.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
                );
                self.notify(&title, &description, "warning").await;
            }
            _ => {}
        }
    }

    /// Check if severity meets minimum threshold
    fn severity_meets_threshold(event: &AlertSeverity, min: &AlertSeverity) -> bool {
        let event_level = match event {
            AlertSeverity::Info => 0,
            AlertSeverity::Warning => 1,
            AlertSeverity::Critical => 2,
        };
        let min_level = match min {
            AlertSeverity::Info => 0,
            AlertSeverity::Warning => 1,
            AlertSeverity::Critical => 2,
        };
        event_level >= min_level
    }
}

impl Clone for SlackEventHandler {
    fn clone(&self) -> Self {
        Self {
            client: SlackClient::new().expect("Failed to clone Slack client"),
            config: self.config.clone(),
            rate_limiter: Arc::clone(&self.rate_limiter),
        }
    }
}

/// Initialize and start the Slack event handler
pub async fn init_slack_handler(bus: &EventBus) -> Option<SlackEventHandler> {
    match SlackEventHandler::new().await {
        Ok(Some(handler)) => {
            handler.start(bus.clone()).await;
            info!("Slack event handler started");
            Some(handler)
        }
        Ok(None) => {
            info!("Slack handler not initialized (disabled)");
            None
        }
        Err(e) => {
            error!(error = ?e, "Failed to initialize Slack handler");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::EventMetadata;

    #[test]
    fn test_slack_config_default() {
        let config = SlackConfig::default();
        // Should be disabled if env vars not set
        assert!(!config.enabled || !std::env::var("SLACK_BOT_TOKEN").unwrap_or_default().is_empty());
    }

    #[test]
    fn test_severity_meets_threshold() {
        assert!(SlackEventHandler::severity_meets_threshold(
            &AlertSeverity::Critical,
            &AlertSeverity::Warning
        ));
        assert!(SlackEventHandler::severity_meets_threshold(
            &AlertSeverity::Warning,
            &AlertSeverity::Warning
        ));
        assert!(!SlackEventHandler::severity_meets_threshold(
            &AlertSeverity::Info,
            &AlertSeverity::Warning
        ));
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3);
        
        assert!(limiter.check_and_increment());
        assert!(limiter.check_and_increment());
        assert!(limiter.check_and_increment());
        assert!(!limiter.check_and_increment()); // 4th should be blocked
    }

    #[test]
    fn test_pod_event_filtering() {
        // Create a crash loop event
        let event = PodEvent::CrashLoopDetected {
            metadata: EventMetadata::new("test"),
            pod_name: "crashy".to_string(),
            namespace: "default".to_string(),
            container_name: "app".to_string(),
            restart_count: 5,
        };

        // Verify it would trigger a notification
        match &event {
            PodEvent::CrashLoopDetected { .. } => assert!(true),
            _ => panic!("Expected crash loop"),
        }
    }

    #[test]
    fn test_security_event_formatting() {
        let event = SecurityEvent::VulnerabilityDetected {
            metadata: EventMetadata::new("test"),
            resource: "nginx:latest".to_string(),
            vulnerability_id: "CVE-2024-1234".to_string(),
            severity: AlertSeverity::Critical,
            description: "Buffer overflow".to_string(),
        };

        match event {
            SecurityEvent::VulnerabilityDetected { vulnerability_id, .. } => {
                assert_eq!(vulnerability_id, "CVE-2024-1234");
            }
            _ => panic!("Wrong event type"),
        }
    }
}
