//! OpenObserve Telemetry Module
//! Sends APM metrics and logs to OpenObserve for performance monitoring
//!
//! Configuration via environment variables or Kubernetes secrets:
//! - OPENOBSERVE_ENDPOINT: URL de l'API OpenObserve
//! - OPENOBSERVE_AUTH: Token d'authentification (base64)
//! - OPENOBSERVE_SECRET_NAME: Nom du secret K8s (default: openobserve-credentials)
//! - OPENOBSERVE_SECRET_NAMESPACE: Namespace du secret (default: kusanagi)
//! - APM_SAMPLE_RATE: Taux d'échantillonnage (default: 1.0)

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{info, warn, error};
use kube::{Client, Api};
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::ByteString;

// ============================================================================
// Configuration
// ============================================================================

lazy_static::lazy_static! {
    static ref TELEMETRY_CONFIG: Mutex<TelemetryConfig> = Mutex::new(TelemetryConfig::default());
    static ref EVENT_QUEUE: Mutex<Vec<TelemetryEvent>> = Mutex::new(Vec::new());
}

static TELEMETRY_ENABLED: AtomicBool = AtomicBool::new(true);
static TELEMETRY_INITIALIZED: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
pub struct TelemetryConfig {
    pub endpoint: String,
    pub auth_token: Option<String>,
    pub batch_size: usize,
    pub sample_rate: f64,
    pub timeout_secs: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            auth_token: None,
            batch_size: 10,
            sample_rate: std::env::var("APM_SAMPLE_RATE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            timeout_secs: std::env::var("APM_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        }
    }
}

/// Initialize telemetry configuration from environment or K8s secrets
pub async fn init_telemetry(client: &Client) {
    if TELEMETRY_INITIALIZED.swap(true, Ordering::SeqCst) {
        return; // Already initialized
    }

    let mut config = TelemetryConfig::default();
    
    // Try environment variables first
    config.endpoint = std::env::var("OPENOBSERVE_ENDPOINT").unwrap_or_default();
    config.auth_token = std::env::var("OPENOBSERVE_AUTH").ok();
    
    // If not configured via env, try K8s secrets
    if config.endpoint.is_empty() || config.auth_token.is_none() {
        let secret_name = std::env::var("OPENOBSERVE_SECRET_NAME")
            .unwrap_or_else(|_| "openobserve-credentials".to_string());
        let secret_namespace = std::env::var("OPENOBSERVE_SECRET_NAMESPACE")
            .unwrap_or_else(|_| "kusanagi".to_string());
        
        match load_credentials_from_secret(client, &secret_namespace, &secret_name).await {
            Ok((endpoint, token)) => {
                if config.endpoint.is_empty() {
                    config.endpoint = endpoint;
                }
                if config.auth_token.is_none() {
                    config.auth_token = Some(token);
                }
                info!("✅ OpenObserve credentials loaded from K8s secret '{}/{}'", 
                    secret_namespace, secret_name);
            }
            Err(e) => {
                warn!("⚠️ Failed to load OpenObserve credentials from secret: {}", e);
                warn!("⚠️ Telemetry will be disabled. Set OPENOBSERVE_ENDPOINT and OPENOBSERVE_AUTH env vars or create the secret.");
                TELEMETRY_ENABLED.store(false, Ordering::Relaxed);
            }
        }
    }
    
    // Validate configuration
    if config.endpoint.is_empty() {
        warn!("⚠️ OPENOBSERVE_ENDPOINT not set, telemetry will be disabled");
        TELEMETRY_ENABLED.store(false, Ordering::Relaxed);
    } else {
        info!("✅ OpenObserve telemetry configured: endpoint={}, sample_rate={}",
            config.endpoint, config.sample_rate);
    }
    
    if config.auth_token.is_none() {
        warn!("⚠️ OpenObserve auth token not configured");
    }
    
    *TELEMETRY_CONFIG.lock().unwrap() = config;
}

/// Load credentials from Kubernetes secret
async fn load_credentials_from_secret(
    client: &Client,
    namespace: &str,
    secret_name: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
    let secret = secrets.get(secret_name).await?;
    
    let data = secret.data.ok_or("Secret has no data")?;
    
    // Extract endpoint
    let endpoint = get_secret_value(&data, "endpoint")
        .or_else(|| get_secret_value(&data, "OPENOBSERVE_ENDPOINT"))
        .or_else(|| get_secret_value(&data, "url"))
        .ok_or("No endpoint found in secret (checked: endpoint, OPENOBSERVE_ENDPOINT, url)")?;
    
    // Extract token
    let token = get_secret_value(&data, "token")
        .or_else(|| get_secret_value(&data, "auth"))
        .or_else(|| get_secret_value(&data, "OPENOBSERVE_AUTH"))
        .or_else(|| get_secret_value(&data, "api-key"))
        .ok_or("No auth token found in secret (checked: token, auth, OPENOBSERVE_AUTH, api-key)")?;
    
    Ok((endpoint, token))
}

fn get_secret_value(data: &std::collections::BTreeMap<String, ByteString>, key: &str) -> Option<String> {
    data.get(key).and_then(|bs| {
        String::from_utf8(bs.0.clone()).ok()
    })
}

/// Re-initialize telemetry (useful when configuration changes)
pub async fn reinit_telemetry(client: &Client) {
    TELEMETRY_INITIALIZED.store(false, Ordering::SeqCst);
    init_telemetry(client).await;
}

/// Check if telemetry is enabled and properly configured
pub fn is_enabled() -> bool {
    TELEMETRY_ENABLED.load(Ordering::Relaxed)
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

    pub fn with_items_count(mut self, count: u64) -> Self {
        self.items_count = Some(count);
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
            warn!("⏱️ APM: No auth token configured, skipping OpenObserve send. \
                   Set OPENOBSERVE_AUTH or create secret '{}/{}'",
                  std::env::var("OPENOBSERVE_SECRET_NAMESPACE").unwrap_or_else(|_| "kusanagi".to_string()),
                  std::env::var("OPENOBSERVE_SECRET_NAME").unwrap_or_else(|_| "openobserve-credentials".to_string()));
            return;
        }
    };

    if config.endpoint.is_empty() {
        warn!("⏱️ APM: No endpoint configured, skipping OpenObserve send");
        return;
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_secs))
        .build()
        .unwrap_or_default();
    
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

// ============================================================================
// Convenience Functions
// ============================================================================

/// Start a new span timer
pub fn start_span(name: &str) -> SpanTimer {
    SpanTimer::new(name)
}

/// Force flush all pending events
pub fn flush() {
    let mut queue = EVENT_QUEUE.lock().unwrap();
    if !queue.is_empty() {
        let events: Vec<_> = queue.drain(..).collect();
        drop(queue);
        tokio::spawn(async move {
            flush_events(events).await;
        });
    }
}
