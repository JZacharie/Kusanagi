# Kusanagi - Middleware & Diagnostic Improvements

## 📋 Résumé

Ce document décrit les nouvelles améliorations apportées à Kusanagi :
- Middleware de logging structuré avec Correlation IDs
- Rate Limiting
- Doctor - Outil d'auto-diagnostic

---

## ✅ Nouvelles Fonctionnalités

### 1. Structured Logging Middleware avec Correlation IDs

**Fichier** : `src/middleware/logging.rs`

#### Description
Chaque requête reçoit un identifiant unique de corrélation (UUID v4) qui permet de tracer les requêtes à travers les logs.

#### Headers
- `X-Correlation-Id` : Identifiant de corrélation (généré ou reçu du client)
- `X-Request-Id` : Identifiant unique de la requête

#### Exemple de logs
```json
{
  "timestamp": "2026-02-02T12:00:00Z",
  "level": "INFO",
  "target": "http_request_start",
  "fields": {
    "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
    "request_id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "method": "GET",
    "path": "/api/pods/status",
    "remote_addr": "10.0.0.1",
    "user_agent": "Mozilla/5.0..."
  },
  "message": "→ Request started"
}
```

#### Utilisation
Le middleware est automatiquement appliqué à toutes les routes dans `main.rs` :
```rust
App::new()
    .wrap(middleware::StructuredLogging::new())
```

#### Récupérer le Correlation ID dans un handler
```rust
use crate::middleware::get_correlation_id;

pub async fn my_handler(req: HttpRequest) {
    if let Some(cid) = get_correlation_id(&req) {
        info!(correlation_id = %cid, "Processing request");
    }
}
```

---

### 2. Rate Limiting Middleware

**Fichier** : `src/middleware/rate_limit.rs`

#### Description
Protection contre les abus d'API avec limitation du nombre de requêtes par client.

#### Configuration par défaut
- **Limite** : 1000 requêtes par minute
- **Stockage** : In-memory (HashMap)
- **Clé** : Adresse IP du client

#### Headers de réponse
- `X-RateLimit-Limit` : Limite maximale
- `X-RateLimit-Remaining` : Requêtes restantes
- `Retry-After` : Secondes avant réessayage (si limité)

#### Codes de statut
- `429 Too Many Requests` : Limite atteinte

#### Utilisation dans main.rs
```rust
let rate_limiter = middleware::RateLimiter::per_minute(1000);

HttpServer::new(move || {
    App::new()
        .wrap(rate_limiter.clone())
        // ... routes
})
```

#### Configuration personnalisée
```rust
use middleware::rate_limit::{RateLimiter, RateLimitConfig, KeyExtractor};
use std::time::Duration;

let config = RateLimitConfig {
    max_requests: 100,        // 100 requêtes
    window: Duration::from_secs(60),  // par minute
    key_extractor: KeyExtractor::Ip,  // par IP
};

let limiter = RateLimiter::new(config);
```

#### Key Extractors disponibles
- `KeyExtractor::Ip` : Par adresse IP (défaut)
- `KeyExtractor::Header("x-api-key")` : Par header personnalisé
- `KeyExtractor::IpAndPath` : Par IP + chemin

---

### 3. Doctor - Outil d'Auto-Diagnostic

**Fichier** : `src/doctor.rs`

#### Description
Endpoint complet pour diagnostiquer l'état de santé de l'application et de ses dépendances.

#### Endpoints

##### Diagnostic complet
```
GET /api/doctor
```

Retourne un rapport complet avec :
- Connexion Kubernetes
- Permissions RBAC
- Connexion Prometheus
- Configuration OpenObserve
- Connexion Database
- Connexion MQTT
- Configuration LLM
- Connexion S3/MinIO
- Utilisation mémoire
- Espace disque

##### Diagnostic rapide
```
GET /api/doctor/quick
```

