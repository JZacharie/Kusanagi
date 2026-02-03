# Guide Pratique : Migration vers Architecture Hexagonale

> **Audience**: Développeurs Rust travaillant sur Kusanagi  
> **Prérequis**: Connaissance de l'architecture hexagonale (lire ADR-001)

---

## 🚀 Quick Start : Migrer un module en 30 minutes

Ce guide vous accompagne pas à pas pour migrer un module legacy vers l'architecture hexagonale.

### Exemple concret : Migration du module "Health"

---

## Étape 1 : Analyse (5 min)

### 1.1 Identifier le fichier legacy
```bash
ls src/legacy/health.rs
# Ou
ls src/legacy/health/
```

### 1.2 Identifier les dépendances externes
```rust
// Dans src/legacy/health.rs, chercher:
use kube::...;          // → KubernetesRepository
use sqlx::...;          // → DatabaseRepository  
use reqwest::...;       // → HttpClient
use std::process::...;  // → SystemService
```

### 1.3 Identifier les structs métiers
```rust
// Quelles structures représentent des concepts métier?
pub struct HealthCheck { ... }
pub struct SystemStatus { ... }
pub enum HealthStatus { Healthy, Degraded, Unhealthy }
```

---

## Étape 2 : Domain Layer (10 min)

### 2.1 Créer l'entité
```bash
touch src/domain/entities/health.rs
```

```rust
// src/domain/entities/health.rs
//! Health check entities

use serde::{Deserialize, Serialize};

/// Overall system health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemHealth {
    pub status: HealthStatus,
    pub checks: Vec<HealthCheck>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl SystemHealth {
    pub fn overall(checks: Vec<HealthCheck>) -> Self {
        let status = if checks.iter().any(|c| c.status == HealthStatus::Unhealthy) {
            HealthStatus::Unhealthy
        } else if checks.iter().any(|c| c.status == HealthStatus::Degraded) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        Self {
            status,
            checks,
            timestamp: chrono::Utc::now(),
        }
    }
}
```

### 2.2 Créer le port (interface)
```bash
touch src/domain/ports/health_port.rs
```

```rust
// src/domain/ports/health_port.rs
//! Health check repository port

use crate::domain::entities::health::{HealthCheck, SystemHealth};
use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Port for health check operations
#[async_trait]
pub trait HealthRepository: Send + Sync {
    /// Perform a specific health check
    async fn check(&self, name: &str) -> Result<HealthCheck>;
    
    /// Get all available check names
    fn available_checks(&self) -> Vec<String>;
}

/// Port for system-level health operations
#[async_trait]
pub trait SystemHealthService: Send + Sync {
    /// Get overall system health
    async fn get_system_health(&self) -> Result<SystemHealth>;
    
    /// Check if system is ready to serve traffic
    async fn is_ready(&self) -> Result<bool>;
    
    /// Check if system is alive
    async fn is_alive(&self) -> Result<bool>;
}
```

### 2.3 Exporter dans mod.rs
```rust
// src/domain/entities/mod.rs
pub mod health;
pub use health::*;

// src/domain/ports/mod.rs
pub mod health_port;
pub use health_port::*;
```

---

## Étape 3 : Application Layer (5 min)

### 3.1 Créer le use case
```bash
touch src/application/use_cases/health_use_cases.rs
```

