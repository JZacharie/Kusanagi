# Kusanagi - Agent Context

## TL;DR
Kusanagi v0.3.0 is a Rust-based Kubernetes monitoring platform using Hexagonal Architecture. Backend: Axum + Tower, Frontend: Vanilla JS PWA. Key patterns: Repository pattern, Use Cases, impl IntoResponse handlers.

## Architecture Overview

### Backend (Rust)
```
src/
├── application/use_cases/     # Business logic
│   ├── GetWeatherUseCase      # Weather operations
│   ├── GetAlertsUseCase       # Alert management
│   ├── GetSecurityUseCase     # Security reports
│   └── BackupUseCase          # Backup operations
├── domain/
│   ├── entities/              # Data models (WeatherInfo, Alert, etc.)
│   ├── ports/                 # Traits (WeatherRepository, AlertRepository)
│   └── services/              # Domain logic
├── infrastructure/repositories/ # Implementations
│   ├── weather_repository.rs  # Open-Meteo API
│   ├── alert_repository.rs    # Alertmanager integration
│   └── security_repository.rs # Trivy reports
└── interfaces/http/           # HTTP handlers
    ├── weather_handlers.rs    # GET /api/weather/current
    ├── homeassistant_handlers.rs
    ├── security_handlers.rs
    └── alert_handlers.rs
```

### Frontend (JS)
```
static/js/
├── k8s.js                     # K8s operations, cache logic
├── dashboard.js               # Core dashboard
├── monitors.js                # Monitoring display
├── weather.js                 # Weather widget
└── core.js                    # Tab management
```

## Critical Patterns

### 1. Handler Signature (MANDATORY)
```rust
// CORRECT - Always use impl Responder
pub async fn handler(use_case: web::Data<T>) -> impl Responder {
    match use_case.execute().await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => HttpResponse::Ok().json(json!({"error": e.to_string()}))
    }
}
```

### 2. Cache Pattern (Services/Ingress)
```rust
const TTL: Duration = Duration::from_secs(180); // 3 min

// Check cache first
if let Some(cached) = cache.get("key").await {
    return HttpResponse::Ok().body(cached);
}
// Fetch and cache with TTL
cache.set("key", json, Some(TTL)).await;
```

### 3. Frontend Cache Pattern
```javascript
const Manager = {
    lastFetch: 0,
    TTL: 180000, // 3 min
    
    async fetch() {
        const now = Date.now();
        if (this.lastFetch !== 0) {
            // Only fetch if active tab AND TTL expired
            if (activeTab !== 'tab') return;
            if (now - this.lastFetch < this.TTL) return;
        }
        this.lastFetch = now;
        // API call
    }
};
```

## API Endpoints

| Endpoint | Method | Handler | Cache |
|----------|--------|---------|-------|
| /api/weather/current | GET | get_weather_handler | 1h |
| /api/ha/devices | GET | get_devices_handler | None |
| /api/ha/sensors | GET | get_sensors_handler | None |
| /api/security/vulnerabilities | GET | get_vulnerabilities_handler | None |
| /api/services | GET | services | 3min |
| /api/ingress | GET | ingress | 3min |
| /api/alerts | GET | get_alerts_handler | 1min |

## Common Fixes

### 500 Errors on Handlers
Change: `-> HttpResponse` or `-> Result<HttpResponse>`  
To: `-> impl Responder`

### SSL/TLS Errors (Slack)
Add to Dockerfile:
```dockerfile
RUN apt-get install -y ca-certificates && update-ca-certificates
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
```

### Cache Not Refreshing
- Check TTL constant (180s for services/ingress)
- Verify `lastFetch` timestamp reset
- Check active tab condition

## Code Style

### Error Handling
```rust
use crate::error::{KusanagiError, Result};
// Use KusanagiError::external_service() for external API errors
// Use KusanagiError::configuration() for config errors
```

### Logging
```rust
use tracing::{debug, error, info, warn};
debug!("Variable: {}", var);
error!("Failed: {}", e);
```

### Adding New Endpoint
1. Create handler in `src/interfaces/http/{module}_handlers.rs`
2. Use `-> impl Responder`
3. Import in `main.rs`
4. Add route: `.route("/path", web::get().to(handler))`
5. Add use_case to App::new() if needed

## Testing
```bash
cargo test --release
cargo clippy --release
```

## Quick Commands
```bash
# Build
cargo build --release

# Docker
docker build -t kusanagi:latest .

# Deploy
./deploy.sh
```

## Key Dependencies
- `actix-web`: Web framework
- `kube`: Kubernetes API
- `reqwest`: HTTP client (native-tls)
- `tokio`: Async runtime
- `serde`: Serialization
