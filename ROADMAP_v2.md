# Kusanagi - Product Roadmap 2026-2027 (Revised)

> **Product Vision**: Kusanagi est la plateforme d'observabilité ultime pour Kubernetes - combinant monitoring en temps réel, auto-remédiation intelligente, et intégration personnelle dans une interface cyberpunk.
> 
> **Version**: 2.1 (Révision stratégique - Février 2026)

---

## 🎯 Strategic Pivot: Stability First

### Leçons apprises (Février 2026)
- **22 modules legacy** restants sur 35 (63% de dette technique)
- **~15% test coverage** vs objectif 80% = écart critique
- **96 warnings** Rust (compilation propre mais qualité perfectible)
- **Architecture hexagonale** : 13/35 modules (37%) - progression réelle mais lente

### Nouvelle approche : "Crawl, Walk, Run"
| Phase | Focus | Durée | Sortie |
|-------|-------|-------|--------|
| **Crawl** | Stabilité & Fondations | Q1-Q2 2026 | v1.2.x |
| **Walk** | Intelligence & Insights | Q3-Q4 2026 | v1.3.x |
| **Run** | Scale & Enterprise | 2027 | v2.0.x |

---

## 📅 Planning Format: Now / Next / Later

### 🔴 NOW (Q1 2026: Mars-Mai) - Foundation
**Theme**: "Production Ready Core"

#### P0 - Critical Path
| Feature | Status | Effort | Definition of Done |
|---------|--------|--------|-------------------|
| **Graceful Shutdown** | 📋 | S | SIGTERM handling, drain connections, zero-downtime deploy |
| **Health Check API** | 📋 | S | `/health`, `/ready`, `/live` endpoints pour K8s probes |
| **Structured Logging** | 📋 | M | JSON format, correlation IDs, configurable levels |
| **Config Hot-Reload** | 📋 | M | File watcher, validation, rollback on error |
| **Cilium Hexagonal** | 🔄 | L | Migration complète, tests >80%, docs |
| **Database Hexagonal** | 🔄 | L | PostgreSQL port, connection pool monitoring |

#### P1 - Quality Gates
| Feature | Target | Current |
|---------|--------|---------|
| Modules hexagonaux | 50% (17/35) | 37% (13/35) |
| Test coverage | 35% | ~15% |
| Clippy warnings | <50 | 0 (juste fixé!) |
| E2E Tests (Playwright) | 5 scénarios | 0 |

#### Technical Debt Priority
```markdown
v1.2.0 (Mars): Core Infrastructure
├── Cilium → Hexagonal
├── Database → Hexagonal  
├── Health System → Hexagonal
└── Graceful Shutdown + Health API

v1.2.5 (Avril): Observability Foundation
├── Structured Logging
├── Config Hot-Reload
├── Connection Pool Monitoring
└── 40% Test Coverage

v1.2.8 (Mai): Polish & Documentation
├── Doctor → Hexagonal
├── i18n Framework
├── OpenAPI Documentation
└── 50% Test Coverage
```

---

### 🟡 NEXT (Q2-Q3 2026: Juin-Septembre) - Intelligence
**Theme**: "Data-Driven Insights"

> **Prérequis strict** : Avoir 3 mois de données historiques avant tout ML/AI

#### Phase 1: Data Pipeline (Juin-Juillet)
| Feature | Description | Effort | Dépendances |
|---------|-------------|--------|-------------|
| **Time-Series DB** | TimescaleDB pour métriques historiques | M | Database hexagonal |
| **Metrics Retention** | Policies 7j/30j/1an par criticité | S | TSDB |
| **Baseline Calculation** | Moyennes mobiles, percentiles | M | 30j de données |

#### Phase 2: Statistical Intelligence (Août-Septembre)
| Feature | Description | Effort |
|---------|-------------|--------|
| **Threshold Intelligence** | Alertes basées sur écarts vs baseline (pas de ML) | M |
| **Capacity Forecasting** | Trend lineaire simple pour saturation disk/memory | M |
| **Alert Correlation** | Groupement temporel (même timeframe = même incident) | S |
| **Cost Visibility** | Premier dashboard coût (pas de recommandations) | L |

