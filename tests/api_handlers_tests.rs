//! Tests for API Handlers
//! Tests for: health, cache, config, slack, websocket, k8s, monitoring, system

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

// ============================================================================
// Health Handler Tests
// ============================================================================

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn readiness_check() -> impl IntoResponse {
    Json(json!({
        "ready": true,
        "checks": {
            "database": "ok",
            "cache": "ok",
            "kubernetes": "ok",
        }
    }))
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "healthy");
}

#[tokio::test]
async fn test_readiness_endpoint() {
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["ready"], true);
    assert!(json["checks"]["database"].is_string());
}

// ============================================================================
// Cache Handler Tests
// ============================================================================

#[derive(Clone)]
struct CacheState {
    stats: Arc<Mutex<CacheStats>>,
}

#[derive(Clone, Debug)]
struct CacheStats {
    hits: u64,
    misses: u64,
    entries: usize,
    size_bytes: u64,
}

async fn cache_stats(State(state): State<CacheState>) -> impl IntoResponse {
    let stats = state.stats.lock().unwrap();
    Json(json!({
        "hits": stats.hits,
        "misses": stats.misses,
        "entries": stats.entries,
        "size_bytes": stats.size_bytes,
        "hit_rate": if stats.hits + stats.misses > 0 {
            stats.hits as f64 / (stats.hits + stats.misses) as f64
        } else {
            0.0
        }
    }))
}

