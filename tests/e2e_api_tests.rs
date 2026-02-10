//! End-to-End API Tests
//! Tests complets simulant des requêtes réelles sur l'application

use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    routing::{get, post},
    Json as AxumJson, Router,
};
use serde_json::json;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;
use tower::ServiceExt;

// State global de l'application
#[derive(Clone)]
struct AppState {
    request_count: Arc<AtomicU64>,
    start_time: Instant,
}

impl AppState {
    fn new() -> Self {
        Self {
            request_count: Arc::new(AtomicU64::new(0)),
            start_time: Instant::now(),
        }
    }

    fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn request_count(&self) -> u64 {
        self.request_count.load(Ordering::SeqCst)
    }
}

// Handlers
async fn health_handler(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    AxumJson(json!({
        "status": "healthy",
        "uptime_secs": state.uptime().as_secs(),
        "requests": state.request_count(),
    }))
}

async fn api_info_handler() -> impl axum::response::IntoResponse {
    AxumJson(json!({
        "name": "Kusanagi",
        "version": "0.3.0",
        "endpoints": [
            "GET /health",
            "GET /api/info",
            "GET /api/metrics",
            "GET /api/alerts",
            "POST /api/alerts/acknowledge",
        ]
    }))
}

async fn metrics_handler(State(state): State<AppState>) -> impl axum::response::IntoResponse {
    let uptime = state.uptime();
    let requests = state.request_count();
    let rps = if uptime.as_secs() > 0 {
        requests as f64 / uptime.as_secs_f64()
    } else {
        0.0
    };

    AxumJson(json!({
        "uptime_seconds": uptime.as_secs(),
        "total_requests": requests,
        "requests_per_second": rps,
    }))
}

// Simuler une base de données
#[derive(Clone)]
struct MockDatabase {
    alerts: Arc<tokio::sync::RwLock<Vec<serde_json::Value>>>,
}

impl MockDatabase {
    fn new() -> Self {
        Self {
            alerts: Arc::new(tokio::sync::RwLock::new(vec![
                json!({
                    "id": "alert-1",
                    "severity": "critical",
                    "message": "High CPU usage",
                    "acknowledged": false,
                }),
                json!({
                    "id": "alert-2",
                    "severity": "warning",
                    "message": "Disk space low",
                    "acknowledged": false,
                }),
            ])),
        }
    }

    async fn get_alerts(&self) -> Vec<serde_json::Value> {
        let alerts = self.alerts.read().await;
        alerts.clone()
    }

    async fn acknowledge_alert(&self, id: &str) -> bool {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a["id"] == id) {
            alert["acknowledged"] = json!(true);
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
struct DbState {
    db: MockDatabase,
}

async fn get_alerts_handler(State(state): State<DbState>) -> impl axum::response::IntoResponse {
    let alerts = state.db.get_alerts().await;
    AxumJson(json!({"alerts": alerts, "count": alerts.len()}))
}

#[derive(serde::Deserialize)]
struct AckRequest {
    id: String,
}

async fn acknowledge_alert_handler(
    State(state): State<DbState>,
    axum::extract::Json(body): axum::extract::Json<AckRequest>,
) -> impl axum::response::IntoResponse {
    let success = state.db.acknowledge_alert(&body.id).await;

    if success {
        (
            StatusCode::OK,
            AxumJson(json!({"acknowledged": true, "id": body.id})),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            AxumJson(json!({"error": "Alert not found", "id": body.id})),
        )
    }
}

// Construire l'application complète
fn create_app() -> Router {
    let state = AppState::new();

    Router::new()
        .route("/health", get(health_handler))
        .route("/api/info", get(api_info_handler))
        .route("/api/metrics", get(metrics_handler))
        .with_state(state)
}

fn create_app_with_db() -> Router {
    let db_state = DbState {
        db: MockDatabase::new(),
    };

    Router::new()
        .route("/api/alerts", get(get_alerts_handler))
        .route("/api/alerts/acknowledge", post(acknowledge_alert_handler))
        .with_state(db_state)
}

// ============================================================================
// Tests End-to-End
// ============================================================================

#[tokio::test]
async fn test_e2e_health_endpoint() {
    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
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
    assert!(json["uptime_secs"].as_u64().is_some());
}

#[tokio::test]
async fn test_e2e_api_info() {
    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/info")
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

    assert_eq!(json["name"], "Kusanagi");
    assert!(json["endpoints"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_e2e_metrics() {
    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/metrics")
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

    // Verify structure
    assert!(json["uptime_seconds"].as_u64().is_some());
    assert!(json["total_requests"].as_u64().is_some());
    assert!(json["requests_per_second"].as_f64().is_some());
}

#[tokio::test]
async fn test_e2e_alerts_list() {
    let app = create_app_with_db();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/alerts")
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
    assert!(json["alerts"].as_array().unwrap().len() == 2);
}

#[tokio::test]
async fn test_e2e_acknowledge_alert_success() {
    let app = create_app_with_db();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/alerts/acknowledge")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"id": "alert-1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["acknowledged"].as_bool().unwrap());
    assert_eq!(json["id"], "alert-1");
}

#[tokio::test]
async fn test_e2e_acknowledge_alert_not_found() {
    let app = create_app_with_db();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/alerts/acknowledge")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"id": "nonexistent"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_e2e_not_found() {
    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_e2e_method_not_allowed() {
    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

// ============================================================================
// Tests de Performance Basiques
// ============================================================================

#[tokio::test]
async fn test_multiple_concurrent_requests() {
    let app = create_app();

    let mut handles = vec![];

    for i in 0..10 {
        let app_clone = app.clone();
        let handle = tokio::spawn(async move {
            let start = Instant::now();

            let response = app_clone
                .oneshot(
                    Request::builder()
                        .method(Method::GET)
                        .uri("/health")
                        .header("X-Request-ID", format!("req-{}", i))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();

            let duration = start.elapsed();
            (response.status(), duration)
        });

        handles.push(handle);
    }

    for handle in handles {
        let (status, _duration) = handle.await.unwrap();
        assert_eq!(status, StatusCode::OK);
    }
}

#[tokio::test]
async fn test_request_response_time() {
    let app = create_app();

    let start = Instant::now();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let duration = start.elapsed();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        duration < Duration::from_millis(100),
        "Request took too long: {:?}",
        duration
    );
}

// ============================================================================
// Tests de Flux Complets
// ============================================================================

#[tokio::test]
async fn test_full_alert_workflow() {
    let app = create_app_with_db();

    // 1. List alerts
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/alerts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let alert_id = json["alerts"][0]["id"].as_str().unwrap();
    assert_eq!(json["alerts"][0]["acknowledged"], false);

    // 2. Acknowledge the alert
    let ack_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/alerts/acknowledge")
                .header("Content-Type", "application/json")
                .body(Body::from(format!(r#"{{"id": "{}"}}"#, alert_id)))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(ack_response.status(), StatusCode::OK);

    // 3. Verify the alert is now acknowledged
    let list_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/alerts")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let acknowledged_alert = json["alerts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == alert_id)
        .unwrap();

    assert_eq!(acknowledged_alert["acknowledged"], true);
}

// ============================================================================
// Tests de Headers et Métadonnées
// ============================================================================

#[tokio::test]
async fn test_response_headers() {
    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Content-Type should be JSON
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type.unwrap().to_str().unwrap().contains("json"));
}
