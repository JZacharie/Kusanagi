# Kusanagi - Guide pour Agents

## 🎯 Vue d'ensemble

Kusanagi est une plateforme de monitoring Kubernetes en Rust utilisant l'**architecture hexagonale** (ports et adapters) avec le framework **Axum**.

```
┌─────────────────────────────────────────────────────────────┐
│  Architecture Hexagonale + Axum                              │
├─────────────────────────────────────────────────────────────┤
│  Interfaces (HTTP/WS)  →  Axum Handlers                     │
│  Application           →  Use Cases                          │
│  Domain                →  Entities + Ports (traits)          │
│  Infrastructure        →  Repositories (implémentations)     │
└─────────────────────────────────────────────────────────────┘
```

---

## 🏗️ Structure du projet

```
src/
├── main.rs                    # Entry point Axum (tokio::main)
├── state.rs                   # AppState - état partagé
├── routes.rs                  # Router Axum
├── lib.rs                     # Exports du library crate
│
├── domain/                    # 💙 Cœur métier (pur Rust)
│   ├── entities/              # Structs métier (Weather, Alert, etc.)
│   ├── ports/                 # Traits (Repository, Service)
│   └── services/              # Logique métier pure
│
├── application/               # 💚 Cas d'usage
│   └── use_cases/             # GetWeatherUseCase, GetAlertsUseCase, etc.
│
├── infrastructure/            # 💛 Adapters techniques
│   └── repositories/          # Implémentations HTTP/Kubernetes
│
├── interfaces/                # 🧡 Interface utilisateur
│   └── http/                  # Handlers Axum
│       ├── alert_handlers.rs
│       ├── backup_handlers.rs
│       ├── homeassistant_handlers.rs
│       ├── security_handlers.rs
│       └── weather_handlers.rs
│
├── api_handlers/              # 📦 Handlers additionnels
│   ├── cache.rs
│   ├── config.rs
│   ├── health.rs
│   ├── slack.rs
│   └── websocket.rs
│
└── handlers/                  # 📦 Stubs de base
    ├── health.rs
    ├── system.rs
    ├── k8s.rs
    ├── monitoring.rs
    └── cache.rs
```

---

## 🔧 Patterns obligatoires

### 1. Handler Axum (Interfaces)

```rust
use axum::{
    extract::{State, Path, Query},
    response::IntoResponse,
    Json,
};
use kusanagi::state::AppState;

// Handler simple
pub async fn get_weather_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.weather_use_case.execute(GetWeatherInput::default()).await {
        Ok(weather) => Json(weather).into_response(),
        Err(e) => Json(json!({"error": e.to_string()})).into_response(),
    }
}

// Handler avec Path params
pub async fn get_security_report_handler(
    State(state): State<AppState>,
    Path(path): Path<ReportPath>,
) -> impl IntoResponse {
    match state.security_use_case.get_report(&path.category, &path.name).await {
        Ok(report) => Json(report).into_response(),
        Err(e) => Json(json!({"error": e.to_string()})).into_response(),
    }
}

// Handler avec Query params
#[derive(Debug, serde::Deserialize)]
pub struct AlertsQuery {
    #[serde(default)]
    pub refresh: bool,
}

pub async fn get_alerts_handler(
    State(state): State<AppState>,
    Query(query): Query<AlertsQuery>,
) -> impl IntoResponse {
    let input = GetAlertsInput { force_refresh: query.refresh };
    // ...
}
```

### 2. Use Case (Application)

```rust
pub struct GetWeatherUseCase {
    repository: Arc<dyn WeatherRepository>,
}

impl GetWeatherUseCase {
    pub fn new(repository: Arc<dyn WeatherRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: GetWeatherInput) -> Result<WeatherResponse> {
        self.repository.get_multi_city_weather(input.force_refresh).await
    }

    pub fn is_local_mode(&self) -> bool {
        self.repository.is_local_mode()
    }
}
```

### 3. Repository (Infrastructure)

