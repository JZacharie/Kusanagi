# Kusanagi - Architecture Documentation

## 🎯 Vision & Mission

**Vision**: Kusanagi est une plateforme d'observabilité et d'auto-remédiation pour Kubernetes, inspirée par Ghost in the Shell.

**Mission**: Fournir une vue unifiée de l'infrastructure K8s avec des capacités d'action automatisée, tout en maintenant une empreinte minimale grâce à Rust.

---

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Frontend (Vanilla JS)                    │
│  ┌──────────┬──────────┬──────────┬──────────┬───────────┐ │
│  │Dashboard │ ArgoCD   │  Nodes   │ Network  │ News Feed │ │
│  │ Manager  │ Monitor  │ Monitor  │ (Cilium) │ Aggregator│ │
│  └──────────┴──────────┴──────────┴──────────┴───────────┘ │
│                            ▲                                 │
│                            │ WebSocket + REST API            │
└────────────────────────────┼─────────────────────────────────┘
                             │
┌────────────────────────────┼─────────────────────────────────┐
│                  Backend (Rust + Actix-web)                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │              API Layer (main.rs)                     │   │
│  │  - REST endpoints                                    │   │
│  │  - WebSocket notifications                           │   │
│  │  - Authentication (future)                           │   │
│  └──────────────────────────────────────────────────────┘   │
│                             │                                │
│  ┌──────────────────────────┼────────────────────────────┐  │
│  │           Business Logic Modules                      │  │
│  │  ┌──────────┬──────────┬──────────┬─────────────┐   │  │
│  │  │ argocd   │  nodes   │ cilium   │  newsfeed   │   │  │
│  │  │  .rs     │   .rs    │  .rs     │    .rs      │   │  │
│  │  ├──────────┼──────────┼──────────┼─────────────┤   │  │
│  │  │ storage  │  pods    │ events   │  prometheus │   │  │
│  │  │  .rs     │   .rs    │  .rs     │    .rs      │   │  │
│  │  ├──────────┼──────────┼──────────┼─────────────┤   │  │
│  │  │  chat    │  mcp     │ export   │ telemetry   │   │  │
│  │  │  .rs     │   .rs    │  .rs     │    .rs      │   │  │
│  │  └──────────┴──────────┴──────────┴─────────────┘   │  │
│  └───────────────────────────────────────────────────────┘  │
│                             │                                │
│  ┌──────────────────────────┼────────────────────────────┐  │
│  │         External Integrations                         │  │
│  │  - Kubernetes API (kube-rs)                           │  │
│  │  - Prometheus/AlertManager                            │  │
│  │  - Cilium/Hubble                                      │  │
│  │  - OpenObserve (RUM/Logs)                             │  │
│  │  - S3/MinIO (Chat storage)                            │  │
│  │  - MCP Servers (Kubernetes, Cilium, Steampipe)       │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## 📦 Component Architecture

### Backend Components

#### Core API Layer (`main.rs`)
- **Responsibility**: HTTP server, routing, middleware
- **Technology**: Actix-web 4.4
- **Key Features**:
  - REST API endpoints
  - WebSocket server for real-time notifications
  - Static file serving
  - CORS handling

#### Kubernetes Integration
- **argocd.rs**: ArgoCD application monitoring and sync operations
- **nodes.rs**: Node status, metrics, and diagnostics
- **pods.rs**: Pod monitoring and force-delete operations
- **storage.rs**: PVC monitoring and capacity tracking
- **events.rs**: Kubernetes events aggregation

#### Observability
- **prometheus.rs**: Prometheus metrics queries
- **alertmanager.rs**: Alert aggregation from AlertManager
- **telemetry.rs**: OpenObserve RUM/APM integration
- **export.rs**: Report generation (JSON/CSV/Markdown)

#### Network & Security
- **cilium.rs**: Cilium/Hubble network flow visualization
- **services.rs**: Service discovery and monitoring
- **ingress.rs**: Ingress rules tracking

#### Intelligence Layer
- **chat.rs**: Chatbot with MCP integration
- **mcp.rs**: Model Context Protocol servers
- **newsfeed.rs**: News aggregation (HN, Korben, GitHub)

#### Data Management
- **chat_storage.rs**: S3/MinIO chat persistence
- **quota.rs**: Google services quota tracking

### Frontend Components

#### Dashboard Manager (`dashboard.js`)
- Widget management
- Layout persistence
- Export functionality

#### Specialized Managers
- **MetricsManager**: Prometheus metrics display
- **AlertsManager**: Alert visualization
- **QuotasManager**: Quota gauges
- **NewsManager**: News feed rendering

#### Network Visualization (`network.js`)
- D3.js/Mermaid service graphs
- Flow matrix rendering
- Bandwidth metrics

#### Real-time Features
- WebSocket connection management
- Live notifications
- Auto-refresh mechanisms

---

## 🔧 Technology Stack

### Backend
| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| Runtime | Rust | 2021 edition | Core language |
| Web Framework | Actix-web | 4.4 | HTTP server |
| Async Runtime | Tokio | 1.35 | Async operations |
| K8s Client | kube-rs | 0.87 | Kubernetes API |
| Serialization | Serde | 1.0 | JSON handling |
| HTTP Client | Reqwest | 0.11 | External APIs |
| RSS Parser | rss | 2.0 | Feed parsing |
| AWS SDK | aws-sdk-s3 | 1.15 | S3 storage |