```rust
// src/application/use_cases/health_use_cases.rs
//! Health check use cases

use crate::domain::entities::health::SystemHealth;
use crate::domain::ports::health_port::SystemHealthService;
use crate::error::Result;
use std::sync::Arc;

/// Use case: Get system health overview
pub struct GetSystemHealthUseCase {
    health_service: Arc<dyn SystemHealthService>,
}

impl GetSystemHealthUseCase {
    pub fn new(health_service: Arc<dyn SystemHealthService>) -> Self {
        Self { health_service }
    }

    pub async fn execute(&self) -> Result<SystemHealth> {
        self.health_service.get_system_health().await
    }
}

/// Use case: Check if system is ready
pub struct CheckReadinessUseCase {
    health_service: Arc<dyn SystemHealthService>,
}

impl CheckReadinessUseCase {
    pub fn new(health_service: Arc<dyn SystemHealthService>) -> Self {
        Self { health_service }
    }

    pub async fn execute(&self) -> Result<bool> {
        self.health_service.is_ready().await
    }
}

/// Use case: Check if system is alive (liveness probe)
pub struct CheckLivenessUseCase {
    health_service: Arc<dyn SystemHealthService>,
}

impl CheckLivenessUseCase {
    pub fn new(health_service: Arc<dyn SystemHealthService>) -> Self {
        Self { health_service }
    }

    pub async fn execute(&self) -> Result<bool> {
        self.health_service.is_alive().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::health::{HealthCheck, HealthStatus};
    use async_trait::async_trait;

    struct MockHealthService;

    #[async_trait]
    impl SystemHealthService for MockHealthService {
        async fn get_system_health(&self) -> Result<SystemHealth> {
            Ok(SystemHealth::overall(vec![
                HealthCheck {
                    name: "database".to_string(),
                    status: HealthStatus::Healthy,
                    message: None,
                    duration_ms: 10,
                }
            ]))
        }

        async fn is_ready(&self) -> Result<bool> {
            Ok(true)
        }

        async fn is_alive(&self) -> Result<bool> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_get_system_health() {
        let mock = Arc::new(MockHealthService);
        let use_case = GetSystemHealthUseCase::new(mock);
        
        let health = use_case.execute().await.unwrap();
        
        assert!(matches!(health.status, HealthStatus::Healthy));
        assert_eq!(health.checks.len(), 1);
    }
}
```

### 3.2 Exporter
```rust
// src/application/use_cases/mod.rs
pub mod health_use_cases;
pub use health_use_cases::*;
```

---

## Étape 4 : Infrastructure Layer (7 min)

### 4.1 Implémenter le repository
```bash
touch src/infrastructure/repositories/health_repository.rs
```

```rust
// src/infrastructure/repositories/health_repository.rs
//! Health check repository implementation

use crate::domain::entities::health::{HealthCheck, HealthStatus, SystemHealth};
use crate::domain::ports::health_port::{HealthRepository, SystemHealthService};
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
use std::time::Instant;

/// Health service implementation
pub struct HealthServiceImpl {
    repositories: Vec<Box<dyn HealthRepository>>,
}

impl HealthServiceImpl {
    pub fn new(repositories: Vec<Box<dyn HealthRepository>>) -> Self {
        Self { repositories }
    }
}

#[async_trait]
impl SystemHealthService for HealthServiceImpl {
    async fn get_system_health(&self) -> Result<SystemHealth> {
        let mut checks = Vec::new();

        for repo in &self.repositories {
            for check_name in repo.available_checks() {
                match repo.check(&check_name).await {
                    Ok(check) => checks.push(check),
                    Err(e) => checks.push(HealthCheck {
                        name: check_name,
                        status: HealthStatus::Unhealthy,
                        message: Some(e.to_string()),
                        duration_ms: 0,
                    }),
                }
            }
        }

        Ok(SystemHealth::overall(checks))
    }

    async fn is_ready(&self) -> Result<bool> {
        // Ready if all critical checks pass
        let health = self.get_system_health().await?;
        Ok(!matches!(health.status, HealthStatus::Unhealthy))
    }

    async fn is_alive(&self) -> Result<bool> {
        // Alive if we can respond (basic check)
        Ok(true)
    }
}

/// Database health check repository
pub struct DatabaseHealthRepository {
    // pool: sqlx::PgPool, // Injected dependency
}

impl DatabaseHealthRepository {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl HealthRepository for DatabaseHealthRepository {
    async fn check(&self, name: &str) -> Result<HealthCheck> {
        let start = Instant::now();
        
        // Implementation: check DB connectivity
        // let result = sqlx::query("SELECT 1").fetch_one(&self.pool).await;
        
        // Simulated for example
        let is_healthy = true;
        
        Ok(HealthCheck {
            name: name.to_string(),
            status: if is_healthy { HealthStatus::Healthy } else { HealthStatus::Unhealthy },
            message: None,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn available_checks(&self) -> Vec<String> {
        vec!["database".to_string()]
    }
}

/// Kubernetes health check repository
pub struct KubernetesHealthRepository {
    // client: kube::Client,
}

#[async_trait]
impl HealthRepository for KubernetesHealthRepository {
    async fn check(&self, name: &str) -> Result<HealthCheck> {
        let start = Instant::now();
        
        // Check K8s API connectivity
        
        Ok(HealthCheck {
            name: name.to_string(),
            status: HealthStatus::Healthy,
            message: None,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    fn available_checks(&self) -> Vec<String> {
        vec!["kubernetes".to_string()]
    }
}
```

