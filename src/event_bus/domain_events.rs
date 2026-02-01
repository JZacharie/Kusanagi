//! Domain events for Kusanagi
//!
//! These events represent significant business occurrences in the system.
//! They are used for:
//! - Audit trails
//! - Reactive processing
//! - Cross-cutting concerns
//! - Integration with external systems

use crate::error::Result;
use crate::event_bus::Event;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// UUID generation using timestamp + random
fn generate_uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let random = rand::random::<u64>();
    format!("{:x}-{:x}", timestamp, random)
}

/// Base event data shared by all domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: String,
    pub correlation_id: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub version: String,
}

impl EventMetadata {
    /// Create new metadata
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            event_id: generate_uuid(),
            correlation_id: generate_uuid(),
            timestamp: Utc::now(),
            source: source.into(),
            version: "1.0".to_string(),
        }
    }
    
    /// Create with specific correlation ID
    pub fn with_correlation(correlation_id: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            event_id: generate_uuid(),
            correlation_id: correlation_id.into(),
            timestamp: Utc::now(),
            source: source.into(),
            version: "1.0".to_string(),
        }
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new("kusanagi")
    }
}

/// Kubernetes cluster events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClusterEvent {
    /// Cluster state changed
    StateChanged {
        metadata: EventMetadata,
        previous_state: String,
        new_state: String,
        reason: String,
    },
    /// Resource utilization threshold crossed
    ThresholdCrossed {
        metadata: EventMetadata,
        resource_type: String,
        threshold: f64,
        current_value: f64,
        severity: AlertSeverity,
    },
    /// New node added
    NodeAdded {
        metadata: EventMetadata,
        node_name: String,
        node_role: String,
    },
    /// Node removed
    NodeRemoved {
        metadata: EventMetadata,
        node_name: String,
        reason: String,
    },
    /// Node became not ready
    NodeNotReady {
        metadata: EventMetadata,
        node_name: String,
        condition: String,
    },
}

impl Event for ClusterEvent {
    fn event_type(&self) -> &'static str {
        match self {
            ClusterEvent::StateChanged { .. } => "Cluster.StateChanged",
            ClusterEvent::ThresholdCrossed { .. } => "Cluster.ThresholdCrossed",
            ClusterEvent::NodeAdded { .. } => "Cluster.NodeAdded",
            ClusterEvent::NodeRemoved { .. } => "Cluster.NodeRemoved",
            ClusterEvent::NodeNotReady { .. } => "Cluster.NodeNotReady",
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            ClusterEvent::StateChanged { metadata, .. } => metadata.timestamp,
            ClusterEvent::ThresholdCrossed { metadata, .. } => metadata.timestamp,
            ClusterEvent::NodeAdded { metadata, .. } => metadata.timestamp,
            ClusterEvent::NodeRemoved { metadata, .. } => metadata.timestamp,
            ClusterEvent::NodeNotReady { metadata, .. } => metadata.timestamp,
        }
    }
    
    fn correlation_id(&self) -> &str {
        match self {
            ClusterEvent::StateChanged { metadata, .. } => &metadata.correlation_id,
            ClusterEvent::ThresholdCrossed { metadata, .. } => &metadata.correlation_id,
            ClusterEvent::NodeAdded { metadata, .. } => &metadata.correlation_id,
            ClusterEvent::NodeRemoved { metadata, .. } => &metadata.correlation_id,
            ClusterEvent::NodeNotReady { metadata, .. } => &metadata.correlation_id,
        }
    }
    
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Pod lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PodEvent {
    /// Pod created
    Created {
        metadata: EventMetadata,
        pod_name: String,
        namespace: String,
        node_name: String,
        labels: HashMap<String, String>,
    },
    /// Pod deleted
    Deleted {
        metadata: EventMetadata,
        pod_name: String,
        namespace: String,
        reason: String,
    },
    /// Pod status changed
    StatusChanged {
        metadata: EventMetadata,
        pod_name: String,
        namespace: String,
        previous_status: String,
        new_status: String,
    },
    /// Pod restarted
    Restarted {
        metadata: EventMetadata,
        pod_name: String,
        namespace: String,
        restart_count: i32,
        reason: String,
    },
    /// Pod crash loop detected
    CrashLoopDetected {
        metadata: EventMetadata,
        pod_name: String,
        namespace: String,
        restart_count: i32,
        container_name: String,
    },
    /// Image pull failed
    ImagePullFailed {
        metadata: EventMetadata,
        pod_name: String,
        namespace: String,
        image: String,
        error: String,
    },
}

