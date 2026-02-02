# Kusanagi - Product Roadmap 2026-2027

> **Product Vision**: Kusanagi est la plateforme d'observabilité ultime pour Kubernetes - combinant monitoring en temps réel, auto-remédiation intelligente, et intégration personnelle dans une interface cyberpunk.

---

## 🎯 Product Strategy 2.0

### Core Pillars (Updated)
1. **🎯 Observability First**: Vue 360° de l'infrastructure (métriques, logs, traces, cost)
2. **🤖 Intelligence Augmentée**: AI/ML pour la corrélation d'incidents et prédiction
3. **⚡ Action-Oriented**: Auto-remédiation et runbooks automatisés
4. **🏠 Personal Integration**: Hub centralisé infrastructure + vie personnelle
5. **🏗️ Robust Architecture**: Architecture Hexagonale complète, >80% test coverage
6. **💰 Cost Intelligence**: Optimisation des coûts cloud et resource right-sizing

### Success Metrics 2026
| Metric | Target | Current |
|--------|--------|---------|
| **Refactoring Progress** | 100% modules hexagonaux | 13/35 (37%) |
| **Test Coverage** | >80% | ~15% |
| **MTTR** | <15 min | ~45 min |
| **Alert Fatigue** | <5 faux positifs/jour | ~20/jour |
| **Code Quality** | Clippy zero warnings | 96 warnings |
| **User Adoption** | Daily active users | Intermittent |

---

## 📅 Release Timeline 2026-2027

```
Q1 2026          Q2 2026          Q3 2026          Q4 2026         Q1 2027
────────────────────────────────────────────────────────────────────────
v0.8 ✅          v1.0 ✅          v1.2 🔄          v1.3 📋          v2.0-alpha
News Feed       Personal Hub     Refactor &        AI Intelligence  Enterprise
v0.9 ✅          v1.1 🔄                           Cost Mgmt        GA
Infra Monitor    Data & Msg       v1.2.5 ✅
                                  Disk Monitor
                                  
Légende: ✅ SHIPPED | 🔄 ACTIVE | 📋 PLANNED | 💭 IDEATION
```

---

## ✅ Completed Achievements

### 🏗️ Architecture Milestone (Février 2026)
**The Great Refactoring** - 13 modules migrés vers Architecture Hexagonale :
- ✅ Pods, Nodes, Events, ArgoCD, Storage
- ✅ Services, Ingress, Cluster, Prometheus
- ✅ Backups, Security, Alertmanager, Chat
- ✅ Node Disk Monitoring (nouveau!)

**Structure du projet**:
```
src/
├── domain/          # 11 entités, 6 ports
├── application/     # 13 use cases modules, 48+ use cases
├── infrastructure/  # 2 repositories
├── interfaces/      # 13 handlers, 50+ endpoints
└── legacy/          # 22 modules restants
```

---

## 🔨 Current Cycle: v1.1.x - Data, Messaging & Observabilité

**Target Date**: Q1 2026 (Mars)
**Status**: 🔄 Active
**Theme**: Infrastructure Data & Health Monitoring

### P0 - Must Have
| Feature | Status | Effort | Owner |
|---------|--------|--------|-------|
| ✅ **Node Disk Monitoring** | ✅ SHIPPED | M | Backend |
| ✅ **PostgreSQL Health Check** | ✅ SHIPPED | M | Backend |
| **MQTT Broker Health** | 🔄 70% | M | Backend |
| **Topic Explorer Dashboard** | 📋 | L | Frontend |
| **Database Performance Metrics** | 📋 | M | Backend |

### P1 - Should Have
| Feature | Status | Effort | Owner |
|---------|--------|--------|-------|
| Connection Pool Monitoring | 📋 | S | Backend |
| Query Performance Analysis | 📋 | M | Backend |
| Schema Drift Detection | 📋 | L | Backend |

---

## 🚀 v1.2.0 - The Great Refactoring (Strategic)

**Target Date**: Q2 2026 (Avril-Mai)
**Status**: 🔄 Active (33% complete)
**Theme**: Technical Excellence & Architecture

