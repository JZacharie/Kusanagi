//! Custom Prometheus Metrics
//!
//! Business metrics for monitoring Kusanagi operations:
//! - Pod operations (scales, deletions, restarts)
//! - API requests by endpoint
//! - Error rates
//! - Cache hit/miss rates
//! - LLM API usage

use prometheus::{Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, Registry, opts, labels};
use std::sync::OnceLock;
use tracing::{error, info};

/// Global metrics registry
static METRICS: OnceLock<KusanagiMetrics> = OnceLock::new();

/// Custom metrics for Kusanagi
pub struct KusanagiMetrics {
    /// Pod operations counter
    pub pod_operations: CounterVec,
    /// API requests counter
    pub api_requests: CounterVec,
    /// API request duration
    pub request_duration: HistogramVec,
    /// Active errors gauge
    pub active_errors: Gauge,
    /// Cache operations
    pub cache_operations: CounterVec,
    /// LLM requests
    pub llm_requests: CounterVec,
    /// LLM request duration
    pub llm_duration: HistogramVec,
    /// Notifications sent
    pub notifications_sent: CounterVec,
    /// WebSocket connections
    pub websocket_connections: Gauge,
    /// Background jobs
    pub background_jobs: CounterVec,
}

impl KusanagiMetrics {
    fn new() -> Self {
        let pod_operations = CounterVec::new(
            opts!("kusanagi_pod_operations_total", "Total pod operations"),
            &["operation", "namespace", "status"]
        ).expect("metric can be created");

        let api_requests = CounterVec::new(
            opts!("kusanagi_api_requests_total", "Total API requests"),
            &["method", "endpoint", "status"]
        ).expect("metric can be created");

        let request_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "kusanagi_request_duration_seconds",
                "API request duration in seconds"
            ).buckets(vec![0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
            &["method", "endpoint"]
        ).expect("metric can be created");

        let active_errors = Gauge::new(
            "kusanagi_active_errors",
            "Number of active errors in the cluster"
        ).expect("metric can be created");

        let cache_operations = CounterVec::new(
            opts!("kusanagi_cache_operations_total", "Cache operations"),
            &["cache_type", "operation", "result"]
        ).expect("metric can be created");

        let llm_requests = CounterVec::new(
            opts!("kusanagi_llm_requests_total", "LLM API requests"),
            &["provider", "model", "status"]
        ).expect("metric can be created");

        let llm_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "kusanagi_llm_duration_seconds",
                "LLM request duration"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]),
            &["provider", "model"]
        ).expect("metric can be created");

        let notifications_sent = CounterVec::new(
            opts!("kusanagi_notifications_sent_total", "Notifications sent"),
            &["channel", "status"]
        ).expect("metric can be created");

        let websocket_connections = Gauge::new(
            "kusanagi_websocket_active_connections",
            "Active WebSocket connections"
        ).expect("metric can be created");

        let background_jobs = CounterVec::new(
            opts!("kusanagi_background_jobs_total", "Background jobs executed"),
            &["job_type", "status"]
        ).expect("metric can be created");

        Self {
            pod_operations,
            api_requests,
            request_duration,
            active_errors,
            cache_operations,
            llm_requests,
            llm_duration,
            notifications_sent,
            websocket_connections,
            background_jobs,
        }
    }

    /// Register all metrics with the global registry
    pub fn register(&self, registry: &Registry) {
        registry.register(Box::new(self.pod_operations.clone())).ok();
        registry.register(Box::new(self.api_requests.clone())).ok();
        registry.register(Box::new(self.request_duration.clone())).ok();
        registry.register(Box::new(self.active_errors.clone())).ok();
        registry.register(Box::new(self.cache_operations.clone())).ok();
        registry.register(Box::new(self.llm_requests.clone())).ok();
        registry.register(Box::new(self.llm_duration.clone())).ok();
        registry.register(Box::new(self.notifications_sent.clone())).ok();
        registry.register(Box::new(self.websocket_connections.clone())).ok();
        registry.register(Box::new(self.background_jobs.clone())).ok();
    }
}

/// Initialize global metrics
pub fn init() -> &'static KusanagiMetrics {
    METRICS.get_or_init(|| {
        info!("Initializing custom metrics");
        KusanagiMetrics::new()
    })
}

/// Get metrics instance
pub fn get() -> &'static KusanagiMetrics {
    METRICS.get().expect("Metrics not initialized")
}

// ============== Helper Functions ==============

/// Record a pod operation
pub fn record_pod_operation(operation: &str, namespace: &str, success: bool) {
    let status = if success { "success" } else { "error" };
    get().pod_operations.with_label_values(&[operation, namespace, status]).inc();
}

/// Record API request
pub fn record_api_request(method: &str, endpoint: &str, status_code: u16) {
    let status = format!("{}", status_code);
    get().api_requests.with_label_values(&[method, endpoint, &status]).inc();
}

/// Record request duration
pub fn record_request_duration(method: &str, endpoint: &str, duration_secs: f64) {
    get().request_duration.with_label_values(&[method, endpoint]).observe(duration_secs);
}

/// Update active errors count
pub fn set_active_errors(count: i64) {
    get().active_errors.set(count as f64);
}

/// Record cache operation
pub fn record_cache_operation(cache_type: &str, operation: &str, hit: bool) {
    let result = if hit { "hit" } else { "miss" };
    get().cache_operations.with_label_values(&[cache_type, operation, result]).inc();
}

/// Record LLM request
pub fn record_llm_request(provider: &str, model: &str, success: bool, duration_secs: f64) {
    let status = if success { "success" } else { "error" };
    get().llm_requests.with_label_values(&[provider, model, status]).inc();
    get().llm_duration.with_label_values(&[provider, model]).observe(duration_secs);
}

/// Record notification
pub fn record_notification(channel: &str, success: bool) {
    let status = if success { "success" } else { "error" };
    get().notifications_sent.with_label_values(&[channel, status]).inc();
}

/// Update WebSocket connections
pub fn set_websocket_connections(count: i64) {
    get().websocket_connections.set(count as f64);
}

/// Record background job
pub fn record_background_job(job_type: &str, success: bool) {
    let status = if success { "success" } else { "error" };
    get().background_jobs.with_label_values(&[job_type, status]).inc();
}

// ============== Metrics Handler ==============

use actix_web::{get, HttpResponse, Responder};

/// Metrics endpoint for Prometheus scraping
#[get("/metrics")]
pub async fn metrics_handler() -> impl Responder {
    use prometheus::Encoder;
    
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    
    match encoder.encode_to_string(&metric_families) {
        Ok(metrics) => HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(metrics),
        Err(e) => {
            error!("Failed to encode metrics: {}", e);
            HttpResponse::InternalServerError().body("Failed to encode metrics")
        }
    }
}
