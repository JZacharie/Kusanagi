//! Event Bus Usage Examples
//!
//! This module demonstrates how to use the event bus for:
//! 1. Publishing events from business logic
//! 2. Initializing the event bus at application startup
//! 3. Reacting to events in different parts of the system

/// # Example 1: Initialize Event Bus at Application Startup
///
/// Add this to your main.rs or startup code:
///
/// ```rust,no_run
/// use kusanagi::event_bus::integration::init_global_integration;
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     // Initialize event bus integration
///     let event_bus = init_global_integration().await;
///     
///     // Start HTTP server
///     HttpServer::new(move || {
///         App::new()
///             .app_data(web::Data::new(AppState { 
///                 client: kube_client.clone(),
///             }))
///             // ... routes
///     })
///     .bind("0.0.0.0:8080")?
///     .run()
///     .await
/// }
/// ```

/// # Example 2: Publish Pod Events from Kubernetes Watcher
///
/// When detecting pod changes, emit events:
///
/// ```rust,no_run
/// use kusanagi::event_bus::{
///     integration::global_integration,
///     PodEvent, EventMetadata,
/// };
///
/// async fn on_pod_created(pod: &Pod) {
///     if let Some(integration) = global_integration() {
///         let node_name = pod.spec.as_ref()
///             .and_then(|s| s.node_name.clone())
///             .unwrap_or_default();
///         
///         let labels = pod.metadata.labels.clone().unwrap_or_default();
///         
///         let event = PodEvent::Created {
///             metadata: EventMetadata::new("k8s-watcher"),
///             pod_name: pod.metadata.name.clone().unwrap_or_default(),
///             namespace: pod.metadata.namespace.clone().unwrap_or_default(),
///             node_name,
///             labels,
///         };
///         
///         if let Err(e) = integration.publish_pod_event(event).await {
///             tracing::error!("Failed to publish pod created event: {}", e);
///         }
///     }
/// }
/// ```

/// # Example 3: Publish Custom Audit Events
///
/// Track user actions for audit trail:
///
/// ```rust,no_run
/// use kusanagi::event_bus::{
///     integration::global_integration,
///     AuditEvent, EventMetadata,
/// };
/// use std::collections::HashMap;
///
/// async fn on_user_action(
///     user_id: &str,
///     action: &str,
///     resource: &str,
///     success: bool,
/// ) {
///     if let Some(integration) = global_integration() {
///         let mut details = HashMap::new();
///         details.insert("ip".to_string(), get_client_ip());
///         details.insert("method".to_string(), "POST".to_string());
///         
///         let event = AuditEvent::UserAction {
///             metadata: EventMetadata::new("api-server"),
///             user_id: user_id.to_string(),
///             action: action.to_string(),
///             resource: resource.to_string(),
///             details,
///             success,
///         };
///         
///         integration.publish_audit_event(event).await.ok();
///     }
/// }
/// ```

/// # Example 4: Subscribe to Events Directly
///
/// For custom processing outside the built-in handlers:
///
/// ```rust,no_run
/// use kusanagi::event_bus::{global_bus, PodEvent};
///
/// async fn start_custom_handler() {
///     let bus = global_bus().expect("Event bus not initialized");
///     let mut rx = bus.subscribe::<PodEvent>().await;
///     
///     tokio::spawn(async move {
///         while let Ok(event) = rx.recv().await {
///             match event {
///                 PodEvent::CrashLoopDetected { pod_name, .. } => {
///                     // Send Slack notification
///                     send_slack_alert(&format!(
///                         "🚨 Crash loop detected on pod: {}", 
///                         pod_name
///                     )).await;
///                 }
///                 _ => {}
///             }
///         }
///     });
/// }
/// ```

/// # Example 5: WebSocket Client Integration
///
/// JavaScript client receiving pod events:
///
/// ```javascript
/// const ws = new WebSocket('ws://localhost:8080/ws/notifications');
///
/// ws.onmessage = (event) => {
///     const msg = JSON.parse(event.data);
///     
///     switch(msg.type) {
///         case 'pod_event':
///             handlePodEvent(msg);
///             break;
///         case 'alert':
///             showAlert(msg);
///             break;
///         case 'stats_update':
///             updateStats(msg);
///             break;
///     }
/// };
///
/// function handlePodEvent(msg) {
///     switch(msg.event_type) {
///         case 'pod_created':
///             console.log(`✅ Pod ${msg.pod_name} created in ${msg.namespace}`);
///             break;
///         case 'pod_crash_loop':
///             console.error(`🚨 Crash loop on ${msg.pod_name}! Restarts: ${msg.restart_count}`);
///             break;
///         case 'pod_status_changed':
///             console.log(`🔄 ${msg.pod_name}: ${msg.previous_status} → ${msg.new_status}`);
///             break;
///     }
/// }
/// ```

/// # Example 6: Correlation ID for Distributed Tracing
///
/// Propagate correlation IDs across operations:
///
/// ```rust,no_run
/// use kusanagi::event_bus::{EventMetadata, PodEvent};
///
/// async fn scale_deployment_with_events(
///     namespace: &str,
///     name: &str,
///     replicas: i32,
/// ) {
///     // Generate correlation ID for this operation
///     let correlation_id = format!("scale-{}", uuid::Uuid::new_v4());
///     
///     // Publish audit event
///     let audit_event = AuditEvent::UserAction {
///         metadata: EventMetadata::with_correlation(
///             &correlation_id, 
///             "api-server"
///         ),
///         user_id: current_user().to_string(),
///         action: "scale_deployment".to_string(),
///         resource: format!("{}/{}", namespace, name),
///         details: [("replicas".to_string(), replicas.to_string())].into(),
///         success: false, // Will be updated
///     };
///     
///     // Perform operation
///     match do_scale(namespace, name, replicas).await {
///         Ok(_) => {
///             // Publish success event with same correlation ID
///             // This allows tracing the full operation chain
///         }
///         Err(e) => {
///             // Publish failure event with same correlation ID
///         }
///     }
/// }
/// ```

/// # Example 7: Custom Event Handler
///
/// Implement the EventHandler trait for custom processing:
///
/// ```rust,no_run
/// use kusanagi::event_bus::{
///     EventHandler, ClusterEvent,
/// };
/// use async_trait::async_trait;
///
/// struct MetricsExporter;
///
/// #[async_trait]
/// impl EventHandler<ClusterEvent> for MetricsExporter {
///     async fn handle(&self, event: ClusterEvent) {
///         match event {
///             ClusterEvent::NodeNotReady { node_name, .. } => {
///                 metrics::counter!("cluster_node_not_ready", 1);
///                 metrics::gauge!("cluster_healthy_nodes", -1.0);
///             }
///             ClusterEvent::NodeAdded { .. } => {
///                 metrics::counter!("cluster_node_added", 1);
///                 metrics::gauge!("cluster_healthy_nodes", 1.0);
///             }
///             _ => {}
///         }
///     }
/// }
/// ```

/// # Example 8: Event Filtering
///
/// Filter events before processing:
///
/// ```rust,no_run
/// use kusanagi::event_bus::{
///     handlers::FilteringHandler,
///     AlertEvent, AlertSeverity,
/// };
///
/// // Only handle critical alerts
/// let critical_only = FilteringHandler::new(
///     |event: &AlertEvent| {
///         matches!(event, 
///             AlertEvent::Fired { severity: AlertSeverity::Critical, .. }
///         )
///     },
///     Box::new(PagerDutyHandler::new()),
/// );
/// ```