### 🎯 Objectifs de Refactoring

#### Phase 1: Core Domain (En cours)
| Module | Status | Priority |
|--------|--------|----------|
| ✅ Cilium (Network) | 🔄 | P0 |
| ✅ Database (PostgreSQL) | 🔄 | P0 |
| ✅ Health System | 📋 | P0 |
| ✅ Configuration | 📋 | P1 |

#### Phase 2: Infrastructure & Integration
| Module | Status | Priority |
|--------|--------|----------|
| Doctor (Diagnostic) | 📋 | P1 |
| Export (Reports) | 📋 | P2 |
| System (Auto-update) | 📋 | P2 |
| Translation (i18n) | 📋 | P3 |

#### Phase 3: External Integrations
| Module | Status | Priority |
|--------|--------|----------|
| Proxmox | 📋 | P2 |
| Home Assistant | 📋 | P2 |
| Weather | 📋 | P3 |
| Calendar | 📋 | P3 |
| Newsfeed | 📋 | P3 |

### 🧪 Quality Gates
- [ ] 100% modules dans architecture hexagonale
- [ ] >80% test coverage (unit + integration)
- [ ] Zero clippy warnings
- [ ] Documentation API complète (OpenAPI)
- [ ] Migration guide pour breaking changes

---

## 🔮 v1.3.0 - Intelligent Observability

**Target Date**: Q3 2026 (Juillet-Septembre)
**Status**: 📋 Planned
**Theme**: AI-Powered Insights & Cost Optimization

### 🤖 AI/ML Features (New)

#### Anomaly Detection
| Feature | Description | Effort |
|---------|-------------|--------|
| **Smart Alerting** | Réduction du bruit par ML (clustering des alertes) | L |
| **Predictive Capacity** | Prédiction de saturation (disk, memory) | L |
| **Error Correlation** | Corrélation automatique erreurs ↔ causes | M |
| **Seasonal Baselines** | Détection d'anomalies basée sur saisonnalité | M |

#### Intelligent Assistant
| Feature | Description | Effort |
|---------|-------------|--------|
| **Kusanagi Copilot** | Assistant conversationnel pour troubleshooting | L |
| **Auto-Runbooks** | Génération automatique de runbooks | M |
| **Root Cause Analysis** | Analyse automatique des causes racines | L |
| **Remediation Suggestions** | Suggestions de remédiation basées sur l'historique | M |

### 💰 Cost Management (New Track)

| Feature | Description | Effort |
|---------|-------------|--------|
| **Resource Right-Sizing** | Recommandations CPU/Memory requests/limits | M |
| ** orphaned Resources** | Détection des ressources orphelines (PVCs, LB, IPs) | M |
| **Namespace Cost Allocation** | Attribution des coûts par namespace/projet | L |
| **Spot Instance Advisor** | Recommandations pour utilisation de spots | M |

### 📊 Enhanced Observability

| Feature | Description | Effort |
|---------|-------------|--------|
| **Distributed Tracing** | Intégration Jaeger/Tempo | L |
| **Log Aggregation** | Centralisation Loki/ELK | L |
| **Service Mesh Metrics** | Istio/Linkerd observability | M |
| **eBPF Deep Dive** | Cilium Hubble flows avancés | M |

---

## 🛡️ v1.4.0 - Security & Compliance

**Target Date**: Q4 2026 (Octobre-Décembre)
**Status**: 💭 Ideation
**Theme**: Security Posture & Compliance

### Security Features
| Feature | Description | Priority |
|---------|-------------|----------|
| **RBAC Visualization** | Graphe des permissions RBAC | P0 |
| **CVE Scanning** | Intégration Trivy/Snyk continue | P0 |
| **Network Policies Audit** | Vérification des policies Cilium | P1 |
| **Pod Security Standards** | Enforcement et monitoringPSS | P1 |
| **Secret Rotation** | Alertes et automation rotation secrets | P2 |
| **Compliance Reports** | CIS Kubernetes Benchmark | P2 |