impl Event for PodEvent {
    fn event_type(&self) -> &'static str {
        match self {
            PodEvent::Created { .. } => "Pod.Created",
            PodEvent::Deleted { .. } => "Pod.Deleted",
            PodEvent::StatusChanged { .. } => "Pod.StatusChanged",
            PodEvent::Restarted { .. } => "Pod.Restarted",
            PodEvent::CrashLoopDetected { .. } => "Pod.CrashLoopDetected",
            PodEvent::ImagePullFailed { .. } => "Pod.ImagePullFailed",
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            PodEvent::Created { metadata, .. } => metadata.timestamp,
            PodEvent::Deleted { metadata, .. } => metadata.timestamp,
            PodEvent::StatusChanged { metadata, .. } => metadata.timestamp,
            PodEvent::Restarted { metadata, .. } => metadata.timestamp,
            PodEvent::CrashLoopDetected { metadata, .. } => metadata.timestamp,
            PodEvent::ImagePullFailed { metadata, .. } => metadata.timestamp,
        }
    }
    
    fn correlation_id(&self) -> &str {
        match self {
            PodEvent::Created { metadata, .. } => &metadata.correlation_id,
            PodEvent::Deleted { metadata, .. } => &metadata.correlation_id,
            PodEvent::StatusChanged { metadata, .. } => &metadata.correlation_id,
            PodEvent::Restarted { metadata, .. } => &metadata.correlation_id,
            PodEvent::CrashLoopDetected { metadata, .. } => &metadata.correlation_id,
            PodEvent::ImagePullFailed { metadata, .. } => &metadata.correlation_id,
        }
    }
    
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Alert events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertEvent {
    /// Alert fired
    Fired {
        metadata: EventMetadata,
        alert_name: String,
        severity: AlertSeverity,
        summary: String,
        description: String,
        labels: HashMap<String, String>,
    },
    /// Alert resolved
    Resolved {
        metadata: EventMetadata,
        alert_name: String,
        resolved_at: DateTime<Utc>,
    },
    /// Alert acknowledged by user
    Acknowledged {
        metadata: EventMetadata,
        alert_name: String,
        acknowledged_by: String,
        note: String,
    },
}

impl Event for AlertEvent {
    fn event_type(&self) -> &'static str {
        match self {
            AlertEvent::Fired { .. } => "Alert.Fired",
            AlertEvent::Resolved { .. } => "Alert.Resolved",
            AlertEvent::Acknowledged { .. } => "Alert.Acknowledged",
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            AlertEvent::Fired { metadata, .. } => metadata.timestamp,
            AlertEvent::Resolved { metadata, .. } => metadata.timestamp,
            AlertEvent::Acknowledged { metadata, .. } => metadata.timestamp,
        }
    }
    
    fn correlation_id(&self) -> &str {
        match self {
            AlertEvent::Fired { metadata, .. } => &metadata.correlation_id,
            AlertEvent::Resolved { metadata, .. } => &metadata.correlation_id,
            AlertEvent::Acknowledged { metadata, .. } => &metadata.correlation_id,
        }
    }
    
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Security events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEvent {
    /// Vulnerability detected
    VulnerabilityDetected {
        metadata: EventMetadata,
        resource: String,
        vulnerability_id: String,
        severity: AlertSeverity,
        description: String,
    },
    /// Policy violation detected
    PolicyViolation {
        metadata: EventMetadata,
        policy: String,
        resource: String,
        namespace: String,
        message: String,
    },
    /// Suspicious network activity
    SuspiciousActivity {
        metadata: EventMetadata,
        source_ip: String,
        destination_ip: String,
        activity_type: String,
        details: String,
    },
}

impl Event for SecurityEvent {
    fn event_type(&self) -> &'static str {
        match self {
            SecurityEvent::VulnerabilityDetected { .. } => "Security.VulnerabilityDetected",
            SecurityEvent::PolicyViolation { .. } => "Security.PolicyViolation",
            SecurityEvent::SuspiciousActivity { .. } => "Security.SuspiciousActivity",
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            SecurityEvent::VulnerabilityDetected { metadata, .. } => metadata.timestamp,
            SecurityEvent::PolicyViolation { metadata, .. } => metadata.timestamp,
            SecurityEvent::SuspiciousActivity { metadata, .. } => metadata.timestamp,
        }
    }
    
    fn correlation_id(&self) -> &str {
        match self {
            SecurityEvent::VulnerabilityDetected { metadata, .. } => &metadata.correlation_id,
            SecurityEvent::PolicyViolation { metadata, .. } => &metadata.correlation_id,
            SecurityEvent::SuspiciousActivity { metadata, .. } => &metadata.correlation_id,
        }
    }
    
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Audit events for user actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    /// User action
    UserAction {
        metadata: EventMetadata,
        user_id: String,
        action: String,
        resource: String,
        details: HashMap<String, String>,
        success: bool,
    },
    /// Configuration change
    ConfigChanged {
        metadata: EventMetadata,
        user_id: String,
        component: String,
        previous_value: String,
        new_value: String,
    },
    /// Authentication event
    Authentication {
        metadata: EventMetadata,
        user_id: String,
        action: String, // login, logout, failed
        ip_address: String,
        user_agent: String,
    },
}

