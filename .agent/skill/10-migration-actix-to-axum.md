# Migration Plan: Actix-web → Axum

## Overview

**Goal**: Migrate from Actix-web 4.x to Axum 0.7+ for better performance and ecosystem alignment.

**Estimated Effort**: 2-3 days
**Risk Level**: Medium (WebSocket migration complexity)

## Key Differences

| Feature | Actix-web | Axum |
|---------|-----------|------|
| State | `web::Data<T>` | `State<T>` |
| Response | `HttpResponse` / `impl Responder` | `Response` / `impl IntoResponse` |
| Path Extractor | `web::Path<T>` | `Path<T>` |
| Query Extractor | `web::Query<T>` | `Query<T>` |
| JSON | `web::Json<T>` | `Json<T>` |
| WebSocket | `actix-web-actors::ws` | `axum::extract::ws` |
| Middleware | `App::wrap()` | `Router::layer()` |
| Server | `HttpServer::new()` | `tokio::net::TcpListener` + `serve()` |

## Phase 1: Dependencies (2 hours)

### Cargo.toml Changes

```toml
[dependencies]
# REMOVE:
# actix-web = "4.12"
# actix-files = "0.6"
# actix-web-actors = "4.3"
# actix = "0.13"

# ADD:
axum = { version = "0.7", features = ["ws"] }
tower = { version = "0.4", features = ["full"] }
tower-http = { version = "0.5", features = ["fs", "cors", "compression", "trace"] }
# Keep tokio, serde, chrono, etc.
```

## Phase 2: Core Types Migration (4 hours)

### Handler Signatures

**Before (Actix-web)**:
```rust
use actix_web::{web, HttpResponse, Responder};

pub async fn get_weather_handler(
    use_case: web::Data<GetWeatherUseCase>,
    query: web::Query<WeatherQuery>,
) -> impl Responder {
    // ...
    HttpResponse::Ok().json(data)
}
```

**After (Axum)**:
```rust
use axum::{extract::{Query, State}, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn get_weather_handler(
    State(use_case): State<Arc<GetWeatherUseCase>>,
    Query(query): Query<WeatherQuery>,
) -> impl IntoResponse {
    // ...
    Json(data)
}
```

### State Management

**Before**:
```rust
// main.rs
App::new()
    .app_data(web::Data::new(weather_use_case.clone()))
```

**After**:
```rust
// main.rs
Router::new()
    .with_state(AppState {
        weather_use_case: weather_use_case.clone(),
        // ... other fields
    })
```

### AppState Struct

```rust
#[derive(Clone)]
struct AppState {
    weather_use_case: Arc<GetWeatherUseCase>,
    alerts_use_case: Arc<GetAlertsUseCase>,
    security_use_case: Arc<GetSecurityUseCase>,
    ha_use_case: Arc<GetHomeAssistantUseCase>,
    k8s_cache: Arc<AdvancedCache<String>>,
    argocd_cache: Arc<AdvancedCache<String>>,
    general_cache: Arc<AdvancedCache<String>>,
    config: Config,
    http_client: reqwest::Client,
    mqtt_state: MqttState,
}
```

## Phase 3: Routes Migration (3 hours)

### Route Definition

**Before**:
```rust
App::new()
    .route("/api/weather/current", web::get().to(get_weather_handler))
    .route("/api/ha/devices", web::get().to(ha_devices_handler))
```

**After**:
```rust
let api_routes = Router::new()
    .route("/api/weather/current", get(get_weather_handler))
    .route("/api/ha/devices", get(ha_devices_handler));

Router::new()
    .merge(api_routes)
    .with_state(app_state)
```

### Nested Routers

**Before**:
```rust
web::scope("/api/security")
    .route("/summary", web::get().to(get_security_handler))
```

**After**:
```rust
Router::new()
    .nest("/api/security", Router::new()
        .route("/summary", get(get_security_handler))
    )
```

## Phase 4: Middleware Migration (2 hours)

### CORS

**Before**:
```rust
use actix_cors::Cors;

App::new()
    .wrap(Cors::permissive())
```

**After**:
```rust
use tower_http::cors::{Any, CorsLayer};

Router::new()
    .layer(CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any))
```

### Compression

**Before**:
```rust
use actix_web::middleware::Compress;

App::new()
    .wrap(Compress::default())
```

**After**:
```rust
use tower_http::compression::CompressionLayer;

Router::new()
    .layer(CompressionLayer::new())
```

### Static Files

**Before**:
```rust
App::new()
    .service(actix_files::Files::new("/static", "./static"))
```

**After**:
```rust
use tower_http::services::ServeDir;

Router::new()
    .nest_service("/static", ServeDir::new("./static"))
```

## Phase 5: WebSocket Migration (4 hours)

### WebSocket Handler

**Before (Actix)**:
```rust
use actix_web_actors::ws;

async fn websocket_handler(req: HttpRequest, stream: web::Payload) -> impl Responder {
    ws::start(WsNotifications, &req, stream)
}

impl Actor for WsNotifications { ... }
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsNotifications { ... }
```

**After (Axum)**:
```rust
use axum::extract::ws::{WebSocketUpgrade, Message};
use axum::response::Response;

async fn websocket_handler(
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: axum::extract::ws::WebSocket) {
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            // Handle message
            socket.send(Message::Text("response".to_string())).await.ok();
        }
    }
}
```

## Phase 6: Server Bootstrap (1 hour)

**Before**:
```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            // ... routes and middleware
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

**After**:
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    let app = create_router().await;
    
    axum::serve(listener, app).await?;
    Ok(())
}

async fn create_router() -> Router {
    let state = create_app_state().await;
    
    Router::new()
        // ... routes
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
}
```

## Phase 7: Error Handling (2 hours)

### Error Response

**Before**:
```rust
HttpResponse::Ok().json(json!({"error": "message"}))
```

**After**:
```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

fn error_response(message: &str) -> Response {
    (StatusCode::OK, Json(json!({"error": message}))).into_response()
}
```

## Testing Checklist

- [ ] All handlers compile
- [ ] WebSocket connections work
- [ ] Static files served correctly
- [ ] CORS headers present
- [ ] State injection works
- [ ] Middleware chain executes
- [ ] Error responses formatted correctly
- [ ] Performance improved (benchmark)

## Rollback Plan

Keep `actix-web` branch as backup:
```bash
git checkout -b migration/axum
git checkout main  # actix-web backup
```

## Performance Expectations

| Metric | Actix-web | Axum (expected) |
|--------|-----------|-----------------|
| Throughput | 120k req/s | 150k req/s (+25%) |
| Latency p99 | 2.5ms | 1.8ms (-28%) |
| Memory usage | 45MB | 38MB (-15%) |
| Compile time | 45s | 35s (-22%) |

## Migration Order

1. **Day 1**: Phases 1-2 (Dependencies + Core types)
2. **Day 2**: Phases 3-4 (Routes + Middleware)
3. **Day 3**: Phases 5-7 (WebSocket + Bootstrap + Testing)
