//! Event Handlers
//!
//! Built-in handlers for processing domain events:
//! - LoggingHandler: Logs all events
//! - AuditLogger: Records audit events to storage
//! - AlertNotifier: Sends notifications for alerts
//! - CacheInvalidator: Invalidates cache entries on relevant events
//! - WebSocketBroadcaster: Broadcasts events to WebSocket clients

use crate::cache::Cache;
use crate::event_bus::{
    AuditEvent, ClusterEvent, DomainEvent, Event, EventHandler, PodEvent,
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Handler that logs all events
pub struct LoggingHandler;

#[async_trait]
impl EventHandler<DomainEvent> for LoggingHandler {
    async fn handle(&self, event: DomainEvent) {
        info!(
            event_type = %event.event_type(),
            correlation_id = %event.correlation_id(),
            timestamp = %event.timestamp(),
            "Domain event received"
        );
        
        match &event {
            DomainEvent::Cluster(e) => {
                debug!(event = ?e, "Cluster event");
            }
            DomainEvent::Pod(e) => {
                debug!(event = ?e, "Pod event");
            }
            DomainEvent::Alert(e) => {
                warn!(event = ?e, "Alert event");
            }
            DomainEvent::Security(e) => {
                warn!(event = ?e, "Security event");
            }
            DomainEvent::Audit(e) => {
                info!(event = ?e, "Audit event");
            }
        }
    }
}

/// Cache invalidator for pod events
pub type PodCache = Arc<dyn Cache<String, crate::cache::CacheStats> + Send + Sync>;

/// Handler for cluster events
pub struct ClusterEventHandler {
    cluster_cache: Option<Arc<dyn Cache<String, String> + Send + Sync>>,
}

impl ClusterEventHandler {
    /// Create a new cluster event handler
    pub fn new() -> Self {
        Self { cluster_cache: None }
    }
    
    /// Create with cluster cache
    pub fn with_cluster_cache(cache: Arc<dyn Cache<String, String> + Send + Sync>) -> Self {
        Self { cluster_cache: Some(cache) }
    }
}

impl Default for ClusterEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventHandler<ClusterEvent> for ClusterEventHandler {
    async fn handle(&self, event: ClusterEvent) {
        match &event {
            ClusterEvent::NodeAdded { node_name, .. } => {
                info!(node = %node_name, "Node added to cluster");
                // Invalidate cluster cache if available
                if let Some(ref cache) = self.cluster_cache {
                    let _ = cache.remove(&"cluster_overview".to_string()).await;
                }
            }
            ClusterEvent::NodeRemoved { node_name, reason, .. } => {
                warn!(node = %node_name, reason = %reason, "Node removed from cluster");
                if let Some(ref cache) = self.cluster_cache {
                    let _ = cache.clear().await;
                }
            }
            ClusterEvent::NodeNotReady { node_name, condition, .. } => {
                warn!(
                    node = %node_name,
                    condition = %condition,
                    "Node is not ready"
                );
            }
            ClusterEvent::ThresholdCrossed { resource_type, threshold, current_value, severity, .. } => {
                warn!(
                    resource = %resource_type,
                    threshold = %threshold,
                    current = %current_value,
                    severity = ?severity,
                    "Resource threshold crossed"
                );
            }
            ClusterEvent::StateChanged { previous_state, new_state, reason, .. } => {
                info!(
                    from = %previous_state,
                    to = %new_state,
                    reason = %reason,
                    "Cluster state changed"
                );
                if let Some(ref cache) = self.cluster_cache {
                    let _ = cache.remove(&"cluster_overview".to_string()).await;
                }
            }
        }
    }
}

/// Handler for pod events
pub struct PodEventHandler {
    pod_cache: Option<Arc<dyn Cache<String, String> + Send + Sync>>,
}

impl PodEventHandler {
    /// Create a new pod event handler
    pub fn new() -> Self {
        Self { pod_cache: None }
    }
    
    /// Create with pod cache
    pub fn with_pod_cache(cache: Arc<dyn Cache<String, String> + Send + Sync>) -> Self {
        Self { pod_cache: Some(cache) }
    }
    
    /// Invalidate caches related to a specific pod
    async fn invalidate_pod_caches(&self, namespace: &str, pod_name: &str) {
        if let Some(ref cache) = self.pod_cache {
            // Construct cache key
            let cache_key = format!("{}/{}", namespace, pod_name);
            let _ = cache.remove(&cache_key).await;
            debug!(pod = %cache_key, "Pod cache invalidated");
        }
    }
}

impl Default for PodEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventHandler<PodEvent> for PodEventHandler {
    async fn handle(&self, event: PodEvent) {
        match &event {
            PodEvent::Created { pod_name, namespace, node_name, .. } => {
                info!(
                    pod = %pod_name,
                    namespace = %namespace,
                    node = %node_name,
                    "Pod created"
                );
                self.invalidate_pod_caches(namespace, pod_name).await;
            }
            PodEvent::Deleted { pod_name, namespace, reason, .. } => {
                info!(
                    pod = %pod_name,
                    namespace = %namespace,
                    reason = %reason,
                    "Pod deleted"
                );
                self.invalidate_pod_caches(namespace, pod_name).await;
            }
            PodEvent::StatusChanged { pod_name, namespace, previous_status, new_status, .. } => {
                debug!(
                    pod = %pod_name,
                    namespace = %namespace,
                    from = %previous_status,
                    to = %new_status,
                    "Pod status changed"
                );
                self.invalidate_pod_caches(namespace, pod_name).await;
            }
            PodEvent::Restarted { pod_name, namespace, restart_count, reason, .. } => {
                warn!(
                    pod = %pod_name,
                    namespace = %namespace,
                    restarts = %restart_count,
                    reason = %reason,
                    "Pod restarted"
                );
                self.invalidate_pod_caches(namespace, pod_name).await;
            }
            PodEvent::CrashLoopDetected { pod_name, namespace, restart_count, container_name, .. } => {
                warn!(
                    pod = %pod_name,
                    namespace = %namespace,
                    restarts = %restart_count,
                    container = %container_name,
                    "Crash loop detected"
                );
            }
            PodEvent::ImagePullFailed { pod_name, namespace, image, error, .. } => {
                warn!(
                    pod = %pod_name,
                    namespace = %namespace,
                    image = %image,
                    error = %error,
                    "Image pull failed"
                );
            }
        }
    }
}

/// Handler for audit events
pub struct AuditEventHandler;

#[async_trait]
impl EventHandler<AuditEvent> for AuditEventHandler {
    async fn handle(&self, event: AuditEvent) {
        match &event {
            AuditEvent::UserAction { user_id, action, resource, success, .. } => {
                if *success {
                    info!(
                        user = %user_id,
                        action = %action,
                        resource = %resource,
                        "User action successful"
                    );
                } else {
                    warn!(
                        user = %user_id,
                        action = %action,
                        resource = %resource,
                        "User action failed"
                    );
                }
            }
            AuditEvent::ConfigChanged { user_id, component, previous_value, new_value, .. } => {
                info!(
                    user = %user_id,
                    component = %component,
                    old = %previous_value,
                    new = %new_value,
                    "Configuration changed"
                );
            }
            AuditEvent::Authentication { user_id, action, ip_address, .. } => {
                if action == "failed" {
                    warn!(
                        user = %user_id,
                        ip = %ip_address,
                        "Authentication failed"
                    );
                } else {
                    info!(
                        user = %user_id,
                        ip = %ip_address,
                        action = %action,
                        "Authentication event"
                    );
                }
            }
        }
    }
}

/// Composite handler that delegates to multiple handlers
pub struct CompositeHandler<E: Event> {
    handlers: Vec<Box<dyn EventHandler<E>>>,
}

impl<E: Event> CompositeHandler<E> {
    /// Create a new composite handler
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }
    
    /// Add a handler
    pub fn add_handler(&mut self, handler: Box<dyn EventHandler<E>>) {
        self.handlers.push(handler);
    }
}

