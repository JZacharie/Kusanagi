# Hexagonal Architecture (Clean Architecture)

## Overview

The codebase now follows Hexagonal Architecture (also known as Clean Architecture), which separates concerns into distinct layers with clear dependencies.

## Architecture Diagram

```
                    ┌─────────────────────────────────────────┐
                    │           External World                │
                    │  (Web, CLI, External APIs, DB, etc.)   │
                    └─────────────────┬───────────────────────┘
                                      │
                    ┌─────────────────▼───────────────────────┐
                    │     Interface Layer (Adapters)          │
                    │  ┌──────────────┐  ┌─────────────────┐  │
                    │  │ HTTP Handlers│  │ WebSocket        │  │
                    │  │ - REST API   │  │ - Real-time      │  │
                    │  │ - Routes     │  │   updates        │  │
                    │  └──────────────┘  └─────────────────┘  │
                    └─────────────────┬───────────────────────┘
                                      │
                    ┌─────────────────▼───────────────────────┐
                    │      Application Layer (Use Cases)      │
                    │  ┌──────────────┐  ┌─────────────────┐  │
                    │  │ Use Cases    │  │ DTOs            │  │
                    │  │ - GetCluster │  │ - Request/      │  │
                    │  │ - ListPods   │  │   Response      │  │
                    │  │ - RestartPod │  │ - Mappers       │  │
                    │  └──────────────┘  └─────────────────┘  │
                    └─────────────────┬───────────────────────┘
                                      │
                    ┌─────────────────▼───────────────────────┐
                    │        Domain Layer (Core)              │
                    │  ┌──────────────┐  ┌─────────────────┐  │
                    │  │ Entities     │  │ Ports           │  │
                    │  │ - Cluster    │  │ (Interfaces)    │  │
                    │  │ - Node       │  │ - Kubernetes    │  │
                    │  │ - Pod        │  │ - Metrics       │  │
                    │  │ - Service    │  │ - Cache         │  │
                    │  └──────────────┘  └─────────────────┘  │
                    │  ┌──────────────┐                       │
                    │  │ Services     │                       │
                    │  │ - Cluster    │                       │
                    │  │ - Pod        │                       │
                    │  │ - Event      │                       │
                    │  └──────────────┘                       │
                    └─────────────────┬───────────────────────┘
                                      │
                    ┌─────────────────▼───────────────────────┐
                    │   Infrastructure Layer (Adapters)       │
                    │  ┌──────────────┐  ┌─────────────────┐  │
                    │  │ Repositories │  │ Clients         │  │
                    │  │ - K8sRepo    │  │ - Prometheus    │  │
                    │  │ - Prometheus │  │ - Alertmanager  │  │
                    │  │   Repo       │  │ - External APIs │  │
                    │  └──────────────┘  └─────────────────┘  │
                    └─────────────────────────────────────────┘
```

## Layer Responsibilities

### 1. Domain Layer (`src/domain/`)

**Contains:**
- **Entities**: Business objects (Pod, Node, Cluster, etc.)
- **Ports**: Interfaces that define what the domain needs (traits)
- **Services**: Business logic and domain operations

**Principles:**
- No external dependencies
- Pure business logic
- Framework-agnostic

**Example:**
```rust
// Domain entity
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub status: PodStatus,
    // ...
}

// Domain port (interface)
#[async_trait]
pub trait KubernetesRepository: Send + Sync {
    async fn get_pod(&self, namespace: &str, name: &str) -> Result<Pod>;
    // ...
}

// Domain service
pub struct PodService {
    k8s_repo: Arc<dyn KubernetesRepository>,
}
```

### 2. Application Layer (`src/application/`)

**Contains:**
- **Use Cases**: Application-specific operations
- **DTOs**: Data Transfer Objects for input/output
- **Mappers**: Conversions between domain and DTOs

**Principles:**
- Orchestrates domain services
- No business logic (only coordination)
- Handles transactions

**Example:**
```rust
// Use case
pub struct GetPodDetailsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetPodDetailsUseCase {
    pub async fn execute(&self, namespace: &str, name: &str) -> Result<PodDetailsDto> {
        let pod = self.k8s_repo.get_pod(namespace, name).await?;
        let logs = self.k8s_repo.get_pod_logs(namespace, name, None, 100).await.ok();
        
        Ok(PodDetailsDto {
            name: pod.name,
            namespace: pod.namespace,
            // ...
        })
    }
}
```

### 3. Infrastructure Layer (`src/infrastructure/`)

**Contains:**
- **Repositories**: Concrete implementations of domain ports
- **Clients**: External API clients

**Principles:**
- Implements domain ports
- Uses external frameworks (kube-rs, reqwest, etc.)
- Handles external concerns

**Example:**
```rust
pub struct K8sRepository {
    client: Client,
}

#[async_trait]
impl KubernetesRepository for K8sRepository {
    async fn get_pod(&self, namespace: &str, name: &str) -> Result<Pod> {
        let pods: Api<k8s_openapi::api::core::v1::Pod> = 
            Api::namespaced(self.client.clone(), namespace);
        let pod = pods.get(name).await?;
        
        // Convert k8s-openapi Pod to domain Pod
        Ok(Pod {
            name: pod.metadata.name.unwrap_or_default(),
            // ...
        })
    }
}
```

### 4. Interface Layer (`src/interfaces/`)

**Contains:**
- **HTTP Handlers**: REST API endpoints
- **WebSocket Handlers**: Real-time communication
- **Middleware**: Cross-cutting concerns

**Principles:**
- Handles HTTP concerns
- No business logic
- Uses application layer

