//! Event-Driven Architecture - Internal Event Bus
//!
//! This module provides a publish/subscribe event bus for decoupled communication
//! between components. It enables reactive, event-driven architecture patterns.
//!
//! # Features
//!
//! - **Async Event Bus**: Non-blocking event distribution
//! - **Typed Events**: Type-safe event handling
//! - **Multiple Subscribers**: One-to-many event broadcasting
//! - **Domain Events**: Business events for audit trails and reactions
//! - **Event Handlers**: Pluggable handlers for event processing
//!
//! # Example
//!
//! ```rust
//! use crate::event_bus::{EventBus, DomainEvent};
//!
//! let bus = EventBus::new();
//!
//! // Subscribe to events
//! let mut rx = bus.subscribe::<PodEvent>();
//! tokio::spawn(async move {
//!     while let Ok(event) = rx.recv().await {
//!         println!("Received: {:?}", event);
//!     }
//! });
//!
//! // Publish events
//! bus.publish(PodEvent::Created { name: "pod-1".into() }).await;
//! ```

pub mod domain_events;
pub mod handlers;
pub mod integration;

pub use domain_events::*;
pub use integration::*;

use crate::error::Result;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, warn};

/// Trait for events that can be published on the bus
///
/// Events must be thread-safe, serializable, and clonable.
pub trait Event: Clone + Send + Sync + Debug + 'static {
    /// Get the event type name
    fn event_type(&self) -> &'static str;
    
    /// Get the event timestamp
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;
    
    /// Get the event correlation ID (for tracing)
    fn correlation_id(&self) -> &str;
    
    /// Serialize to JSON
    fn to_json(&self) -> Result<String>;
}

/// Wrapper for type-erased events
#[derive(Debug, Clone)]
pub struct EventWrapper {
    pub type_id: TypeId,
    pub type_name: &'static str,
    pub payload: Arc<dyn Any + Send + Sync>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: String,
}

impl EventWrapper {
    /// Create a new event wrapper
    pub fn new<E: Event>(event: E) -> Self {
        let type_name = event.event_type();
        let timestamp = event.timestamp();
        let correlation_id = event.correlation_id().to_string();
        Self {
            type_id: TypeId::of::<E>(),
            type_name,
            payload: Arc::new(event),
            timestamp,
            correlation_id,
        }
    }
    
    /// Downcast to concrete event type
    pub fn downcast<E: Event>(&self) -> Option<E> {
        self.payload.clone()
            .downcast::<E>()
            .ok()
            .map(|arc| (*arc).clone())
    }
}

/// Event bus for publish/subscribe pattern
///
/// Maintains channels for each event type and distributes events
/// to all subscribers.
pub struct EventBus {
    /// Map of type ID to broadcast channel sender
    channels: Arc<RwLock<HashMap<TypeId, broadcast::Sender<EventWrapper>>>>,
    /// Default channel capacity
    capacity: usize,
}

impl EventBus {
    /// Create a new event bus with default capacity
    pub fn new() -> Self {
        Self::with_capacity(100)
    }
    
    /// Create a new event bus with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            capacity,
        }
    }
    
    /// Subscribe to events of a specific type
    ///
    /// Returns a receiver that will receive all events of type E.
    pub async fn subscribe<E: Event>(&self) -> broadcast::Receiver<E> {
        let type_id = TypeId::of::<E>();
        
        // Get or create channel
        let mut channels = self.channels.write().await;
        let sender = channels.entry(type_id).or_insert_with(|| {
            let (tx, _rx) = broadcast::channel(self.capacity);
            tx
        });
        
        // Create a channel for typed events
        let (tx, rx) = broadcast::channel(self.capacity);
        
        // Spawn a task to forward wrapped events to typed receiver
        let mut wrapped_rx = sender.subscribe();
        tokio::spawn(async move {
            while let Ok(wrapper) = wrapped_rx.recv().await {
                if let Some(event) = wrapper.downcast::<E>() {
                    if tx.send(event).is_err() {
                        break; // Receiver dropped
                    }
                }
            }
        });
        
        rx
    }
    
    /// Subscribe to the raw event wrapper
    ///
    /// Useful for logging all events regardless of type.
    pub async fn subscribe_all(&self) -> broadcast::Receiver<EventWrapper> {
        // Create a special channel that receives all events
        let (tx, rx) = broadcast::channel(self.capacity);
        
        // Clone channels to avoid holding lock across await
        let channels = self.channels.read().await.clone();
        
        // Forward all events from all channels
        for (_, sender) in channels {
            let tx = tx.clone();
            let mut rx = sender.subscribe();
            tokio::spawn(async move {
                while let Ok(wrapper) = rx.recv().await {
                    if tx.send(wrapper).is_err() {
                        break;
                    }
                }
            });
        }
        
        rx
    }
    
    /// Publish an event to all subscribers
    pub async fn publish<E: Event>(&self, event: E) -> Result<()> {
        let type_id = TypeId::of::<E>();
        let wrapper = EventWrapper::new(event);
        
        debug!(
            event_type = wrapper.type_name,
            correlation_id = %wrapper.correlation_id,
            "Publishing event"
        );
        
        let channels = self.channels.read().await;
        
        if let Some(sender) = channels.get(&type_id) {
            match sender.send(wrapper) {
                Ok(count) => {
                    debug!(subscribers = count, "Event published successfully");
                    Ok(())
                }
                Err(_) => {
                    warn!("No subscribers for event type");
                    Ok(())
                }
            }
        } else {
            debug!("No channel for event type, creating on first publish");
            drop(channels);
            
            let mut channels = self.channels.write().await;
            let (tx, _rx) = broadcast::channel(self.capacity);
            channels.insert(type_id, tx.clone());
            
            match tx.send(wrapper) {
                Ok(_) => Ok(()),
                Err(_) => Ok(()),
            }
        }
    }
    
    /// Get subscriber count for an event type
    pub async fn subscriber_count<E: Event>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        let channels = self.channels.read().await;
        
        channels
            .get(&type_id)
            .map(|sender| sender.receiver_count())
            .unwrap_or(0)
    }
    
    /// Get total event types being tracked
    pub async fn event_type_count(&self) -> usize {
        self.channels.read().await.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            channels: Arc::clone(&self.channels),
            capacity: self.capacity,
        }
    }
}