impl Event for AuditEvent {
    fn event_type(&self) -> &'static str {
        match self {
            AuditEvent::UserAction { .. } => "Audit.UserAction",
            AuditEvent::ConfigChanged { .. } => "Audit.ConfigChanged",
            AuditEvent::Authentication { .. } => "Audit.Authentication",
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            AuditEvent::UserAction { metadata, .. } => metadata.timestamp,
            AuditEvent::ConfigChanged { metadata, .. } => metadata.timestamp,
            AuditEvent::Authentication { metadata, .. } => metadata.timestamp,
        }
    }
    
    fn correlation_id(&self) -> &str {
        match self {
            AuditEvent::UserAction { metadata, .. } => &metadata.correlation_id,
            AuditEvent::ConfigChanged { metadata, .. } => &metadata.correlation_id,
            AuditEvent::Authentication { metadata, .. } => &metadata.correlation_id,
        }
    }
    
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl Default for AlertSeverity {
    fn default() -> Self {
        AlertSeverity::Info
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Union of all domain events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    Cluster(ClusterEvent),
    Pod(PodEvent),
    Alert(AlertEvent),
    Security(SecurityEvent),
    Audit(AuditEvent),
}

impl Event for DomainEvent {
    fn event_type(&self) -> &'static str {
        match self {
            DomainEvent::Cluster(e) => e.event_type(),
            DomainEvent::Pod(e) => e.event_type(),
            DomainEvent::Alert(e) => e.event_type(),
            DomainEvent::Security(e) => e.event_type(),
            DomainEvent::Audit(e) => e.event_type(),
        }
    }
    
    fn timestamp(&self) -> DateTime<Utc> {
        match self {
            DomainEvent::Cluster(e) => e.timestamp(),
            DomainEvent::Pod(e) => e.timestamp(),
            DomainEvent::Alert(e) => e.timestamp(),
            DomainEvent::Security(e) => e.timestamp(),
            DomainEvent::Audit(e) => e.timestamp(),
        }
    }
    
    fn correlation_id(&self) -> &str {
        match self {
            DomainEvent::Cluster(e) => e.correlation_id(),
            DomainEvent::Pod(e) => e.correlation_id(),
            DomainEvent::Alert(e) => e.correlation_id(),
            DomainEvent::Security(e) => e.correlation_id(),
            DomainEvent::Audit(e) => e.correlation_id(),
        }
    }
    
    fn to_json(&self) -> Result<String> {
        match self {
            DomainEvent::Cluster(e) => e.to_json(),
            DomainEvent::Pod(e) => e.to_json(),
            DomainEvent::Alert(e) => e.to_json(),
            DomainEvent::Security(e) => e.to_json(),
            DomainEvent::Audit(e) => e.to_json(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_metadata_creation() {
        let meta = EventMetadata::new("test");
        assert!(!meta.event_id.is_empty());
        assert!(!meta.correlation_id.is_empty());
        assert_eq!(meta.source, "test");
    }

    #[test]
    fn test_cluster_event_serialization() {
        let event = ClusterEvent::StateChanged {
            metadata: EventMetadata::new("test"),
            previous_state: "Healthy".to_string(),
            new_state: "Degraded".to_string(),
            reason: "High CPU".to_string(),
        };
        
        let json = event.to_json().unwrap();
        assert!(json.contains("StateChanged"));
        assert!(json.contains("Degraded"));
    }

    #[test]
    fn test_pod_event_types() {
        let created = PodEvent::Created {
            metadata: EventMetadata::new("test"),
            pod_name: "test-pod".to_string(),
            namespace: "default".to_string(),
            node_name: "node-1".to_string(),
            labels: HashMap::new(),
        };
        
        assert_eq!(created.event_type(), "Pod.Created");
    }

    #[test]
    fn test_alert_severity_display() {
        assert_eq!(format!("{}", AlertSeverity::Info), "INFO");
        assert_eq!(format!("{}", AlertSeverity::Warning), "WARNING");
        assert_eq!(format!("{}", AlertSeverity::Critical), "CRITICAL");
    }

    #[test]
    fn test_domain_event_wrapper() {
        let cluster_event = ClusterEvent::NodeAdded {
            metadata: EventMetadata::new("test"),
            node_name: "node-1".to_string(),
            node_role: "worker".to_string(),
        };
        
        let domain_event = DomainEvent::Cluster(cluster_event);
        
        assert!(domain_event.event_type().contains("NodeAdded"));
        assert!(!domain_event.correlation_id().is_empty());
    }
}
