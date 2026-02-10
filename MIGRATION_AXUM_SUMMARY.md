# Migration Actix-web → Axum - Rapport Final

## 🎯 Objectif
Migration complète de l'architecture HTTP de Actix-web 4.12 vers Axum 0.7, tout en préservant l'architecture hexagonale existante.

---

## ✅ Phases complétées

### Phase 1: Core Setup & Handlers Conversion
**Statut:** ✅ Terminé

#### Changements majeurs:
- **Cargo.toml**: Remplacement des dépendances Actix-web par Axum
  ```toml
  # Avant
  actix-web = "4.12"
  actix-files = "0.6"
  actix-web-actors = "4.3"
  
  # Après
  axum = { version = "0.7", features = ["ws"] }
  tower = "0.4"
  tower-http = { version = "0.5", features = ["fs", "cors", "compression-gzip", "trace"] }
  tracing-subscriber = "0.3"
  ```

- **AppState** (`src/state.rs`): Centralisation de l'état applicatif
  - Caches: `k8s_cache`, `argocd_cache`, `general_cache`
  - Use cases: `alerts_use_case`, `weather_use_case`, `security_use_case`, `ha_use_case`, `backup_use_case`
  - Clients: `kube_client`, `http_client`

- **Handlers convertis**: Tous les handlers HTTP migrés vers le pattern Axum
  - `State<AppState>` au lieu de `web::Data<T>`
  - `impl IntoResponse` au lieu de `impl Responder`
  - `Json<T>` pour les réponses JSON

### Phase 2: Migration WebSocket
**Statut:** ✅ Terminé

#### Changements:
- **Ancien**: `actix-web-actors` avec Actor + StreamHandler
- **Nouveau**: `axum::ws` avec fonctions async + `tokio::select!`

#### Fichier: `src/api_handlers/websocket.rs`

| Fonctionnalité | Actix-web-actors | Axum::ws |
|---------------|------------------|----------|
| Structure | Actor pattern | Fonction async |
| Context | `WebsocketContext<Self>` | `WebSocket` socket |
| Heartbeat | `ctx.run_interval()` | `tokio::spawn` + `interval.tick()` |
| Messages | `StreamHandler<ws::Message>` | `socket.recv()` |
| Envoi | `ctx.text()` | `socket.send(Message::Text())` |

### Phase 3: Finalisation
**Statut:** ✅ Terminé

- ✅ Nettoyage des warnings
- ✅ Build release validé
- ✅ Documentation mise à jour

---

## 📁 Structure des fichiers

```
src/
├── main.rs                    # Entry point Axum (tokio::main)
├── state.rs                   # AppState avec tous les use cases
├── routes.rs                  # Router Axum
├── handlers/                  # Handlers stubs basiques
│   ├── mod.rs
│   ├── health.rs
│   ├── system.rs
│   ├── k8s.rs
│   ├── monitoring.rs
│   └── cache.rs
├── api_handlers/              # Handlers Axum complets
│   ├── mod.rs
│   ├── health.rs
│   ├── cache.rs
│   ├── config.rs
│   ├── slack.rs
│   └── websocket.rs           # WebSocket handler (Phase 2)
└── interfaces/http/           # Handlers hexagonaux convertis
    ├── mod.rs
    ├── alert_handlers.rs
    ├── backup_handlers.rs
    ├── homeassistant_handlers.rs
    ├── security_handlers.rs
    └── weather_handlers.rs
```

---

## 🔧 Patterns de migration

### Handler HTTP simple

**Avant (Actix-web):**
```rust
use actix_web::{web, HttpResponse, Responder};

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({"status": "healthy"}))
}

// Route
app.route("/health", web::get().to(health_check))
```

**Après (Axum):**
```rust
use axum::{response::IntoResponse, Json};

async fn health_check() -> impl IntoResponse {
    Json(json!({"status": "healthy"}))
}

// Route
Router::new().route("/health", get(health_check))
```

### Handler avec State

**Avant (Actix-web):**
```rust
async fn get_alerts(
    data: web::Data<Arc<GetAlertsUseCase>>,
) -> impl Responder {
    match data.execute().await {
        Ok(alerts) => HttpResponse::Ok().json(alerts),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": e})),
    }
}
```

**Après (Axum):**
```rust
async fn get_alerts_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.alerts_use_case.execute().await {
        Ok(alerts) => Json(alerts).into_response(),
        Err(e) => Json(json!({"error": e.to_string()})).into_response(),
    }
}
```

### WebSocket

**Avant (Actix-web-actors):**
```rust
impl Actor for NotificationSession {
    type Context = ws::WebsocketContext<Self>;
    
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            ctx.ping(b"");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for NotificationSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            // ...
        }
    }
}
```

**Après (Axum::ws):**
```rust
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // Heartbeat task
    let heartbeat_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            // Send heartbeat
        }
    });
    
    // Main loop
    loop {
        tokio::select! {
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Ping(ping)) => {
                        socket.send(Message::Pong(ping)).await.ok();
                    }
                    // ...
                }
            }
        }
    }
}
```

---

## 📊 Endpoints disponibles

### Core
- `GET /` - Interface web
- `GET /health` - Health check
- `GET /api` - API info
- `GET /api/config` - Configuration
- `GET /api/cache/stats` - Statistiques cache
- `POST /api/slack/notify` - Notification Slack
- `GET /api/ws/notifications` - WebSocket notifications

### Fichiers statiques
- `GET /static/*` - Fichiers statiques

---

## 🚀 Build & Exécution

```bash
# Build release
cargo build --release

# Exécuter
./target/release/kusanagi

# Ou avec cargo
cargo run --release
```

Variables d'environnement:
- `KUSANAGI_HOST` - Host d'écoute (défaut: 0.0.0.0)
- `KUSANAGI_PORT` - Port d'écoute (défaut: 8080)

---

## 📈 Résultats

| Métrique | Avant | Après |
|----------|-------|-------|
| Framework HTTP | Actix-web 4.12 | Axum 0.7 |
| WebSocket | actix-web-actors | axum::ws natif |
| Middleware | Actix natif | Tower |
| Gestion d'état | web::Data<T> | State<T> |
| Compilation | ✅ | ✅ |
| Warnings | Nombreux | 2 (mineurs) |

---

## 📝 Notes

- L'architecture hexagonale est préservée (Domaine → Application → Infrastructure → Interfaces)
- Les use cases sont maintenant partagés via `Arc<T>` dans `AppState`
- Les middlewares Tower (CORS, compression, tracing) remplacent ceux d'Actix
- Le WebSocket est maintenant géré de manière native avec `tokio::select!`

---

**Migration terminée avec succès !** ✅