### 4.2 Exporter
```rust
// src/infrastructure/repositories/mod.rs
pub mod health_repository;
pub use health_repository::*;
```

---

## Étape 5 : Interface Layer (3 min)

### 5.1 Créer les handlers
```bash
touch src/interfaces/http/health_handlers.rs
```

```rust
// src/interfaces/http/health_handlers.rs
//! Health check HTTP handlers

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use std::sync::Arc;

use crate::application::use_cases::{
    CheckLivenessUseCase, CheckReadinessUseCase, GetSystemHealthUseCase,
};
use crate::domain::ports::health_port::SystemHealthService;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    checks: Vec<CheckDetail>,
    timestamp: String,
}

#[derive(Serialize)]
struct CheckDetail {
    name: String,
    status: String,
    message: Option<String>,
    duration_ms: u64,
}

#[derive(Serialize)]
struct SimpleStatus {
    status: String,
}

/// GET /health - Detailed health status
#[get("/health")]
pub async fn health_check(
    health_service: web::Data<Arc<dyn SystemHealthService>>,
) -> impl Responder {
    let use_case = GetSystemHealthUseCase::new(health_service.into_inner());
    
    match use_case.execute().await {
        Ok(health) => {
            let response = HealthResponse {
                status: format!("{:?}", health.status),
                checks: health.checks.into_iter().map(|c| CheckDetail {
                    name: c.name,
                    status: format!("{:?}", c.status),
                    message: c.message,
                    duration_ms: c.duration_ms,
                }).collect(),
                timestamp: health.timestamp.to_rfc3339(),
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            HttpResponse::ServiceUnavailable().json(SimpleStatus {
                status: format!("Error: {}", e),
            })
        }
    }
}

/// GET /ready - Kubernetes readiness probe
#[get("/ready")]
pub async fn readiness_check(
    health_service: web::Data<Arc<dyn SystemHealthService>>,
) -> impl Responder {
    let use_case = CheckReadinessUseCase::new(health_service.into_inner());
    
    match use_case.execute().await {
        Ok(true) => HttpResponse::Ok().json(SimpleStatus { 
            status: "ready".to_string() 
        }),
        Ok(false) => HttpResponse::ServiceUnavailable().json(SimpleStatus { 
            status: "not ready".to_string() 
        }),
        Err(_) => HttpResponse::ServiceUnavailable().json(SimpleStatus { 
            status: "error".to_string() 
        }),
    }
}

/// GET /live - Kubernetes liveness probe
#[get("/live")]
pub async fn liveness_check(
    health_service: web::Data<Arc<dyn SystemHealthService>>,
) -> impl Responder {
    let use_case = CheckLivenessUseCase::new(health_service.into_inner());
    
    match use_case.execute().await {
        Ok(true) => HttpResponse::Ok().json(SimpleStatus { 
            status: "alive".to_string() 
        }),
        _ => HttpResponse::ServiceUnavailable().json(SimpleStatus { 
            status: "dead".to_string() 
        }),
    }
}
```

### 5.2 Exporter
```rust
// src/interfaces/http/mod.rs
pub mod health_handlers;
pub use health_handlers::*;
```

---

## Étape 6 : Wiring (Intégration)

