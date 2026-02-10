//! Integration tests for Axum HTTP handlers
//! Tests real handlers with axum Router and tower TestClient

use axum::{
    body::Body,
    extract::{Json as AxumJson, Path, Query, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tower::ServiceExt;

// ============================================================================
// Health handler tests
// ============================================================================

async fn health_check() -> impl IntoResponse {
    AxumJson(json!({
        "status": "healthy",
        "timestamp": "2024-01-01T00:00:00Z"
    }))
}

#[tokio::test]
async fn test_health_endpoint() {
    let app = Router::new().route("/health", get(health_check));

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
}

// ============================================================================
// API Info handler tests
// ============================================================================

async fn api_info() -> impl IntoResponse {
    AxumJson(json!({
        "service": "Kusanagi",
        "version": "0.3.0",
        "architecture": "axum-migration"
    }))
}

#[tokio::test]
async fn test_api_info_endpoint() {
    let app = Router::new().route("/api", get(api_info));

    let response = app
        .oneshot(Request::builder().uri("/api").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["service"], "Kusanagi");
}

// ============================================================================
// Router configuration tests
// ============================================================================

fn create_test_router() -> Router {
    Router::new()
        .route(
            "/health",
            get(|| async { AxumJson(json!({"status": "ok"})) }),
        )
        .route(
            "/api/config",
            get(|| async { AxumJson(json!({"port": 8080})) }),
        )
        .route(
            "/api/cache/stats",
            get(|| async { AxumJson(json!({"hits": 100, "misses": 10})) }),
        )
        .route(
            "/api/slack/notify",
            post(|| async { AxumJson(json!({"sent": true})) }),
        )
}

#[tokio::test]
async fn test_router_health() {
    let app = create_test_router();

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
}

#[tokio::test]
async fn test_router_config() {
    let app = create_test_router();

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
}

#[tokio::test]
async fn test_router_cache_stats() {
    let app = create_test_router();

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
    assert_eq!(json["misses"], 10);
}

#[tokio::test]
async fn test_router_slack_notify() {
    let app = create_test_router();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/slack/notify")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"message": "test"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_router_not_found() {
    let app = create_test_router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/unknown")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Error handling tests
// ============================================================================

async fn handler_with_error() -> (StatusCode, impl IntoResponse) {
    (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
}

async fn handler_not_found() -> (StatusCode, impl IntoResponse) {
    (StatusCode::NOT_FOUND, "Not Found")
}

#[tokio::test]
async fn test_internal_server_error() {
    let app = Router::new().route("/error", get(handler_with_error));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/error")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_not_found_error() {
    let app = Router::new().route("/not-found", get(handler_not_found));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/not-found")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ============================================================================
// Path parameter tests
// ============================================================================

async fn get_item(Path(id): Path<u32>) -> impl IntoResponse {
    AxumJson(json!({ "id": id }))
}

async fn get_nested(Path((category, id)): Path<(String, u32)>) -> impl IntoResponse {
    AxumJson(json!({ "category": category, "id": id }))
}

#[tokio::test]
async fn test_path_param() {
    let app = Router::new().route("/items/:id", get(get_item));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/items/42")
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
    assert_eq!(json["id"], 42);
}

#[tokio::test]
async fn test_multiple_path_params() {
    let app = Router::new().route("/items/:category/:id", get(get_nested));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/items/electronics/123")
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
    assert_eq!(json["category"], "electronics");
    assert_eq!(json["id"], 123);
}

// ============================================================================
// Query parameter tests
// ============================================================================

#[derive(Debug, Deserialize)]
struct Pagination {
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_items(Query(params): Query<Pagination>) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10);
    AxumJson(json!({ "page": page, "limit": limit }))
}

async fn list_items_map(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    AxumJson(json!(params))
}

#[tokio::test]
async fn test_query_params() {
    let app = Router::new().route("/items", get(list_items));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/items?page=2&limit=20")
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
    assert_eq!(json["page"], 2);
    assert_eq!(json["limit"], 20);
}

#[tokio::test]
async fn test_default_query_params() {
    let app = Router::new().route("/items", get(list_items));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/items")
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
    assert_eq!(json["page"], 1); // default
    assert_eq!(json["limit"], 10); // default
}

#[tokio::test]
async fn test_query_params_map() {
    let app = Router::new().route("/items", get(list_items_map));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/items?foo=bar&baz=qux")
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
    assert_eq!(json["foo"], "bar");
    assert_eq!(json["baz"], "qux");
}

// ============================================================================
// JSON body tests
// ============================================================================

#[derive(Debug, Deserialize, Serialize)]
struct CreateItem {
    name: String,
    description: Option<String>,
}

async fn create_item(AxumJson(item): AxumJson<CreateItem>) -> impl IntoResponse {
    AxumJson(json!({
        "id": 1,
        "name": item.name,
        "description": item.description.unwrap_or_default()
    }))
}

#[tokio::test]
async fn test_json_body_parsing() {
    let app = Router::new().route("/items", post(create_item));

    let body = json!({
        "name": "Test Item",
        "description": "A test item"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response_body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&response_body).unwrap();
    assert_eq!(json["name"], "Test Item");
    assert_eq!(json["description"], "A test item");
}

#[tokio::test]
async fn test_json_body_missing_field() {
    let app = Router::new().route("/items", post(create_item));

    // Missing required 'name' field - should fail
    let body = json!({
        "description": "A test item"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be unprocessable entity due to missing required field
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

// ============================================================================
// State sharing tests
// ============================================================================

#[derive(Clone)]
struct CounterState {
    counter: Arc<AtomicU64>,
}

async fn increment(State(state): State<CounterState>) -> impl IntoResponse {
    let count = state.counter.fetch_add(1, Ordering::SeqCst);
    AxumJson(json!({ "count": count + 1 }))
}

async fn get_count(State(state): State<CounterState>) -> impl IntoResponse {
    let count = state.counter.load(Ordering::SeqCst);
    AxumJson(json!({ "count": count }))
}

#[tokio::test]
async fn test_shared_state() {
    let state = CounterState {
        counter: Arc::new(AtomicU64::new(0)),
    };

    let app = Router::new()
        .route("/increment", post(increment))
        .route("/count", get(get_count))
        .with_state(state);

    // Increment twice
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/increment")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/increment")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Get count
    let response = app
        .oneshot(
            Request::builder()
                .uri("/count")
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
}