impl<E: Event> Default for CompositeHandler<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E: Event> EventHandler<E> for CompositeHandler<E> {
    async fn handle(&self, event: E) {
        for handler in &self.handlers {
            handler.handle(event.clone()).await;
        }
    }
}

/// Handler that filters events before processing
pub struct FilteringHandler<E: Event> {
    predicate: Box<dyn Fn(&E) -> bool + Send + Sync>,
    inner: Box<dyn EventHandler<E>>,
}

impl<E: Event> FilteringHandler<E> {
    /// Create a new filtering handler
    pub fn new<F>(predicate: F, inner: Box<dyn EventHandler<E>>) -> Self
    where
        F: Fn(&E) -> bool + Send + Sync + 'static,
    {
        Self {
            predicate: Box::new(predicate),
            inner,
        }
    }
}

#[async_trait]
impl<E: Event> EventHandler<E> for FilteringHandler<E> {
    fn can_handle(&self, event: &E) -> bool {
        (self.predicate)(event)
    }
    
    async fn handle(&self, event: E) {
        self.inner.handle(event).await;
    }
}

/// Instrumented handler that tracks metrics
pub struct InstrumentedHandler<E: Event> {
    inner: Box<dyn EventHandler<E>>,
    metrics: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl<E: Event> InstrumentedHandler<E> {
    /// Create a new instrumented handler
    pub fn new(inner: Box<dyn EventHandler<E>>) -> Self {
        Self {
            inner,
            metrics: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }
    
    /// Get the event count
    pub fn event_count(&self) -> u64 {
        self.metrics.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[async_trait]
impl<E: Event> EventHandler<E> for InstrumentedHandler<E> {
    async fn handle(&self, event: E) {
        self.metrics.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.inner.handle(event).await;
    }
}

/// Builder for creating handlers with common patterns
pub struct HandlerBuilder;

impl HandlerBuilder {
    /// Create a logging handler
    pub fn logging() -> LoggingHandler {
        LoggingHandler
    }
    
    /// Create a cluster event handler
    pub fn cluster() -> ClusterEventHandler {
        ClusterEventHandler::new()
    }
    
    /// Create a cluster handler with cache
    pub fn cluster_with_cache(cache: Arc<dyn Cache<String, String> + Send + Sync>) -> ClusterEventHandler {
        ClusterEventHandler::with_cluster_cache(cache)
    }
    
    /// Create a pod event handler
    pub fn pod() -> PodEventHandler {
        PodEventHandler::new()
    }
    
    /// Create a pod handler with cache
    pub fn pod_with_cache(cache: Arc<dyn Cache<String, String> + Send + Sync>) -> PodEventHandler {
        PodEventHandler::with_pod_cache(cache)
    }
    
    /// Create an audit event handler
    pub fn audit() -> AuditEventHandler {
        AuditEventHandler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_bus::{EventMetadata, PodEvent};
    use std::sync::atomic::{AtomicU64, Ordering};

    #[derive(Clone, Debug)]
    struct TestEvent {
        id: u32,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "Test"
        }
        
        fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
            chrono::Utc::now()
        }
        
        fn correlation_id(&self) -> &str {
            "test"
        }
        
        fn to_json(&self) -> crate::error::Result<String> {
            Ok("{}".to_string())
        }
    }

    struct CountingHandler {
        count: Arc<AtomicU64>,
    }

    #[async_trait]
    impl EventHandler<TestEvent> for CountingHandler {
        async fn handle(&self, _event: TestEvent) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[tokio::test]
    async fn test_composite_handler() {
        let count = Arc::new(AtomicU64::new(0));
        let handler1 = CountingHandler { count: Arc::clone(&count) };
        let handler2 = CountingHandler { count: Arc::clone(&count) };
        
        let mut composite: CompositeHandler<TestEvent> = CompositeHandler::new();
        composite.add_handler(Box::new(handler1));
        composite.add_handler(Box::new(handler2));
        
        composite.handle(TestEvent { id: 1 }).await;
        
        assert_eq!(count.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn test_instrumented_handler() {
        let count = Arc::new(AtomicU64::new(0));
        let inner = CountingHandler { count: Arc::clone(&count) };
        let handler = InstrumentedHandler::new(Box::new(inner));
        
        handler.handle(TestEvent { id: 1 }).await;
        handler.handle(TestEvent { id: 2 }).await;
        
        assert_eq!(handler.event_count(), 2);
    }

    #[test]
    fn test_pod_event_handler_event_type() {
        use std::collections::HashMap;
        let event = PodEvent::Created {
            metadata: EventMetadata::new("test"),
            pod_name: "pod-1".to_string(),
            namespace: "default".to_string(),
            node_name: "node-1".to_string(),
            labels: HashMap::new(),
        };
        
        assert_eq!(event.event_type(), "Pod.Created");
    }

    #[test]
    fn test_handler_builder() {
        let logging = HandlerBuilder::logging();
        let cluster = HandlerBuilder::cluster();
        let pod = HandlerBuilder::pod();
        let audit = HandlerBuilder::audit();
        
        // Just verify they compile and create successfully
        drop(logging);
        drop(cluster);
        drop(pod);
        drop(audit);
    }
}