Retourne un statut simplifié :
```json
{
  "healthy": true,
  "kubernetes": true,
  "permissions": true,
  "duration_ms": 150
}
```

#### Exemple de réponse complète
```json
{
  "overall_status": "ok",
  "timestamp": "2026-02-02T12:00:00Z",
  "version": "0.3.0",
  "checks": [
    {
      "name": "Kubernetes Connection",
      "status": "ok",
      "message": "Connected to Kubernetes v1.28.0",
      "details": "Major: 1, Minor: 28",
      "recommendation": null,
      "duration_ms": 45
    },
    {
      "name": "OpenObserve Telemetry",
      "status": "warning",
      "message": "OpenObserve telemetry not configured",
      "details": "Create secret 'openobserve-credentials'...",
      "recommendation": "Run: kubectl create secret generic...",
      "duration_ms": 5
    }
  ],
  "summary": {
    "total": 10,
    "ok": 8,
    "warning": 2,
    "error": 0,
    "skipped": 0
  },
  "recommendations": [
    "[OpenObserve Telemetry] Run: kubectl create secret..."
  ]
}
```

#### Statuts possibles
- `ok` : Tout fonctionne correctement
- `warning` : Fonctionne mais pourrait être amélioré
- `error` : Problème détecté
- `skipped` : Vérification ignorée (optionnel)

#### Codes HTTP de réponse
- `200 OK` : Tout va bien (même avec des warnings)
- `503 Service Unavailable` : Au moins une erreur critique

---

## 📦 Structure des fichiers

```
src/
├── middleware/
│   ├── mod.rs           # Export des middlewares
│   ├── logging.rs       # Structured logging + Correlation ID
│   └── rate_limit.rs    # Rate limiting
├── doctor.rs            # Outil de diagnostic
└── main.rs              # Intégration des middlewares
```

---

## 🔧 Configuration

### Variables d'environnement pour le logging
```bash
# Format des logs
RUST_LOG=info,kusanagi=debug

# Ou dans kusanagi.toml
[log]
level = "info"
format = "json"  # ou "pretty"
```

### Configuration Rate Limiting
```bash
# Par défaut: 1000 req/min
# Pour modifier, éditer main.rs ou utiliser Config
```

---

## 🧪 Tests

### Tester le Correlation ID
```bash
# Envoyer une requête avec un Correlation ID existant
curl -H "X-Correlation-Id: my-custom-id" http://localhost:8080/api/health/full

# Vérifier qu'il est retourné dans la réponse
curl -v http://localhost:8080/api/health/full 2>&1 | grep X-Correlation-Id
```

### Tester le Rate Limiting
```bash
# Envoyer 100 requêtes rapidement
for i in {1..100}; do curl -s http://localhost:8080/api/health/full; done

# La 101ème devrait retourner 429
curl -v http://localhost:8080/api/health/full
```

### Tester le Doctor
```bash
# Diagnostic complet
curl http://localhost:8080/api/doctor | jq

# Diagnostic rapide
curl http://localhost:8080/api/doctor/quick | jq
```

---

## 📊 Monitoring

### Logs structurés avec correlation_id
Les logs incluent automatiquement le `correlation_id` pour permettre le tracing distribué :

```bash
# Filtrer les logs par correlation_id
kubectl logs -n kusanagi deployment/kusanagi | grep "550e8400-e29b-41d4-a716-446655440000"
```

### Alertes sur les erreurs
Les erreurs serveur (5xx) sont automatiquement loguées avec level ERROR :
```
target=http_request_server_error, status=500, "← Server error"
```

---

## 🚀 Prochaines améliorations

- [ ] **Redis backend** pour le rate limiting (multi-instance)
- [ ] **Authentication middleware** avec JWT
- [ ] **CORS middleware** configurables
- [ ] **Compression middleware** (gzip/brotli)
- [ ] **Cache middleware** pour les réponses

---

**Date** : 2026-02-02  
**Version** : 0.3.0  
**Auteur** : AI Assistant