**Example:**
```rust
#[get("/api/pods/{namespace}/{name}")]
async fn get_pod_details(
    data: web::Data<AppState>,
    path: web::Path<GetPodPath>,
) -> impl Responder {
    let use_case = GetPodDetailsUseCase::new(Arc::clone(&data.k8s_repo));
    
    match use_case.execute(&path.namespace, &path.name).await {
        Ok(dto) => HttpResponse::Ok().json(dto),
        Err(e) => e.error_response(),
    }
}
```

## Dependency Rule

```
Domain ← Application ← Infrastructure
  ↑                      ↑
  └────── Interface ←────┘
```

Dependencies point **inward**:
- Domain knows nothing about other layers
- Application depends only on Domain
- Infrastructure implements Domain ports
- Interface uses Application layer

## Benefits

1. **Testability**: Domain logic can be tested without external dependencies
2. **Flexibility**: Swap infrastructure (e.g., mock vs real Kubernetes)
3. **Maintainability**: Changes in one layer don't affect others
4. **Clarity**: Clear separation of concerns

## Example: Adding a New Feature

Let's say we want to add a "Get Pod Logs" feature:

### 1. Define Domain Port
```rust
// src/domain/ports/mod.rs
#[async_trait]
pub trait KubernetesRepository: Send + Sync {
    async fn get_pod_logs(&self, namespace: &str, name: &str, 
                         container: Option<&str>, tail: i64) -> Result<String>;
}
```

### 2. Implement Infrastructure
```rust
// src/infrastructure/repositories/mod.rs
#[async_trait]
impl KubernetesRepository for K8sRepository {
    async fn get_pod_logs(&self, namespace: &str, name: &str,
                         container: Option<&str>, tail: i64) -> Result<String> {
        let pods: Api<Pod> = Api::namespaced(self.client.clone(), namespace);
        let params = LogParams {
            container: container.map(|c| c.to_string()),
            tail_lines: Some(tail),
            ..Default::default()
        };
        pods.logs(name, &params).await.map_err(|e| e.into())
    }
}
```

### 3. Create Use Case
```rust
// src/application/use_cases/pod_use_cases.rs
pub struct GetPodLogsUseCase {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl GetPodLogsUseCase {
    pub async fn execute(&self, req: PodLogsRequestDto) -> Result<String> {
        self.k8s_repo.get_pod_logs(
            &req.namespace,
            &req.pod_name,
            req.container.as_deref(),
            req.tail.unwrap_or(100)
        ).await
    }
}
```

### 4. Add HTTP Handler
```rust
// src/interfaces/http/mod.rs
#[get("/api/pods/{namespace}/{name}/logs")]
async fn get_pod_logs(
    data: web::Data<AppState>,
    path: web::Path<GetPodPath>,
    query: web::Query<PodLogsQuery>,
) -> impl Responder {
    let use_case = GetPodLogsUseCase::new(Arc::clone(&data.k8s_repo));
    
    match use_case.execute(PodLogsRequestDto {
        namespace: path.namespace.clone(),
        pod_name: path.name.clone(),
        container: query.container.clone(),
        tail: query.tail,
    }).await {
        Ok(logs) => HttpResponse::Ok().body(logs),
        Err(e) => e.error_response(),
    }
}
```

## Testing

### Unit Tests (Domain)
```rust
#[test]
fn test_pod_status_is_error() {
    assert!(PodStatus::Failed.is_error());
    assert!(!PodStatus::Running.is_error());
}
```

### Integration Tests (Use Cases with Mocks)
```rust
#[tokio::test]
async fn test_get_pod_details_use_case() {
    let mock_repo = Arc::new(MockK8sRepo);
    let use_case = GetPodDetailsUseCase::new(mock_repo);
    
    let result = use_case.execute("default", "test-pod").await;
    assert!(result.is_ok());
}
```

### E2E Tests (Full Stack)
```rust
#[actix_web::test]
async fn test_get_pod_http_endpoint() {
    let app = test::init_service(
        App::new().configure(configure_routes)
    ).await;
    
    let req = TestRequest::get()
        .uri("/api/pods/default/test-pod")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

## Migration from Old Structure

### Before (Monolithic)
```
src/
  main.rs         # Everything mixed together
  pods.rs         # HTTP handlers + K8s API calls + business logic
  nodes.rs        # Same
  events.rs       # Same
```

### After (Hexagonal)
```
src/
  main.rs              # Wiring only
  domain/              # Pure business logic
    entities/          # Domain objects
    ports/             # Interfaces
    services/          # Business operations
  application/         # Use cases
    use_cases/         # Application operations
    dtos/              # Input/output objects
    mappers/           # Conversions
  infrastructure/      # External implementations
    repositories/      # K8s, Prometheus, etc.
    clients/           # External API clients
  interfaces/          # Delivery mechanisms
    http/              # REST API handlers
    websocket/         # WebSocket handlers
    middleware/        # HTTP middleware
```

## Statistics

| Layer | Files | Lines of Code | Tests |
|-------|-------|---------------|-------|
| Domain | 4 | ~1,500 | 20+ |
| Application | 6 | ~2,000 | 10+ |
| Infrastructure | 3 | ~800 | 5+ |
| Interface | 4 | ~600 | 3+ |
| **Total** | **17** | **~5,000** | **118** |

## Next Steps

1. **Migrate remaining modules**: Move existing code to new structure
2. **Add dependency injection**: Use a container for wiring
3. **Add more tests**: Aim for 80%+ coverage
4. **Add OpenAPI documentation**: Auto-generate from handlers
5. **Add event sourcing**: For audit trails and replay