/// Global event bus instance
use std::sync::OnceLock;

static GLOBAL_EVENT_BUS: OnceLock<EventBus> = OnceLock::new();

/// Initialize the global event bus
pub fn init_global_bus() -> &'static EventBus {
    GLOBAL_EVENT_BUS.get_or_init(EventBus::new)
}

/// Get the global event bus
///
/// Panics if `init_global_bus()` hasn't been called.
pub fn global_bus() -> &'static EventBus {
    GLOBAL_EVENT_BUS.get().expect("EventBus not initialized. Call init_global_bus() first.")
}

/// Event handler trait
///
/// Implement this trait to create pluggable event handlers.
#[async_trait::async_trait]
pub trait EventHandler<E: Event>: Send + Sync {
    /// Handle an event
    async fn handle(&self, event: E);
    
    /// Check if this handler should process the event
    fn can_handle(&self, _event: &E) -> bool {
        true
    }
}

/// Event processor that runs handlers
pub struct EventProcessor {
    bus: EventBus,
}

impl EventProcessor {
    /// Create a new event processor
    pub fn new(bus: EventBus) -> Self {
        Self { bus }
    }
    
    /// Register a handler for an event type
    pub async fn register_handler<E, H>(&self, handler: H)
    where
        E: Event,
        H: EventHandler<E> + 'static,
    {
        let bus = self.bus.clone();
        
        tokio::spawn(async move {
            let mut rx = bus.subscribe::<E>().await;
            
            while let Ok(event) = rx.recv().await {
                if handler.can_handle(&event) {
                    handler.handle(event).await;
                }
            }
        });
    }
}

/// Event metrics for monitoring
#[derive(Debug, Default)]
pub struct EventMetrics {
    pub events_published: u64,
    pub events_dropped: u64,
    pub active_subscriptions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use serde::{Deserialize, Serialize};

    // Test event
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        id: u32,
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        correlation_id: String,
    }

    impl Event for TestEvent {
        fn event_type(&self) -> &'static str {
            "TestEvent"
        }
        
        fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
            self.timestamp
        }
        
        fn correlation_id(&self) -> &str {
            &self.correlation_id
        }
        
        fn to_json(&self) -> Result<String> {
            Ok(serde_json::to_string(self).unwrap())
        }
    }

    impl TestEvent {
        fn new(id: u32, message: impl Into<String>) -> Self {
            Self {
                id,
                message: message.into(),
                timestamp: Utc::now(),
                correlation_id: format!("corr-{}", id),
            }
        }
    }

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe::<TestEvent>().await;
        
        let event = TestEvent::new(1, "Hello");
        bus.publish(event.clone()).await.unwrap();
        
        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, 1);
        assert_eq!(received.message, "Hello");
    }

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe::<TestEvent>().await;
        let mut rx2 = bus.subscribe::<TestEvent>().await;
        
        let event = TestEvent::new(1, "Broadcast");
        bus.publish(event).await.unwrap();
        
        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();
        
        assert_eq!(received1.id, received2.id);
    }

    #[tokio::test]
    async fn test_event_bus_subscriber_count() {
        let bus = EventBus::new();
        
        assert_eq!(bus.subscriber_count::<TestEvent>().await, 0);
        
        let _rx = bus.subscribe::<TestEvent>().await;
        assert_eq!(bus.subscriber_count::<TestEvent>().await, 1);
        
        let _rx2 = bus.subscribe::<TestEvent>().await;
        assert_eq!(bus.subscriber_count::<TestEvent>().await, 2);
    }

    #[tokio::test]
    async fn test_event_wrapper_downcast() {
        let event = TestEvent::new(1, "Test");
        let wrapper = EventWrapper::new(event.clone());
        
        assert_eq!(wrapper.type_name, "TestEvent");
        
        let downcasted = wrapper.downcast::<TestEvent>().unwrap();
        assert_eq!(downcasted.id, event.id);
    }

    #[tokio::test]
    async fn test_event_bus_clone() {
        let bus1 = EventBus::new();
        let bus2 = bus1.clone();
        
        let mut rx = bus2.subscribe::<TestEvent>().await;
        
        let event = TestEvent::new(1, "Cloned");
        bus1.publish(event).await.unwrap();
        
        let received = rx.recv().await.unwrap();
        assert_eq!(received.message, "Cloned");
    }

    #[tokio::test]
    async fn test_global_bus() {
        let bus = init_global_bus();
        let mut rx = bus.subscribe::<TestEvent>().await;
        
        let event = TestEvent::new(1, "Global");
        bus.publish(event).await.unwrap();
        
        let received = rx.recv().await.unwrap();
        assert_eq!(received.message, "Global");
    }
}
