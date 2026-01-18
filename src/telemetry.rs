//! OpenObserve Telemetry Module
//! Sends APM metrics and logs to OpenObserve for performance monitoring

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};

// ============================================================================
// Configuration
// ============================================================================

lazy_static::lazy_static! {
    static ref TELEMETRY_CONFIG: Mutex<TelemetryConfig> = Mutex::new(TelemetryConfig::default());
    static ref EVENT_QUEUE: Mutex<Vec<TelemetryEvent>> = Mutex::new(Vec::new());
}

static TELEMETRY_ENABLED: AtomicBool = AtomicBool::new(true);

#[derive(Clone)]
pub struct TelemetryConfig {
    pub endpoint: String,
    pub auth_token: Option<String>,
    pub batch_size: usize,
    pub flush_interval_secs: u64,
    pub sample_rate: f64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("OPENOBSERVE_ENDPOINT")
                .unwrap_or_else(|_| "https://o2-openobserve.p.zacharie.org/api/default/v1/logs".to_string()),
            auth_token: std::env::var("OPENOBSERVE_AUTH").ok(),
            batch_size: 10,
            flush_interval_secs: 5,
            sample_rate: std::env::var("APM_SAMPLE_RATE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
        }
    }
}

// ============================================================================
// Telemetry Events
// ============================================================================

#[derive(Serialize, Clone, Debug)]
pub struct TelemetryEvent {
    pub timestamp: String,
    pub service: String,
    pub version: String,
    pub event_type: String,
    pub span_name: String,
    pub duration_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_count: Option<u64>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl TelemetryEvent {
    pub fn new(span_name: &str, duration: Duration) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            service: "kusanagi".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            event_type: "apm".to_string(),
            span_name: span_name.to_string(),
            duration_ms: duration.as_secs_f64() * 1000.0,
            namespace: None,
            endpoint: None,
            status: None,
            error: None,
            items_count: None,
            extra: std::collections::HashMap::new(),
        }
    }

    pub fn with_namespace(mut self, ns: Option<&str>) -> Self {
        self.namespace = ns.map(String::from);
        self
    }

    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = Some(endpoint.to_string());
        self
    }

    pub fn with_status(mut self, status: &str) -> Self {
        self.status = Some(status.to_string());
        self
    }

    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }

    pub fn with_items_count(mut self, count: u64) -> Self {
        self.items_count = Some(count);
        self
    }

    pub fn with_extra<V: Serialize>(mut self, key: &str, value: V) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.extra.insert(key.to_string(), v);
        }
        self
    }
}

// ============================================================================
// Span Timer (RAII-style timing)
// ============================================================================

/// RAII-style span timer that automatically records duration on drop
pub struct SpanTimer {
    span_name: String,
    start: Instant,
    namespace: Option<String>,
    endpoint: Option<String>,
    recorded: bool,
}

impl SpanTimer {
    pub fn new(span_name: &str) -> Self {
        info!(span = span_name, "⏱️ APM: Starting span");
        Self {
            span_name: span_name.to_string(),
            start: Instant::now(),
            namespace: None,
            endpoint: None,
            recorded: false,
        }
    }

    pub fn with_namespace(mut self, ns: Option<&str>) -> Self {
        self.namespace = ns.map(String::from);
        self
    }

    pub fn with_endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = Some(endpoint.to_string());
        self
    }

    /// Manually record the span (prevents automatic recording on drop)
    pub fn record(mut self, status: &str, items_count: Option<u64>) {
        self.recorded = true;
        let duration = self.start.elapsed();
        
        let mut event = TelemetryEvent::new(&self.span_name, duration)
            .with_status(status);
        
        if let Some(ref ns) = self.namespace {
            event = event.with_namespace(Some(ns));
        }
        if let Some(ref ep) = self.endpoint {
            event = event.with_endpoint(ep);
        }
        if let Some(count) = items_count {
            event = event.with_items_count(count);
        }

        info!(
            span = %self.span_name,
            duration_ms = duration.as_millis(),
            status = status,
            items_count = ?items_count,
            "⏱️ APM: Span completed"
        );

        queue_event(event);
    }

    /// Record an error
    pub fn record_error(mut self, error: &str) {
        self.recorded = true;
        let duration = self.start.elapsed();
        
        let mut event = TelemetryEvent::new(&self.span_name, duration)
            .with_status("error")
            .with_error(error);
        
        if let Some(ref ns) = self.namespace {
            event = event.with_namespace(Some(ns));
        }
        if let Some(ref ep) = self.endpoint {
            event = event.with_endpoint(ep);
        }

        error!(
            span = %self.span_name,
            duration_ms = duration.as_millis(),
            error = error,
            "⏱️ APM: Span failed"
        );

        queue_event(event);
    }
}