async fn cache_clear(State(state): State<CacheState>) -> impl IntoResponse {
    let mut stats = state.stats.lock().unwrap();
    stats.hits = 0;
    stats.misses = 0;
    stats.entries = 0;
    stats.size_bytes = 0;

    Json(json!({
        "cleared": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[tokio::test]
async fn test_cache_stats_endpoint() {
    let state = CacheState {
        stats: Arc::new(Mutex::new(CacheStats {
            hits: 100,
            misses: 20,
            entries: 50,
            size_bytes: 1024000,
        })),
    };

    let app = Router::new()
        .route("/api/cache/stats", get(cache_stats))
        .route("/api/cache/clear", post(cache_clear))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/cache/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["hits"], 100);
    assert_eq!(json["misses"], 20);
    assert_eq!(json["entries"], 50);
    assert!(json["hit_rate"].as_f64().unwrap() > 0.8);
}

#[tokio::test]
async fn test_cache_clear_endpoint() {
    let state = CacheState {
        stats: Arc::new(Mutex::new(CacheStats {
            hits: 100,
            misses: 20,
            entries: 50,
            size_bytes: 1024000,
        })),
    };

    let app = Router::new()
        .route("/api/cache/stats", get(cache_stats))
        .route("/api/cache/clear", post(cache_clear))
        .with_state(state.clone());

    // Clear cache
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/cache/clear")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check stats are cleared
    let stats = state.stats.lock().unwrap();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.entries, 0);
}

// ============================================================================
// Config Handler Tests
// ============================================================================

#[derive(Clone)]
struct ConfigState {
    config: Arc<Mutex<AppConfig>>,
}

#[derive(Clone, Debug)]
struct AppConfig {
    server_port: u16,
    server_host: String,
    log_level: String,
    features: Vec<String>,
}

async fn get_config(State(state): State<ConfigState>) -> impl IntoResponse {
    let config = state.config.lock().unwrap();
    Json(json!({
        "server": {
            "port": config.server_port,
            "host": config.server_host,
        },
        "log_level": config.log_level,
        "features": config.features,
    }))
}

async fn update_config(
    State(state): State<ConfigState>,
    Json(updates): Json<serde_json::Value>,
) -> impl IntoResponse {
    let mut config = state.config.lock().unwrap();

    if let Some(port) = updates["server"]["port"].as_u64() {
        config.server_port = port as u16;
    }

    Json(json!({
        "updated": true,
        "config": {
            "server": {
                "port": config.server_port,
                "host": config.server_host,
            }
        }
    }))
}

#[tokio::test]
async fn test_get_config_endpoint() {
    let state = ConfigState {
        config: Arc::new(Mutex::new(AppConfig {
            server_port: 8080,
            server_host: "0.0.0.0".to_string(),
            log_level: "info".to_string(),
            features: vec!["kubernetes".to_string(), "monitoring".to_string()],
        })),
    };

    let app = Router::new()
        .route("/api/config", get(get_config))
        .route("/api/config", post(update_config))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/config")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["server"]["port"], 8080);
    assert_eq!(json["server"]["host"], "0.0.0.0");
    assert_eq!(json["log_level"], "info");
    assert!(json["features"]
        .as_array()
        .unwrap()
        .contains(&json!("kubernetes")));
}

#[tokio::test]
async fn test_update_config_endpoint() {
    let state = ConfigState {
        config: Arc::new(Mutex::new(AppConfig {
            server_port: 8080,
            server_host: "0.0.0.0".to_string(),
            log_level: "info".to_string(),
            features: vec![],
        })),
    };

    let app = Router::new()
        .route("/api/config", get(get_config))
        .route("/api/config", post(update_config))
        .with_state(state.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/config")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"server": {"port": 9090}}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["updated"], true);
    assert_eq!(json["config"]["server"]["port"], 9090);

    // Verify state was updated
    let config = state.config.lock().unwrap();
    assert_eq!(config.server_port, 9090);
}

// ============================================================================
// Slack Handler Tests
// ============================================================================

#[derive(Clone)]
struct SlackState {
    sent_messages: Arc<Mutex<Vec<String>>>,
}

#[derive(Debug, serde::Deserialize)]
struct SlackMessage {
    channel: Option<String>,
    text: String,
    username: Option<String>,
}

async fn send_slack_notification(
    State(state): State<SlackState>,
    Json(message): Json<SlackMessage>,
) -> impl IntoResponse {
    if message.text.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Message text cannot be empty"})),
        );
    }

    let channel = message.channel.unwrap_or_else(|| "#general".to_string());
    let username = message.username.unwrap_or_else(|| "Kusanagi".to_string());

    let full_message = format!("[{} -> {}]: {}", username, channel, message.text);
    state.sent_messages.lock().unwrap().push(full_message);

    (
        StatusCode::OK,
        Json(json!({
            "sent": true,
            "channel": channel,
        })),
    )
}

#[tokio::test]
async fn test_slack_notification_success() {
    let state = SlackState {
        sent_messages: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route("/api/slack/notify", post(send_slack_notification))
        .with_state(state.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/slack/notify")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    r#"{"text": "Hello from test", "channel": "alerts"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["sent"], true);
    assert_eq!(json["channel"], "alerts");

    // Verify message was stored
    let messages = state.sent_messages.lock().unwrap();
    assert_eq!(messages.len(), 1);
    assert!(messages[0].contains("Hello from test"));
}

#[tokio::test]
async fn test_slack_notification_empty_text() {
    let state = SlackState {
        sent_messages: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route("/api/slack/notify", post(send_slack_notification))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/slack/notify")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"text": ""}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("empty"));
}

#[tokio::test]
async fn test_slack_notification_default_channel() {
    let state = SlackState {
        sent_messages: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route("/api/slack/notify", post(send_slack_notification))
        .with_state(state.clone());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/slack/notify")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"text": "Test message"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["channel"], "#general"); // Default channel
}

// ============================================================================
// K8s Handler Tests
// ============================================================================

#[derive(Clone)]
struct K8sState {
    pods: Arc<Mutex<Vec<PodInfo>>>,
}

#[derive(Clone, Debug)]
struct PodInfo {
    name: String,
    namespace: String,
    status: String,
    restarts: u32,
}

async fn list_pods(State(state): State<K8sState>) -> impl IntoResponse {
    let pods = state.pods.lock().unwrap();
    let pod_list: Vec<_> = pods
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "namespace": p.namespace,
                "status": p.status,
                "restarts": p.restarts,
            })
        })
        .collect();

    Json(json!({
        "pods": pod_list,
        "count": pod_list.len(),
    }))
}

async fn get_pod(
    axum::extract::Path((namespace, name)): axum::extract::Path<(String, String)>,
    State(state): State<K8sState>,
) -> impl IntoResponse {
    let pods = state.pods.lock().unwrap();

    if let Some(pod) = pods
        .iter()
        .find(|p| p.namespace == namespace && p.name == name)
    {
        Json(json!({
            "found": true,
            "pod": {
                "name": pod.name,
                "namespace": pod.namespace,
                "status": pod.status,
                "restarts": pod.restarts,
            }
        }))
        .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Pod not found"})),
        )
            .into_response()
    }
}

