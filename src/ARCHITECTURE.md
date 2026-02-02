# Kusanagi Architecture

## Overview

Kusanagi follows **Hexagonal Architecture** (Ports and Adapters) with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────────┐
│                         INTERFACE LAYER                          │
│  (HTTP handlers, WebSocket, CLI, external APIs)                  │
├─────────────────────────────────────────────────────────────────┤
│                       APPLICATION LAYER                          │
│  (Use cases, application services, DTOs, mappers)                │
├─────────────────────────────────────────────────────────────────┤
│                         DOMAIN LAYER                             │
│  (Entities, value objects, domain services, repository ports)    │
├─────────────────────────────────────────────────────────────────┤
│                      INFRASTRUCTURE LAYER                        │
│  (Repository implementations, external clients, cache)           │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
src/
├── main.rs                 # Application entry point
├── config.rs              # Configuration management
├── error.rs               # Error types and handling
├── cache.rs               # Caching infrastructure
├── features.rs            # Feature flags
├── response.rs            # Response utilities
├── validation.rs          # Input validation
│
├── domain/                # Domain layer (business logic)
│   ├── entities/          # Domain entities (Pod, Node, Event, etc.)
│   ├── ports/             # Repository interfaces (driven ports)
│   └── services/          # Domain services
│
├── application/           # Application layer (use cases)
│   ├── dtos/              # Data Transfer Objects
│   ├── mappers/           # Entity <-> DTO converters
│   └── use_cases/         # Use case implementations
│       ├── pod_use_cases.rs
│       ├── node_use_cases.rs
│       ├── event_use_cases.rs
│       ├── argocd_use_cases.rs
│       ├── storage_use_cases.rs
│       ├── service_use_cases.rs
│       └── ingress_use_cases.rs
│
├── infrastructure/        # Infrastructure layer
│   ├── repositories/      # Repository implementations
│   │   ├── kubernetes_repository.rs
│   │   └── argocd_repository.rs
│   ├── clients/           # External API clients
│   └── external/          # External integrations
│
├── interfaces/            # Interface layer
│   ├── http/              # HTTP handlers
│   │   ├── mod.rs         # Route configuration
│   │   ├── pod_handlers.rs
│   │   ├── node_handlers.rs
│   │   ├── event_handlers.rs
│   │   ├── argocd_handlers.rs
│   │   ├── storage_handlers.rs
│   │   ├── service_handlers.rs
│   │   └── ingress_handlers.rs
│   ├── middleware/        # HTTP middleware
│   └── websocket/         # WebSocket handlers
│
├── legacy/                # Modules being refactored
│   ├── argocd.rs
│   ├── nodes.rs
│   ├── events.rs
│   ├── storage.rs
│   ├── services.rs
│   ├── ingress.rs
│   ├── apps.rs
│   ├── backups.rs
│   ├── chat.rs
│   ├── prometheus.rs
│   ├── alertmanager.rs
│   └── ... (36 files total)
│
├── event_bus/             # Event system
├── jobs/                  # Background jobs
├── metrics/               # Custom metrics
├── middleware/            # Additional middleware
└── resilience/            # Circuit breakers, retry, timeout
```

## Key Principles

### 1. Dependency Rule
Dependencies point inward:
- **Domain** knows nothing about other layers
- **Application** depends only on Domain
- **Infrastructure** implements Domain ports
- **Interfaces** uses Application services

### 2. Refactoring Status

| Module | Status | Location |
|--------|--------|----------|
| Pods | ✅ Refactored | Hexagonal |
| Nodes | ✅ Refactored | Hexagonal |
| Events | ✅ Refactored | Hexagonal |
| ArgoCD | ✅ Refactored | Hexagonal |
| Storage | ✅ Refactored | Hexagonal |
| Services | ✅ Refactored | Hexagonal |
| Ingress | ✅ Refactored | Hexagonal |
| Cluster | 🔄 Pending | Legacy |
| Backups | 🔄 Pending | Legacy |
| Chat | 🔄 Pending | Legacy |
| Prometheus | 🔄 Pending | Legacy |
| Alertmanager | 🔄 Pending | Legacy |
| ... | 🔄 Pending | Legacy |

### 3. API Endpoints

#### Refactored Endpoints (using hexagonal architecture)
```
GET  /api/pods                    -> ListPodsUseCase
GET  /api/pods/{ns}/{name}        -> GetPodDetailsUseCase
GET  /api/nodes                   -> GetNodesUseCase
GET  /api/nodes/status            -> GetNodesStatusUseCase
GET  /api/nodes/{name}            -> GetNodeDetailsUseCase
GET  /api/events                  -> GetRecentEventsUseCase
GET  /api/events/warnings         -> GetWarningEventsUseCase
GET  /api/storage                 -> GetStorageInfoUseCase
GET  /api/services                -> ListServicesUseCase
GET  /api/ingresses               -> ListIngressesUseCase
GET  /api/argocd/applications     -> GetArgoCdApplicationsUseCase
```

## Adding New Modules

To add a new module following hexagonal architecture:

1. **Domain** (`src/domain/`)
   - Add entities to `entities/mod.rs`
   - Add repository port to `ports/mod.rs`

2. **Application** (`src/application/`)
   - Create use cases in `use_cases/{module}_use_cases.rs`
   - Add DTOs to `dtos/mod.rs` if needed
   - Add mappers to `mappers/mod.rs` if needed

3. **Infrastructure** (`src/infrastructure/`)
   - Implement repository in `repositories/{module}_repository.rs`

4. **Interface** (`src/interfaces/http/`)
   - Create handlers in `http/{module}_handlers.rs`
   - Register routes in `http/mod.rs`

## Testing

Each layer can be tested independently:
- **Domain**: Unit tests with mocked repositories
- **Application**: Integration tests with test doubles
- **Infrastructure**: Integration tests with real dependencies
- **Interfaces**: HTTP-level integration tests