### AI Safety (from v1.3.0)
| Feature | Description | Status |
|---------|-------------|--------|
| **Prompt Injection Detection** | Protection contre attaques prompt | 📋 |
| **Output Validation** | Validation des réponses IA | 📋 |
| **Data Provenance** | Traçabilité des données IA | 📋 |

---

## 🏢 v2.0.0 - Enterprise Platform

**Target Date**: Q1-Q2 2027
**Status**: 💭 Ideation
**Theme**: Scale, Multi-tenancy, Ecosystem

### Core Enterprise Features
| Feature | Description | Effort |
|---------|-------------|--------|
| **Multi-Cluster** | Gestion centralisée de N clusters | XL |
| **SSO/Keycloak** | Authentification enterprise | L |
| **GitOps Integration** | Flux/ArgoCD deep integration | L |
| **Custom Dashboards** | Builder de dashboards personnalisés | XL |
| **API Gateway** | Rate limiting, API keys, versioning | M |
| **Multi-tenancy** | Isolation namespace hard | XL |

### Ecosystem
| Feature | Description | Effort |
|---------|-------------|--------|
| **Plugin System** | Architecture plugins WASM | XL |
| **Public API** | API externe documentée | L |
| **Mobile App** | Application iOS/Android | XL |
| **CLI Tool** | kubectl plugin pour Kusanagi | M |

---

## 🔄 Continuous Improvement Backlog

### Technical Debt
| Item | Priority | Effort |
|------|----------|--------|
| Migration legacy → hexagonal | P0 | XL |
| Tests unitaires (>80%) | P0 | XL |
| Tests E2E (Playwright) | P1 | L |
| Documentation technique | P1 | M |
| Benchmarks performances | P2 | M |

### Observability Enhancements
| Item | Priority | Effort |
|------|----------|--------|
| SLO/SLI Tracking | P1 | M |
| Error Budgets | P1 | M |
| Custom Metrics DSL | P2 | L |
| Alert Routing Rules | P1 | M |

### Developer Experience
| Item | Priority | Effort |
|------|----------|--------|
| Local Development (Kind) | P1 | M |
| Hot Reload | P2 | S |
| Debug Mode Verbose | P2 | S |
| Makefile Commands | P2 | S |

---

## 📊 Prioritization Framework

### Priority Matrix
```
Impact
  ↑
  │ P0: Multi-cluster      P0: Refactoring
  │ P1: Cost Mgmt          P0: Disk Monitor ✅
  │ P1: RBAC               P1: AI Assistant
  │ P2: Mobile             P2: Calendar
  │
  └────────────────────────────────→ Effort
       S        M        L        XL
```

### Decision Criteria
1. **User Impact** : Combien d'utilisateurs sont affectés?
2. **Technical Debt** : Réduit-il la dette technique?
3. **Strategic Alignment** : Soutient-il la vision produit?
4. **Effort/Reward** : Le ROI est-il positif?

---

## 🎯 2026 OKRs (Objectives & Key Results)

### O1: Architecture Excellence
- KR1: 100% modules en architecture hexagonale (Q2)
- KR2: >80% test coverage (Q3)
- KR3: Zero clippy warnings (Q2)

### O2: Intelligent Observability
- KR1: Déploiement Anomaly Detection (Q3)
- KR2: Réduction de 50% des faux positifs d'alertes (Q3)
- KR3: Prédiction de saturation avec 85% accuracy (Q4)

### O3: Cost Optimization
- KR1: Identification de 20% de ressources sous-utilisées (Q3)
- KR2: Recommandations right-sizing pour 100% des workloads (Q4)
- KR3: Dashboard cost allocation par équipe (Q4)

### O4: Developer Experience
- KR1: Setup local < 5 minutes (Q2)
- KR2: Documentation API 100% OpenAPI (Q2)
- KR3: CLI tool pour opérations courantes (Q4)

---

**Last Updated**: 2026-02-02
**Version**: 2.0
**Product Manager**: AI Assistant
**Engineering Lead**: Joseph Zacharie

**Changelog**:
- 2026-02-02: Ajout v1.2.5 (Disk Monitoring), roadmap étendue 2027, OKRs 2026
- 2026-02-01: Version initiale