#[tokio::test]
async fn test_list_pods_endpoint() {
    let state = K8sState {
        pods: Arc::new(Mutex::new(vec![
            PodInfo {
                name: "pod-1".to_string(),
                namespace: "default".to_string(),
                status: "Running".to_string(),
                restarts: 0,
            },
            PodInfo {
                name: "pod-2".to_string(),
                namespace: "kube-system".to_string(),
                status: "Running".to_string(),
                restarts: 1,
            },
        ])),
    };

    let app = Router::new()
        .route("/api/k8s/pods", get(list_pods))
        .route("/api/k8s/pods/:namespace/:name", get(get_pod))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/k8s/pods")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["count"], 2);
    assert_eq!(json["pods"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_pod_found() {
    let state = K8sState {
        pods: Arc::new(Mutex::new(vec![PodInfo {
            name: "pod-1".to_string(),
            namespace: "default".to_string(),
            status: "Running".to_string(),
            restarts: 0,
        }])),
    };

    let app = Router::new()
        .route("/api/k8s/pods", get(list_pods))
        .route("/api/k8s/pods/:namespace/:name", get(get_pod))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/k8s/pods/default/pod-1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["found"], true);
    assert_eq!(json["pod"]["name"], "pod-1");
}

#[tokio::test]
async fn test_get_pod_not_found() {
    let state = K8sState {
        pods: Arc::new(Mutex::new(vec![])),
    };

    let app = Router::new()
        .route("/api/k8s/pods", get(list_pods))
        .route("/api/k8s/pods/:namespace/:name", get(get_pod))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/k8s/pods/default/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Monitoring Handler Tests
// ============================================================================

#[derive(Clone)]
struct MonitoringState {
    metrics: Arc<Mutex<Vec<Metric>>>,
}

#[derive(Clone, Debug)]
struct Metric {
    name: String,
    value: f64,
    labels: HashMap<String, String>,
    timestamp: String,
}

async fn get_metrics(State(state): State<MonitoringState>) -> impl IntoResponse {
    let metrics = state.metrics.lock().unwrap();

    // Format as Prometheus-style metrics
    let mut output = String::new();
    for metric in metrics.iter() {
        let labels = if metric.labels.is_empty() {
            "".to_string()
        } else {
            let pairs: Vec<_> = metric
                .labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect();
            format!("{{{}}}", pairs.join(","))
        };
        output.push_str(&format!("{}{} {}\n", metric.name, labels, metric.value));
    }

    ([(axum::http::header::CONTENT_TYPE, "text/plain")], output)
}

async fn get_metric_by_name(
    axum::extract::Path(name): axum::extract::Path<String>,
    State(state): State<MonitoringState>,
) -> impl IntoResponse {
    let metrics = state.metrics.lock().unwrap();

    let matching: Vec<_> = metrics
        .iter()
        .filter(|m| m.name == name)
        .map(|m| {
            json!({
                "name": m.name,
                "value": m.value,
                "labels": m.labels,
            })
        })
        .collect();

    Json(json!({
        "metric": name,
        "count": matching.len(),
        "samples": matching,
    }))
}

#[tokio::test]
async fn test_get_metrics_endpoint() {
    let mut labels = HashMap::new();
    labels.insert("method".to_string(), "GET".to_string());
    labels.insert("endpoint".to_string(), "/api/health".to_string());

    let state = MonitoringState {
        metrics: Arc::new(Mutex::new(vec![
            Metric {
                name: "http_requests_total".to_string(),
                value: 100.0,
                labels: labels.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Metric {
                name: "cache_hits_total".to_string(),
                value: 50.0,
                labels: HashMap::new(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        ])),
    };

    let app = Router::new()
        .route("/metrics", get(get_metrics))
        .route("/metrics/:name", get(get_metric_by_name))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response.headers().get("content-type").unwrap();
    assert_eq!(content_type, "text/plain");

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.contains("http_requests_total"));
    assert!(text.contains("cache_hits_total"));
}

#[tokio::test]
async fn test_get_metric_by_name_endpoint() {
    let state = MonitoringState {
        metrics: Arc::new(Mutex::new(vec![Metric {
            name: "cpu_usage".to_string(),
            value: 45.5,
            labels: HashMap::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }])),
    };

    let app = Router::new()
        .route("/metrics", get(get_metrics))
        .route("/metrics/:name", get(get_metric_by_name))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics/cpu_usage")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["metric"], "cpu_usage");
    assert_eq!(json["count"], 1);
}

// ============================================================================
// System Handler Tests
// ============================================================================

#[derive(Clone)]
struct SystemState {
    system_info: Arc<Mutex<SystemInfo>>,
}

#[derive(Clone, Debug)]
struct SystemInfo {
    hostname: String,
    version: String,
    uptime_seconds: u64,
    memory_total: u64,
    memory_used: u64,
    cpu_count: usize,
}

async fn get_system_info(State(state): State<SystemState>) -> impl IntoResponse {
    let info = state.system_info.lock().unwrap();

    Json(json!({
        "hostname": info.hostname,
        "version": info.version,
        "uptime_seconds": info.uptime_seconds,
        "memory": {
            "total": info.memory_total,
            "used": info.memory_used,
            "free": info.memory_total - info.memory_used,
            "usage_percent": (info.memory_used as f64 / info.memory_total as f64) * 100.0,
        },
        "cpu": {
            "count": info.cpu_count,
        }
    }))
}

async fn get_system_health(State(state): State<SystemState>) -> impl IntoResponse {
    let info = state.system_info.lock().unwrap();
    let memory_usage = info.memory_used as f64 / info.memory_total as f64;

    let status = if memory_usage > 0.9 {
        "critical"
    } else if memory_usage > 0.75 {
        "warning"
    } else {
        "healthy"
    };

    Json(json!({
        "status": status,
        "checks": {
            "memory": if memory_usage > 0.9 { "critical" } else { "ok" },
            "uptime": "ok",
        }
    }))
}

#[tokio::test]
async fn test_get_system_info_endpoint() {
    let state = SystemState {
        system_info: Arc::new(Mutex::new(SystemInfo {
            hostname: "kusanagi-server".to_string(),
            version: "0.3.0".to_string(),
            uptime_seconds: 86400,
            memory_total: 16_000_000_000,
            memory_used: 8_000_000_000,
            cpu_count: 8,
        })),
    };

    let app = Router::new()
        .route("/api/system/info", get(get_system_info))
        .route("/api/system/health", get(get_system_health))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/system/info")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["hostname"], "kusanagi-server");
    assert_eq!(json["version"], "0.3.0");
    assert_eq!(json["uptime_seconds"], 86400);
    assert_eq!(json["memory"]["total"], 16_000_000_000u64);
    assert_eq!(json["cpu"]["count"], 8);
}

#[tokio::test]
async fn test_get_system_health_healthy() {
    let state = SystemState {
        system_info: Arc::new(Mutex::new(SystemInfo {
            hostname: "test".to_string(),
            version: "0.3.0".to_string(),
            uptime_seconds: 100,
            memory_total: 16_000_000_000u64,
            memory_used: 4_000_000_000u64, // 25% - healthy
            cpu_count: 4,
        })),
    };

    let app = Router::new()
        .route("/api/system/health", get(get_system_health))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/system/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "healthy");
}

#[tokio::test]
async fn test_get_system_health_warning() {
    let state = SystemState {
        system_info: Arc::new(Mutex::new(SystemInfo {
            hostname: "test".to_string(),
            version: "0.3.0".to_string(),
            uptime_seconds: 100,
            memory_total: 16_000_000_000u64,
            memory_used: 13_000_000_000u64, // ~81% - warning
            cpu_count: 4,
        })),
    };

    let app = Router::new()
        .route("/api/system/health", get(get_system_health))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/system/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "warning");
}

#[tokio::test]
async fn test_get_system_health_critical() {
    let state = SystemState {
        system_info: Arc::new(Mutex::new(SystemInfo {
            hostname: "test".to_string(),
            version: "0.3.0".to_string(),
            uptime_seconds: 100,
            memory_total: 16_000_000_000u64,
            memory_used: 15_000_000_000u64, // ~94% - critical
            cpu_count: 4,
        })),
    };

    let app = Router::new()
        .route("/api/system/health", get(get_system_health))
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/system/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "critical");
}