impl Drop for SpanTimer {
    fn drop(&mut self) {
        if !self.recorded {
            let duration = self.start.elapsed();
            let mut event = TelemetryEvent::new(&self.span_name, duration)
                .with_status("completed");
            
            if let Some(ref ns) = self.namespace {
                event = event.with_namespace(Some(ns));
            }
            if let Some(ref ep) = self.endpoint {
                event = event.with_endpoint(ep);
            }

            info!(
                span = %self.span_name,
                duration_ms = duration.as_millis(),
                "⏱️ APM: Span auto-completed"
            );

            queue_event(event);
        }
    }
}

// ============================================================================
// Event Queue & Flushing
// ============================================================================

fn queue_event(event: TelemetryEvent) {
    if !TELEMETRY_ENABLED.load(Ordering::Relaxed) {
        return;
    }

    // Sample rate check
    let config = TELEMETRY_CONFIG.lock().unwrap();
    if config.sample_rate < 1.0 && rand::random::<f64>() > config.sample_rate {
        return;
    }
    drop(config);

    let mut queue = EVENT_QUEUE.lock().unwrap();
    queue.push(event);
    
    let batch_size = TELEMETRY_CONFIG.lock().unwrap().batch_size;
    if queue.len() >= batch_size {
        let events: Vec<_> = queue.drain(..).collect();
        drop(queue);
        tokio::spawn(async move {
            flush_events(events).await;
        });
    }
}

async fn flush_events(events: Vec<TelemetryEvent>) {
    if events.is_empty() {
        return;
    }

    let config = TELEMETRY_CONFIG.lock().unwrap().clone();
    
    let auth_token = match config.auth_token {
        Some(token) => token,
        None => {
            warn!("⏱️ APM: No auth token configured, skipping OpenObserve send");
            return;
        }
    };

    let client = reqwest::Client::new();
    
    match client
        .post(&config.endpoint)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Basic {}", auth_token))
        .json(&events)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                info!(count = events.len(), "⏱️ APM: Sent {} events to OpenObserve", events.len());
            } else {
                warn!(
                    status = %response.status(),
                    "⏱️ APM: OpenObserve returned error status"
                );
            }
        }
        Err(e) => {
            error!(error = %e, "⏱️ APM: Failed to send events to OpenObserve");
        }
    }
}

/// Force flush all queued events
pub async fn force_flush() {
    let events: Vec<_> = {
        let mut queue = EVENT_QUEUE.lock().unwrap();
        queue.drain(..).collect()
    };
    
    if !events.is_empty() {
        flush_events(events).await;
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Start a new span timer
pub fn start_span(name: &str) -> SpanTimer {
    SpanTimer::new(name)
}

/// Send a standalone metric event
pub async fn send_metric(name: &str, value: f64, tags: &[(&str, &str)]) {
    let mut event = TelemetryEvent {
        timestamp: chrono::Utc::now().to_rfc3339(),
        service: "kusanagi".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        event_type: "metric".to_string(),
        span_name: name.to_string(),
        duration_ms: value,
        namespace: None,
        endpoint: None,
        status: None,
        error: None,
        items_count: None,
        extra: std::collections::HashMap::new(),
    };

    for (key, val) in tags {
        event.extra.insert(key.to_string(), serde_json::Value::String(val.to_string()));
    }

    queue_event(event);
}

/// Check if telemetry is enabled
pub fn is_enabled() -> bool {
    TELEMETRY_ENABLED.load(Ordering::Relaxed)
}

/// Enable/disable telemetry
pub fn set_enabled(enabled: bool) {
    TELEMETRY_ENABLED.store(enabled, Ordering::Relaxed);
    info!(enabled = enabled, "⏱️ APM: Telemetry status changed");
}