#### P1 - Continue Refactoring
| Feature | Target |
|---------|--------|
| Modules hexagonaux | 75% (26/35) |
| Test coverage | 60% |
| Documentation | 100% modules critiques |

---

### 🟢 LATER (Q4 2026 - 2027) - Scale
**Theme**: "Enterprise & AI"

> **Condition de démarrage** : 60% coverage + 75% modules hexagonaux + 6 mois de données

#### v1.3.x (Q4 2026) - Advanced Intelligence
- **Anomaly Detection ML** : Isolation Forest sur métriques (vrai ML)
- **Kusanagi Copilot** : Assistant conversationnel (LLM local pour privacy)
- **Auto-Remediation v1** : Runbooks simples (restart pod, scale deployment)
- **Cost Optimization** : Recommandations right-sizing basées sur usage réel

#### v1.4.x (Q1 2027) - Security & Compliance
- **RBAC Visualization** : Graphe permissions (d3.js/cytoscape)
- **CVE Scanning** : Intégration Trivy continue
- **Network Policy Audit** : Validation Cilium Hubble
- **Compliance Reports** : CIS Kubernetes Benchmark auto

#### v2.0.x (2027) - Enterprise Platform
- **Multi-Cluster** : Gestion centralisée N clusters
- **SSO/Keycloak** : Auth enterprise
- **Plugin System (WASM)** : Extensibilité
- **CLI Tool** : `kubectl kusanagi` plugin

---

## 🚧 Risks & Mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Refactoring break prod** | High | Critical | Feature flags, canary deploys, rollback automatisé |
| **1 dev bottleneck** | High | High | Scope réduit, pas de parallélisation agressive |
| **AI features sans données** | Medium | Medium | Phase "Statistical Intelligence" obligatoire avant ML |
| **Dependency drift** | Medium | Medium | Renovate bot + tests d'intégration |
| **Burnout** | Medium | Critical | Milestones de 4 semaines max, features coupables |

---

## ✅ Definition of Done (Refactoring)

Chaque module migré vers l'architecture hexagonale DOIT avoir :

```markdown
### Code Quality
- [ ] 100% code dans `domain/`, `application/`, `infrastructure/`, `interfaces/`
- [ ] 0 `unwrap()` ou `expect()` (sauf cas légitimes documentés)
- [ ] Error handling cohérent avec `KusanagiError`
- [ ] Clippy 0 warnings sur le module

### Testing
- [ ] Tests unitaires >80% coverage du module
- [ ] Tests d'intégration (si dépendances externes)
- [ ] Mocks pour tous les ports (repository pattern)
- [ ] CI passe (cargo test, clippy, fmt)

### Documentation
- [ ] OpenAPI spec à jour (si endpoints HTTP)
- [ ] README.md dans le module si complexe
- [ ] ADR (Architecture Decision Record) si trade-offs

### Observability
- [ ] Metrics (latency, error rate, throughput)
- [ ] Tracing spans pour opérations async
- [ ] Logs structurés (pas de `println!`)
```

---

## 📊 Revised OKRs 2026

### O1: Production Stability
**Focus**: Arrêter d'ajouter des features, consolider l'existant

| Key Result | Target | Q1 | Q2 | Q3 | Q4 |
|------------|--------|----|----|----|----|
| **Graceful shutdown** | 100% | ✅ | ✅ | ✅ | ✅ |
| **Health checks** | 100% | ✅ | ✅ | ✅ | ✅ |
| **Uptime** | 99.9% | - | 99% | 99.5% | 99.9% |
| **MTTR** | <15min | 45min | 30min | 20min | 15min |

### O2: Architecture Excellence
**Focus**: Dette technique contrôlée, pas éliminée

| Key Result | Target | Q1 | Q2 | Q3 | Q4 |
|------------|--------|----|----|----|----|
| **Hexagonal modules** | 75% | 50% | 75% | 85% | 90% |
| **Test coverage** | 70% | 35% | 55% | 70% | 75% |
| **Clippy warnings** | 0 | 0 | 0 | 0 | 0 |
| **E2E scenarios** | 20 | 5 | 10 | 15 | 20 |

### O3: Intelligent Observability
**Focus**: Données d'abord, intelligence après

