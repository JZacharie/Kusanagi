# Kusanagi - Product Roadmap 2026

> **Product Vision**: Kusanagi est la plateforme d'observabilité ultime pour Kubernetes - combinant monitoring en temps réel, auto-remédiation intelligente, et intégration personnelle dans une interface cyberpunk.

---

## 🎯 Product Strategy

### Core Pillars
1. **Observability First**: Vue complète de l'infrastructure K8s
2. **Action-Oriented**: Capacités de remédiation et contrôle
3. **Intelligence**: Insights automatisés via MCP et AI
4. **Personal Integration**: Hub centralisé pour infrastructure ET vie personnelle
5. **Robust Architecture**: Codebase propre, testée et maintenable (Domain-Driven Design)

### Success Metrics
- **Adoption**: Utilisation quotidienne par l'équipe
- **MTTR**: Réduction du temps de résolution des incidents
- **Coverage**: 100% des composants critiques monitorés
- **Code Quality**: Clippy zero-warnings & >80% test coverage
- **UX**: Score Lighthouse > 95

## 🚨 Immediate Remediation Tasks (Urgent)

Les alertes suivantes ont été détectées et doivent être traitées en priorité :

### 🔴 Critical Alerts
- [ ] **KubeClientCertificateExpiration**: Renouveler les certificats clients expirant dans moins de 24h.
- [ ] **KubeControllerManagerDown**: Diagnostiquer et redémarrer le Controller Manager (disparu de Prometheus).
- [ ] **KubeProxyDown**: Rétablir le service KubeProxy sur les nœuds affectés.
- [ ] **KubeSchedulerDown**: Restaurer le Scheduler du plan de contrôle.

### 🟠 Warning Alerts
- [ ] **HPA Scaling**: Investiguer les limites HPA atteintes dans `devsecops`, `turbot`, `argocd`.
- [ ] **Job Failures**: Analyser les échecs de jobs dans `vault`, `vault-sbx`, `sim`.
- [ ] **TargetDown**: Identifier les targets inaccessibles pour Prometheus.

---

## 📅 Release Timeline

```
Q1 2026          Q2 2026          Q3 2026          Q4 2026
───────────────────────────────────────────────────────────
v0.8 ✅          v1.0 ✅          v1.2 🔄          v1.3 📋          v2.0
News Feed       Personal Hub     Refactor & Setup  AI Safety       Enterprise
v0.9 ✅          v1.1 🔄
Infra Monitor    Data & Msg
```

---

## 🚀 Completed Releases

### v0.8.0 - News Feed (SHIPPED ✅)
- Tech News Aggregation (Hacker News, RSS)
- Cyberpunk UI

### v0.9.0 - Infrastructure Monitoring (SHIPPED ✅)
- Proxmox Integration (VMs/CTs)
- Home Assistant Sensors
- Resource Tracking

### v1.0.0 - Personal Dashboard Hub (SHIPPED/MVP ✅)
**Theme**: Personal Productivity Integration
- ✅ Google Calendar Widget
- ✅ Weather Widget & Multi-City
- ✅ Slack Notifications & Sync
- [ ] Event Creation (deferred)

---

## 🔨 Current Cycle: v1.1.0 - Data & Messaging Monitoring

**Target Date**: Late Q1 2026
**Status**: 🔄 Active
**Theme**: Persistence, Messaging & Health Verification

### Priority Features

#### P0 - Must Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| ✅ **PostgreSQL Health Check** | M | PG Connection | Backend |
| **MQTT Broker Health** | M | MQTT Connection | Backend |
| Topic Explorer Dashboard (MQTT) | L | MQTT Broker Health | Frontend |
| Database Latency & Version | S | Health Check | Frontend |

#### P1 - Should Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| Database Verification (Cache Hit, Conn) | M | PG Connection | Backend |
| Schema Integrity Scan | L | DB Verification | Backend |
| Size Tracking & Bloat | M | DB Verification | Backend |

---

## 🔧 Strategic Initiative: v1.2.0 - The Great Refactoring & Onboarding

**Target Date**: Q2 2026
**Status**: 📋 Planned (Critical)
**Theme**: Technical Debt Paydown & User Experience

### Priority Features

#### P0 - Architecture Refactoring (Critical)
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| **Domain Layer Migration** | L | - | Backend |
| **Infrastructure Layer Isolation** | L | - | Backend |
| **Interface Layer Separation** | M | - | Backend |
| **Clean Architecture Enforcement** | M | - | Backend |

#### P1 - Onboarding Experience
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| **Interactive Setup Wizard** | L | - | Frontend |
| Environment Interview API | M | Setup Wizard | Backend |
| "Doctor" Self-Diagnostic Tool | M | - | Backend |

### Technical Goals
- Move logic from `src/*.rs` flat files to `domain/`, `infrastructure/`, `interfaces/`.
- Decouple business logic from Actix/Kube-rs.
- Implement Repository pattern for all external data sources.

---

## 🛡️ Future Release: v1.3.0 - AI Safety & Security

**Target Date**: Q3 2026
**Status**: 📋 Planned
**Theme**: Robustness Against Adversarial Attacks

### Priority Features

#### P0 - Must Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| **Spotlighting (Data Provenance)** | S | chat.rs | Backend |
| **Harmlessness Screens** | M | - | Backend |
| **Canary Tokens Implementation** | S | chat.rs | Backend |

#### P1 - Should Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| Output Validation Engine | M | MCP Integration | Backend |
| **Input Paraphrasing** | M | - | Backend |

---

## 🏢 Long-term Vision: v2.0.0 - Enterprise Features

**Target Date**: Q4 2026
**Status**: 💭 Ideation
**Theme**: Security, Scale, Intelligence

### Strategic Initiatives
- [ ] RBAC & Keycloak Auth
- [ ] Multi-cluster support
- [ ] AI-powered auto-remediation (Agentic Loops)
- [ ] Distributed tracing

---

## 📊 Feature Prioritization Framework

### Priority Levels
- **P0 (Must Have)**: Core functionality, blocks release
- **P1 (Should Have)**: Important but can be deferred
- **P2 (Nice to Have)**: Enhancement, low impact if missing

---

## 🔄 Continuous Improvements

### Code Quality
- [ ] **Rust Architecture Audit** (Merged into v1.2)
- [ ] Add unit tests (>80% coverage)
- [ ] Reduce technical debt

### DevOps
- [ ] CI/CD pipeline optimization
- [ ] Security scanning (Trivy)

---

**Last Updated**: 2026-02-01
**Product Manager**: AI Assistant
**Engineering Lead**: Joseph Zacharie
