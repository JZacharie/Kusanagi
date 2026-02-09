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
│       ├── ingress_use_cases.rs
│       ├── cluster_use_cases.rs
│       ├── prometheus_use_cases.rs
│       ├── backup_use_cases.rs
│       ├── security_use_cases.rs
│       ├── alert_use_cases.rs
│       ├── chat_use_cases.rs
│       └── node_metrics_use_cases.rs
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
│   │   ├── ingress_handlers.rs
│   │   ├── cluster_handlers.rs
│   │   ├── prometheus_handlers.rs
│   │   ├── backup_handlers.rs
│   │   ├── security_handlers.rs
│   │   ├── alert_handlers.rs
│   │   ├── chat_handlers.rs
│   │   └── node_metrics_handlers.rs
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
| Cluster | ✅ Refactored | Hexagonal |
| Prometheus | ✅ Refactored | Hexagonal |
| Backups | ✅ Refactored | Hexagonal |
| Security | ✅ Refactored | Hexagonal |
| Alertmanager | ✅ Refactored | Hexagonal |
| Chat | ✅ Refactored | Hexagonal |
| Weather | ✅ Refactored | Hexagonal |
| ... | 🔄 Pending | Legacy |

### 3. API Endpoints

#### Refactored Endpoints (using hexagonal architecture)
```
# Pods
GET  /api/pods                    -> ListPodsUseCase
GET  /api/pods/{ns}/{name}        -> GetPodDetailsUseCase

# Nodes
GET  /api/nodes                   -> GetNodesUseCase
GET  /api/nodes/status            -> GetNodesStatusUseCase
GET  /api/nodes/{name}            -> GetNodeDetailsUseCase
GET  /api/nodes/{name}/ready      -> IsNodeReadyUseCase

# Events
GET  /api/events                  -> GetRecentEventsUseCase
GET  /api/events/warnings         -> GetWarningEventsUseCase
GET  /api/events/stats            -> GetEventStatsUseCase

# Cluster
GET  /api/cluster/overview        -> GetClusterOverviewUseCase
GET  /api/cluster/empty-namespaces -> GetEmptyNamespacesUseCase
GET  /api/cluster/stats           -> GetClusterStatsUseCase

# Storage
GET  /api/storage                 -> GetStorageInfoUseCase
GET  /api/storage/stats           -> GetStorageStatsUseCase

# Services
GET  /api/services                -> ListServicesUseCase
GET  /api/services/stats          -> GetServiceStatsUseCase
GET  /api/services/{ns}/{name}    -> GetServiceDetailsUseCase

# Ingresses
GET  /api/ingresses               -> ListIngressesUseCase
GET  /api/ingresses/stats         -> GetIngressStatsUseCase
GET  /api/ingresses/{ns}/{name}   -> GetIngressDetailsUseCase

# ArgoCD
GET  /api/argocd/applications     -> GetArgoCdApplicationsUseCase
GET  /api/argocd/applications/{name}/status -> GetApplicationStatusUseCase
POST /api/argocd/applications/{name}/sync  -> SyncApplicationUseCase

# Prometheus
GET  /api/metrics                 -> GetClusterMetricsUseCase
GET  /api/prometheus/query        -> QueryMetricUseCase
GET  /api/prometheus/query_raw    -> QueryRawUseCase
GET  /api/prometheus/range        -> QueryRangeUseCase

# Backups
GET  /api/backups                 -> GetBackupStatusUseCase
GET  /api/backups/stats           -> GetBackupStatsUseCase
GET  /api/backups/cronjobs        -> ListCronJobsUseCase
POST /api/backups/{ns}/{name}/trigger -> TriggerBackupUseCase

# Security
GET  /api/security/reports        -> ListSecurityReportsUseCase
GET  /api/security/summary        -> GetSecuritySummaryUseCase
GET  /api/security/enriched/{cat}/{name} -> GetSecurityReportUseCase
POST /api/security/enrich/{cat}/{name}   -> EnrichSecurityReportUseCase
POST /api/security/enrich-all     -> RunSecurityEnrichmentUseCase

# Alerts
GET  /api/alerts                  -> GetActiveAlertsUseCase
GET  /api/alerts/cached           -> GetCachedAlertsUseCase
GET  /api/alerts/stats            -> GetAlertStatsUseCase
GET  /api/alerts/{fingerprint}    -> GetAlertUseCase
POST /api/alerts/silence          -> SilenceAlertUseCase

# Chat
POST /api/chat                    -> ProcessChatMessageUseCase
POST /api/chat/command            -> HandleChatCommandUseCase
POST /api/chat/query              -> QueryAiUseCase
GET  /api/chat/history            -> GetChatHistoryUseCase
POST /api/chat/clear              -> ClearChatHistoryUseCase

# Weather
GET  /api/weather/current         -> GetWeatherUseCase
POST /api/weather/refresh         -> GetWeatherUseCase (force_refresh)

# Node Metrics (with Disk Usage)
GET  /api/nodes/with-metrics      -> GetNodesWithDiskMetricsUseCase
GET  /api/nodes/{name}/disk       -> GetNodeDiskUsageUseCase
GET  /api/nodes/disk-summary      -> GetClusterDiskSummaryUseCase
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