### Frontend
| Component | Technology | Purpose |
|-----------|-----------|---------|
| Framework | Vanilla JS | No framework overhead |
| Styling | Custom CSS | Cyberpunk theme |
| Network Viz | D3.js/Mermaid | Graph rendering |
| RUM | OpenObserve | Real user monitoring |

### Infrastructure
- **Deployment**: Helm Chart + ArgoCD
- **Namespace**: `kusanagi`
- **Ingress**: `kusanagi.p.zacharie.org`
- **Observability**: OpenObserve (RUM, Logs, APM)

---

## 🔐 Security Architecture

### Current State
- ⚠️ **No authentication** - Public access
- ✅ **RBAC**: ClusterRole with read/patch permissions
- ✅ **ServiceAccount**: `kusanagi` with limited scope

### Planned Improvements (v2.0)
- [ ] Keycloak/OIDC authentication
- [ ] Role-based access control
- [ ] API key authentication
- [ ] Audit logging
- [ ] Secret management (Vault integration)

---

## 📊 Data Flow

### Kubernetes Monitoring Flow
```
K8s API Server → kube-rs → Rust modules → REST API → Frontend
                                ↓
                         Prometheus/Metrics
```

### News Feed Flow
```
External APIs → Cache (30min) → Background refresh → REST API → Frontend
  (HN/Korben/GitHub)         (Arc<RwLock>)
```

### Chat Flow
```
User → Frontend → REST API → MCP Servers → K8s/Cilium/Steampipe
                      ↓
                   S3 Storage (MinIO)
```

### Real-time Notifications
```
K8s Events → WebSocket Server → Connected Clients
Alerts     →                  → Browser notifications
```

---

## 🎨 Design Patterns

### Backend Patterns
1. **Module-per-Feature**: Each feature in separate `.rs` file
2. **Async/Await**: All I/O operations are async
3. **Error Handling**: `KusanagiError` with `thiserror` (see `src/error.rs`)
4. **Caching**: Arc<RwLock<>> for thread-safe caching
5. **Background Tasks**: tokio::spawn for periodic jobs
6. **Structured Errors**: Each error variant has specific HTTP status mapping

### Frontend Patterns
1. **Manager Objects**: Encapsulated state management
2. **Event-Driven**: DOM events + WebSocket messages
3. **Progressive Enhancement**: Works without JS (basic)
4. **Local Storage**: User preferences persistence

---

## 🚀 Performance Characteristics

### Backend
- **Memory**: ~50MB base (Rust efficiency)
- **CPU**: <5% idle, <20% under load
- **Startup**: <2 seconds
- **Response Time**: <100ms (cached), <500ms (K8s API)

### Frontend
- **Bundle Size**: ~200KB (no framework)
- **Load Time**: <1s (TTFB)
- **RUM Score**: 95+ (Lighthouse)

### Caching Strategy
- **News Feed**: 30 minutes
- **Metrics**: 30 seconds
- **K8s Resources**: On-demand (no cache)

---

## 🔄 Scalability Considerations

### Current Limitations
- Single instance (no horizontal scaling)
- In-memory cache (lost on restart)
- No database (stateless)

### Future Improvements
- [ ] Redis for distributed caching
- [ ] PostgreSQL for persistent storage
- [ ] Multi-replica deployment
- [ ] Load balancer support

---

## 🐛 Known Technical Debt

### High Priority
1. **Authentication**: No auth mechanism
2. **Error Handling**: ✅ Completed - `thiserror` implemented with 52 tests
3. **Testing**: 🔄 In Progress - Error module has 52 tests, adding more
4. **Logging**: Basic tracing, needs structured logging

### Medium Priority
1. **Code Duplication**: Similar patterns across modules
2. **Type Safety**: Some serde_json::Value usage
3. **Configuration**: ✅ Completed - Structured config with `config` crate + 16 tests
4. **Documentation**: Missing inline docs

### Low Priority
1. **Unused Code**: Dead code warnings
2. **CSS Organization**: Single large file
3. **JS Modules**: Should use ES6 modules

---

## 🔮 Future Architecture Vision

### Microservices Evolution
```
┌─────────────────────────────────────────────────────────┐
│                   API Gateway                           │
│              (Authentication/Rate Limiting)             │
└─────────────────────────────────────────────────────────┘
         │              │              │
    ┌────┴────┐    ┌────┴────┐   ┌────┴────┐
    │ Core    │    │ Intel   │   │ External│
    │ Monitor │    │ Layer   │   │ Integr. │
    │ Service │    │ Service │   │ Service │
    └─────────┘    └─────────┘   └─────────┘
         │              │              │
    ┌────┴──────────────┴──────────────┴────┐
    │         Shared Data Layer              │
    │    (Redis, PostgreSQL, S3)             │
    └────────────────────────────────────────┘
```

### Agent-Based Architecture (v3.0)
- **Kusanagi Core**: Central controller
- **Node Agents**: DaemonSet on each node
- **Auto-Remediation**: Automated issue resolution
- **ML/AI**: Anomaly detection and prediction

---

## 📚 References

- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Actix-web Documentation](https://actix.rs/)
- [kube-rs Guide](https://kube.rs/)
- [Kubernetes API Reference](https://kubernetes.io/docs/reference/)
- [OpenObserve RUM](https://openobserve.ai/)
