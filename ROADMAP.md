# 🗺️ Kusanagi Roadmap

> This document outlines the planned development direction for Kusanagi.

## 📅 Version Timeline

| Version | Target Date | Focus | Status |
|---------|-------------|-------|--------|
| v0.2.1 | Q1 2026 | Stability & Testing | 🔄 In Progress |
| v0.3.0 | Q2 2026 | Observability | 📋 Planned |
| v0.4.0 | Q3 2026 | Security & Auth | 📋 Planned |
| v1.0.0 | Q4 2026 | Production Ready | 📋 Planned |

---

## 🎯 v0.2.1 - Stability & Testing (Current)

### Goals
- Improve code quality and test coverage
- Complete hexagonal architecture migration
- Enhance documentation

### Tasks

#### Testing & Quality
- [ ] Increase test coverage to 60%
  - [ ] Unit tests for all domain services
  - [ ] Integration tests for HTTP handlers
  - [ ] Mock implementations for repositories
- [ ] Add property-based tests with `proptest`
- [ ] Setup mutation testing with `cargo-mutants`

#### Architecture Migration
- [ ] Migrate remaining legacy modules:
  - [ ] `legacy/security.rs` → hexagonal
  - [ ] `legacy/alertmanager.rs` → hexagonal
  - [ ] `legacy/proxmox.rs` → hexagonal (optional)
  - [ ] `legacy/weather.rs` → hexagonal (optional)

#### Documentation
- [ ] API documentation with utoipa
- [ ] Interactive API explorer (Swagger UI)
- [ ] Deployment guides
- [ ] Troubleshooting guide

---

## 📊 v0.3.0 - Observability & Performance

### Goals
- Comprehensive observability stack
- Performance optimizations
- Enhanced monitoring capabilities

### Features

#### Observability
- [ ] OpenTelemetry integration
  - [ ] Distributed tracing
  - [ ] Custom metrics
  - [ ] Log correlation
- [ ] Advanced health checks
  - [ ] Deep health endpoint
  - [ ] Dependency status
  - [ ] Readiness/liveness probes
- [ ] Performance profiling
  - [ ] Memory profiling
  - [ ] CPU profiling
  - [ ] Request tracing

#### Performance
- [ ] Connection pooling
- [ ] Request coalescing
- [ ] Intelligent caching strategies
  - [ ] LRU cache
  - [ ] Cache warming
  - [ ] Cache invalidation
- [ ] Benchmark suite

#### Storage
- [ ] Optional database persistence
  - [ ] SQLite for simple deployments
  - [ ] PostgreSQL for production
- [ ] Historical data retention
- [ ] Configuration persistence

---

## 🔒 v0.4.0 - Security & Authentication

### Goals
- Enterprise-grade security
- Multi-tenant support
- Audit capabilities

### Features

#### Authentication
- [ ] OAuth2/OIDC support
  - [ ] GitHub OAuth
  - [ ] Google OAuth
  - [ ] Generic OIDC
- [ ] API key authentication
- [ ] Session management

#### Authorization
- [ ] RBAC implementation
  - [ ] Role definitions
  - [ ] Permission system
  - [ ] Resource-level access control
- [ ] Namespace isolation
- [ ] Read-only mode

#### Security Features
- [ ] Audit logging
- [ ] Request signing
- [ ] IP allowlisting
- [ ] Rate limiting per user

---

## 🚀 v1.0.0 - Production Ready

### Goals
- Stable, enterprise-ready platform
- Complete feature set
- Professional support options

### Features

#### Platform
- [ ] Multi-cluster support
- [ ] High availability mode
- [ ] Disaster recovery
- [ ] Backup/restore

#### Enterprise
- [ ] SSO integration
- [ ] LDAP/AD support
- [ ] Audit compliance (SOC2, ISO)
- [ ] Professional support

#### Ecosystem
- [ ] Plugin system
- [ ] Custom dashboard widgets
- [ ] Webhook integrations
- [ ] Slack/Teams bots

---

## 🌟 Future Ideas

These are potential features for future versions:

### AI & Machine Learning
- [ ] Anomaly detection
- [ ] Predictive scaling
- [ ] Natural language queries
- [ ] Automated troubleshooting

### Multi-Cloud
- [ ] AWS EKS support
- [ ] Azure AKS support
- [ ] GCP GKE support
- [ ] Cross-cluster management

### GitOps Enhancements
- [ ] ArgoCD application management
- [ ] Flux integration
- [ ] Terraform state visualization
- [ ] Git webhook triggers

### Developer Experience
- [ ] VSCode extension
- [ ] CLI tool
- [ ] Mobile app
- [ ] Dark/light themes

---

## 📈 Success Metrics

| Metric | Current | v0.2.1 | v0.3.0 | v1.0.0 |
|--------|---------|--------|--------|--------|
| Test Coverage | 15% | 60% | 70% | 80% |
| Legacy Code | 36 files | 0 files | 0 files | 0 files |
| CI Time | 15 min | 10 min | 8 min | 5 min |
| Contributors | 1 | 3 | 5 | 10+ |
| Open Issues | - | < 20 | < 10 | < 5 |

---

## 🤝 Contributing to the Roadmap

We welcome input on the roadmap! Here's how to contribute:

1. **Suggest a Feature**: Open a discussion with the `roadmap` label
2. **Vote on Features**: React to issues/roadmap items
3. **Contribute Code**: Pick up tasks and submit PRs

### Prioritization

We use the following criteria to prioritize:

1. **User Impact**: How many users benefit?
2. **Technical Debt**: Does it reduce complexity?
3. **Strategic Value**: Does it align with our vision?
4. **Community Input**: What do users want?

---

## 📞 Questions?

For questions about the roadmap:

- Open a [GitHub Discussion](https://github.com/JZacharie/Kusanagi/discussions)
- Comment on specific roadmap items
- Contact maintainers

---

*Last updated: 2026-02-09*