| Key Result | Target | Q1 | Q2 | Q3 | Q4 |
|------------|--------|----|----|----|----|
| **Metrics retention** | 90j | 30j | 60j | 90j | 90j |
| **Baseline coverage** | 100% | 0% | 50% | 100% | 100% |
| **Alert noise reduction** | 50% | 0% | 20% | 40% | 50% |
| **Cost visibility** | 100% | 0% | 0% | 50% | 100% |

### O4: Developer Velocity
**Focus**: Temps de setup, feedback loop

| Key Result | Target | Q1 | Q2 | Q3 | Q4 |
|------------|--------|----|----|----|----|
| **Setup local** | <5min | 30min | 15min | 10min | 5min |
| **Hot reload** | ✅ | ❌ | ✅ | ✅ | ✅ |
| **CI build time** | <5min | 10min | 7min | 5min | 5min |

---

## 🎨 User Experience Roadmap

### Theme: "Cyberpunk Dashboard Evolution"

#### v1.2.x - Consistency
- [ ] Design System unifié (tokens CSS)
- [ ] Dark/Light mode auto (system preference)
- [ ] Mobile responsive (tablettes)

#### v1.3.x - Interactivity
- [ ] Real-time WebSocket updates (pas de polling)
- [ ] Custom dashboard builder (drag & drop)
- [ ] Keyboard shortcuts (vim-style)

#### v2.0.x - Immersion
- [ ] 3D cluster visualization (Three.js)
- [ ] AR/VR mode (expérimental)
- [ ] Voice commands

---

## 🔄 Dependency Management Strategy

### Phase 1: Security & Stability (Q1 2026)
```toml
# Cargo.toml - Minimal changes
[dependencies]
# Only security updates + bug fixes
aws-sdk-s3 = "1.121"  # ✓ Déjà fait
rumqttc = "0.25"      # ✓ Déjà fait
```

### Phase 2: Major Upgrades (Q2 2027)
- Planifier upgrades breaking (kube-rs, actix-web, sqlx)
- Feature flags pour rollback facile

---

## 📈 Success Metrics Dashboard

```markdown
## Daily
- [ ] Build passe (CI green)
- [ ] Clippy 0 warnings
- [ ] Test coverage > previous day

## Weekly
- [ ] Modules legacy restants
- [ ] MTTR moyen
- [ ] Alertes bruit/false positive ratio

## Monthly
- [ ] User engagement (time on platform)
- [ ] Features shipped vs planned
- [ ] Dette technique index (complexité cyclomatique)
```

---

## 🎯 Immediate Next Steps (This Week)

1. **Finaliser v1.2.0 scope**
   - [ ] Choisir: Cilium vs Database pour prochain module
   - [ ] Estimer effort précis (story points)
   - [ ] Définir "good enough" (pas de perfectionnisme)

2. **Setup production readiness**
   - [ ] Implémenter `/health` endpoint
   - [ ] Ajouter graceful shutdown
   - [ ] Configurer probes Kubernetes

3. **Documentation**
   - [ ] ADR: "Why Hexagonal Architecture"
   - [ ] Guide: "How to migrate a module"
   - [ ] Runbook: "Rollback procedure"

---

## 📝 Changelog Roadmap

| Version | Date | Changes |
|---------|------|---------|
| 2.1 | 2026-02-03 | Révision stratégique: Stability First, format Now/Next/Later, OKRs réalistes |
| 2.0 | 2026-02-02 | Ajout v1.2.5, roadmap étendue 2027 |
| 1.0 | 2026-02-01 | Version initiale |

---

**Product Manager**: AI Assistant  
**Engineering Lead**: Joseph Zacharie  
**Last Updated**: 2026-02-03  
**Status**: ✅ Approved for Q1 2026

---

## 💡 Appendix: Module Migration Priority Matrix

| Module | Usage | Complexity | Business Value | Priority |
|--------|-------|------------|----------------|----------|
| Cilium | High | High | Critical | P0 |
| Database | High | Medium | Critical | P0 |
| Health | High | Low | High | P1 |
| Doctor | Medium | Medium | Medium | P1 |
| Proxmox | Low | Medium | Low | P3 |
| Home Assistant | Low | Low | Personal | P3 |
| Weather | Low | Low | Personal | P3 |

**Règle**: P0 = maintenant, P1 = Q2, P2 = Q3, P3 = 2027