### 6.1 Créer la factory/dependency injection
```rust
// src/main.rs ou src/app.rs
use crate::domain::ports::health_port::SystemHealthService;
use crate::infrastructure::repositories::health_repository::{
    DatabaseHealthRepository, HealthServiceImpl, KubernetesHealthRepository,
};

fn create_health_service() -> Arc<dyn SystemHealthService> {
    let repositories: Vec<Box<dyn HealthRepository>> = vec![
        Box::new(DatabaseHealthRepository::new()),
        Box::new(KubernetesHealthRepository::new()),
    ];
    
    Arc::new(HealthServiceImpl::new(repositories))
}

// Dans AppState
pub struct AppState {
    pub health_service: Arc<dyn SystemHealthService>,
    // ... autres champs
}

// Configuration des routes
async fn main() -> std::io::Result<()> {
    let health_service = create_health_service();
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(health_service.clone()))
            .service(health_check)
            .service(readiness_check)
            .service(liveness_check)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
```

---

## ✅ Checklist de Validation

Avant de marquer la migration comme terminée :

- [ ] **Compilation** : `cargo build` passe sans erreurs
- [ ] **Tests** : `cargo test` passe (unitaires + intégration)
- [ ] **Lint** : `cargo clippy` 0 warnings sur le nouveau code
- [ ] **Format** : `cargo fmt` passé
- [ ] **Documentation** : Commentaires rustdoc sur les pub items
- [ ] **API Docs** : OpenAPI mis à jour (si endpoints HTTP)

---

## 🧪 Pattern: Testing avec Mocks

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::health::{HealthCheck, HealthStatus};
    use crate::domain::ports::health_port::{HealthRepository, SystemHealthService};
    use async_trait::async_trait;

    // Mock simple
    struct MockHealthRepository {
        should_fail: bool,
    }

    #[async_trait]
    impl HealthRepository for MockHealthRepository {
        async fn check(&self, name: &str) -> Result<HealthCheck> {
            if self.should_fail {
                Err(KusanagiError::internal("DB error"))
            } else {
                Ok(HealthCheck {
                    name: name.to_string(),
                    status: HealthStatus::Healthy,
                    message: None,
                    duration_ms: 10,
                })
            }
        }

        fn available_checks(&self) -> Vec<String> {
            vec!["test".to_string()]
        }
    }

    #[tokio::test]
    async fn test_health_service_with_mock() {
        let mock = MockHealthRepository { should_fail: false };
        let service = HealthServiceImpl::new(vec![Box::new(mock)]);
        
        let health = service.get_system_health().await.unwrap();
        
        assert!(matches!(health.status, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_health_service_with_failure() {
        let mock = MockHealthRepository { should_fail: true };
        let service = HealthServiceImpl::new(vec![Box::new(mock)]);
        
        let health = service.get_system_health().await.unwrap();
        
        assert!(matches!(health.status, HealthStatus::Unhealthy));
    }
}
```

---

## 🔥 Anti-Patterns à Éviter

### ❌ Le "Fat Use Case"
```rust
// Mauvais: Use case avec trop de logique
impl SomeUseCase {
    async fn execute(&self) -> Result<()> {
        // 100 lignes de logique métier + appels DB + HTTP
        // → Difficile à tester, à comprendre
    }
}
```

### ✅ Solution : Services métier
```rust
// Bon: Déléguer à des services du domaine
impl SomeUseCase {
    async fn execute(&self) -> Result<()> {
        let data = self.repo.fetch().await?;
        let processed = self.domain_service.process(data)?;
        self.repo.save(processed).await
    }
}
```

### ❌ Le "Leaky Abstraction"
```rust
// Mauvais: Exposer les types infra dans le domaine
pub struct Pod {
    pub kube_pod: k8s_openapi::api::core::v1::Pod,  // ❌
}
```

### ✅ Solution : Mapping propre
```rust
// Bon: Types purs métier
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub status: PodStatus,  // Enum métier, pas k8s::PodStatus
}
```

---

## 📚 Ressources

- [Architecture Hexagonale - Alistair Cockburn](https://alistair.cockburn.us/hexagonal-architecture/)
- [Rust by Example - Traits](https://doc.rust-lang.org/rust-by-example/trait.html)
- [Actix Web Documentation](https://actix.rs/docs/)

---

**Prochaine étape** : Lire `docs/ADR-001-hexagonal-refactoring-strategy.md` pour le contexte stratégique.
