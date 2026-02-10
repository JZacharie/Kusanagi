# Changelog

## [0.3.0] - 2026-02-10

### 🚀 Major Changes
- **Framework Migration**: Complete migration from Actix-web 4.12 to Axum 0.7
- **WebSocket Upgrade**: Migrated from actix-web-actors to native axum::ws
- **Middleware Stack**: Replaced Actix middleware with Tower middleware (CORS, compression, tracing)

### 🔧 Technical Details

#### Architecture
- **State Management**: Centralized `AppState` with all use cases wrapped in `Arc<T>`
- **Handler Pattern**: `State<AppState>` extractor → `impl IntoResponse`
- **Routing**: Axum `Router` with `.route()` and `.nest_service()`

#### Dependencies
```toml
# Added
axum = { version = "0.7", features = ["ws"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors", "compression-gzip", "trace"] }
tracing-subscriber = "0.3"

# Removed
actix-web = "4.12"
actix-files = "0.6"
actix-web-actors = "4.3"
```

#### Files Changed
- `src/main.rs` - Complete rewrite for Axum entry point
- `src/state.rs` - New centralized state management
- `src/routes.rs` - Axum router configuration
- `src/api_handlers/` - New Axum-compatible handlers
- `src/interfaces/http/` - Migrated hexagonal handlers

### 📁 New Files
- `src/api_handlers/websocket.rs` - Native Axum WebSocket implementation
- `AGENTS.md` - Developer guide for agents
- `SKILL.md` - Quick reference skill file
- `MIGRATION_AXUM_SUMMARY.md` - Complete migration documentation

### ✅ Endpoints
All endpoints preserved and migrated:
- `GET /` - Web interface
- `GET /health` - Health check
- `GET /api` - API info
- `GET /api/config` - Configuration
- `GET /api/cache/stats` - Cache statistics
- `POST /api/slack/notify` - Slack notifications
- `GET /api/ws/notifications` - WebSocket endpoint

### 📊 Performance
- Build time: ~22s (release)
- Binary size: ~36MB
- Warnings: 2 minor (down from 20+)

---

## [0.2.0] - Previous Version

### Features
- Hexagonal Architecture (Domain/Application/Infrastructure/Interfaces)
- Kubernetes monitoring (462 pods, 16 nodes, 447 services)
- ArgoCD integration (183 applications)
- Trivy security scanning
- Proxmox integration
- Home Assistant integration
- MQTT support
- WebSocket notifications (actix-web-actors)