```rust
#[async_trait]
pub trait WeatherRepository: Send + Sync {
    async fn get_multi_city_weather(&self, force_refresh: bool) -> Result<WeatherResponse>;
    async fn force_refresh(&self) -> Result<()>;
    fn is_local_mode(&self) -> bool;
}

pub struct WeatherRepositoryImpl {
    cache: Arc<AdvancedCache<WeatherResponse>>,
    client: reqwest::Client,
}

#[async_trait]
impl WeatherRepository for WeatherRepositoryImpl {
    async fn get_multi_city_weather(&self, force_refresh: bool) -> Result<WeatherResponse> {
        // Implémentation HTTP + cache
    }
    // ...
}
```

### 4. AppState

```rust
#[derive(Clone)]
pub struct AppState {
    pub k8s_cache: Arc<AdvancedCache<String>>,
    pub argocd_cache: Arc<AdvancedCache<String>>,
    pub general_cache: Arc<AdvancedCache<String>>,
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

## 🌐 Configuration des routes

```rust
// src/main.rs
let app = Router::new()
    // Core routes
    .route("/", get(index_handler))
    .route("/health", get(health_check))
    .route("/api", get(api_info))
    .route("/api/config", get(get_config))
    .route("/api/cache/stats", get(cache_stats))
    .route("/api/slack/notify", post(send_slack_notification))
    // WebSocket
    .route("/api/ws/notifications", get(ws_notifications_handler))
    // Static files
    .nest_service("/static", ServeDir::new("./static"))
    // Middleware
    .layer(CorsLayer::permissive())
    .layer(CompressionLayer::new())
    .layer(TraceLayer::new_for_http())
    // State
    .with_state(state);
```

---

## 📦 Dépendances clés

```toml
[dependencies]
# Core
tokio = { version = "1.0", features = ["full"] }
axum = { version = "0.7", features = ["ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors", "compression-gzip", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# HTTP Client
reqwest = { version = "0.12", features = ["json"] }

# Kubernetes
kube = { version = "3.0", features = ["runtime", "derive"] }
k8s-openapi = { version = "0.27", features = ["latest"] }

# Observability
tracing = "0.1"
tracing-subscriber = "0.3"

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Async
async-trait = "0.1"
futures = "0.3"
```

---

## 🔍 Checklist pour nouvelles fonctionnalités

### Ajouter un nouveau endpoint API

- [ ] Créer/modifier le handler dans `src/interfaces/http/` ou `src/api_handlers/`
- [ ] Utiliser le pattern `State<AppState>` pour accéder aux use cases
- [ ] Retourner `impl IntoResponse` avec `Json()` ou format approprié
- [ ] Ajouter la route dans `src/main.rs` avec `.route("/path", get|post(handler))`
- [ ] Si nécessaire, créer le UseCase dans `src/application/use_cases/`
- [ ] Si nécessaire, créer le Repository dans `src/infrastructure/repositories/`
- [ ] Mettre à jour `AppState` si nouveau use case

### Ajouter un WebSocket

- [ ] Utiliser `WebSocketUpgrade` dans le handler
- [ ] Implémenter `handle_socket(socket, state)` avec `tokio::select!`
- [ ] Gérer les messages: `Message::Text`, `Message::Ping`, `Message::Close`
- [ ] Spawn des tâches pour heartbeat et background checks
- [ ] Définir les types de messages avec `#[serde(tag = "type")]`

---

## ⚠️ Conventions de code

1. **Imports**: Grouper par catégorie (std, crates, locaux)
2. **Erreurs**: Toujours retourner des réponses JSON avec message d'erreur, jamais de panic
3. **State**: Toujours passer par `State<AppState>`, jamais de global state
4. **Cache**: Utiliser `Arc<AdvancedCache<T>>` pour le cache partagé
5. **Async**: Utiliser `async_trait` pour les traits avec méthodes async
6. **Logging**: Utiliser `tracing` (info!, debug!, warn!, error!)

---

## 🚀 Commandes utiles

```bash
# Build
 cargo build --release

# Run
 cargo run --release

# Test
 cargo test

# Check
 cargo check

# Fix auto
 cargo fix --allow-dirty
```

---

## 📚 Ressources

- [Migration Axum Summary](MIGRATION_AXUM_SUMMARY.md)
- [Documentation Axum](https://docs.rs/axum/latest/axum/)
- [Documentation Tower](https://docs.rs/tower/latest/tower/)

---

**Version**: 0.3.0  
**Framework**: Axum 0.7  
**Architecture**: Hexagonale (Ports & Adapters)
