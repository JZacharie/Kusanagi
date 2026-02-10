# Skill: Kusanagi Development v0.3.0

## TL;DR
Kusanagi v0.3.0 = Kubernetes monitoring platform in Rust using **Axum** + **Hexagonal Architecture**.

---

## Quick Reference

### Add New API Endpoint
```rust
// 1. Handler in src/api_handlers/my_handler.rs
use axum::{extract::State, response::IntoResponse, Json};
use kusanagi::state::AppState;

pub async fn my_handler(State(state): State<AppState>) -> impl IntoResponse {
    match state.my_use_case.execute().await {
        Ok(data) => Json(data).into_response(),
        Err(e) => Json(json!({"error": e.to_string()})).into_response(),
    }
}

// 2. Route in src/main.rs
.route("/api/my-endpoint", get(my_handler))
```

### Handler with Params
```rust
// Path params
pub async fn handler(Path((ns, name)): Path<(String, String)>) {}

// Query params
#[derive(serde::Deserialize)]
struct Query { refresh: bool }
pub async fn handler(Query(q): Query<Query>) {}
```

### WebSocket Handler
```rust
use axum::extract::ws::{WebSocketUpgrade, Message, WebSocket};

pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(t)) => { /* handle */ }
            Ok(Message::Close(_)) => break,
            _ => {}
        }
    }
}
```

---

## Architecture Layers

```
┌────────────────────────────────────────┐
│  HTTP Handlers (src/api_handlers/)     │  ← You are here
│  State<AppState> → IntoResponse        │
├────────────────────────────────────────┤
│  Use Cases (src/application/)          │  ← Business logic
│  Arc<dyn Repository>                   │
├────────────────────────────────────────┤
│  Repositories (src/infrastructure/)    │  ← External APIs
│  HTTP calls, Kubernetes API, etc.      │
├────────────────────────────────────────┤
│  Entities (src/domain/entities/)       │  ← Data structures
│  Plain structs with serde              │
└────────────────────────────────────────┘
```

---

## AppState Fields
```rust
pub struct AppState {
    pub k8s_cache: Arc<AdvancedCache<String>>,      // K8s data cache
    pub argocd_cache: Arc<AdvancedCache<String>>,   // ArgoCD cache
    pub general_cache: Arc<AdvancedCache<String>>,  // General cache
    pub alerts_use_case: Arc<GetAlertsUseCase>,
    pub weather_use_case: Arc<GetWeatherUseCase>,
    pub security_use_case: Arc<GetSecurityUseCase>,
    pub ha_use_case: Arc<GetHomeAssistantUseCase>,
    pub backup_use_case: Arc<BackupUseCase>,
    pub kube_client: Option<Arc<kube::Client>>,
    pub http_client: Arc<reqwest::Client>,
}
```

---

## Common Tasks

### Add Use Case to State
1. Add field to `AppState` struct in `src/state.rs`
2. Initialize in `AppState::new()` method

### Create Repository
```rust
#[async_trait]
pub trait MyRepository: Send + Sync {
    async fn get_data(&self) -> Result<Data>;
}

pub struct MyRepositoryImpl { /* fields */ }

#[async_trait]
impl MyRepository for MyRepositoryImpl {
    async fn get_data(&self) -> Result<Data> {
        // Implementation
    }
}
```

### Error Response Pattern
```rust
// Always return JSON, never panic
Err(e) => Json(json!({
    "error": e.to_string(),
    "fallback": "default value"
})).into_response()
```

---

## Build & Run
```bash
cargo build --release          # Release build
cargo run --release            # Run
cargo check                    # Fast check
```

---

## Key Dependencies
- `axum` - Web framework
- `tower-http` - Middleware (CORS, compression, static files)
- `kube` - Kubernetes client
- `reqwest` - HTTP client
- `tracing` - Logging

---

## File Locations
- Handlers: `src/api_handlers/` or `src/interfaces/http/`
- State: `src/state.rs`
- Routes: `src/main.rs` (Router configuration)
- Use Cases: `src/application/use_cases/`
- Repositories: `src/infrastructure/repositories/`
