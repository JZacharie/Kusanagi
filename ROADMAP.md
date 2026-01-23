# Kusanagi - Product Roadmap 2026

> **Product Vision**: Kusanagi est la plateforme d'observabilité ultime pour Kubernetes - combinant monitoring en temps réel, auto-remédiation intelligente, et intégration personnelle dans une interface cyberpunk.

---

## 🎯 Product Strategy

### Core Pillars
1. **Observability First**: Vue complète de l'infrastructure K8s
2. **Action-Oriented**: Capacités de remédiation et contrôle
3. **Intelligence**: Insights automatisés via MCP et AI
4. **Personal Integration**: Hub centralisé pour infrastructure ET vie personnelle

### Success Metrics
- **Adoption**: Utilisation quotidienne par l'équipe
- **MTTR**: Réduction du temps de résolution des incidents
- **Coverage**: 100% des composants critiques monitorés
- **Code Quality**: Clippy zero-warnings & >80% test coverage
- **UX**: Score Lighthouse > 95

---

## 📅 Release Timeline

```
Q1 2026          Q2 2026          Q3 2026          Q4 2026
───────────────────────────────────────────────────────────
v0.8 ✅          v0.9 ✅          v1.0 🔄          v1.1 📋          v1.2 📋          v2.0
News Feed       Infra Monitor    Personal Hub     Data & Messaging Setup Wizard     Enterprise
```

---

## 🚀 Current Release: v0.8.0 - News Feed (SHIPPED ✅)

**Release Date**: January 2026  
**Status**: ✅ Completed  
**Theme**: Tech News Aggregation

### Delivered Features
- ✅ Hacker News integration (Docker/K8s filtering)
- ✅ Korben RSS feed parser
- ✅ GitHub trending repositories
- ✅ Multi-source aggregation with caching
- ✅ Cyberpunk UI with source filters
- ✅ Search and auto-refresh

### Impact
- Keeps team informed on latest tech trends
- Centralized news consumption
- Reduced context switching

---

## 🚀 Release: v0.9.0 - Infrastructure Monitoring (SHIPPED ✅)

**Release Date**: January 2026  
**Status**: ✅ Completed  
**Theme**: Extended Infrastructure Visibility

### Priority Features

#### P0 - Must Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| **Proxmox Integration** | L | Proxmox API access | Backend |
| VM/CT Status Dashboard | M | Proxmox Integration | Frontend |
| Resource Usage Tracking | M | Proxmox Integration | Backend |

#### P1 - Should Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| **Home Assistant Integration** | M | HA API access | Backend |
| Sensor Dashboard | S | HA Integration | Frontend |
| Automation Status | S | HA Integration | Backend |

#### P2 - Nice to Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| Device Tracking | S | HA Integration | Frontend |
| Proxmox Cluster View | M | Proxmox Integration | Frontend |

### Technical Requirements
- Proxmox API client (Rust)
- Home Assistant REST API integration
- New dashboard tabs
- Real-time sensor updates

### Success Criteria
- [x] All Proxmox VMs/CTs visible
- [x] Home Assistant sensors updating in real-time
- [ ] <500ms response time for dashboards
- [ ] Mobile-responsive design

---

## 🔨 Next Release: v1.0.0 - Personal Dashboard Hub

**Target Date**: February 2026  
**Status**: 🔄 In Progress  
**Theme**: Personal Productivity Integration

### Priority Features

#### P0 - Must Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| **Google Calendar Integration** | L | OAuth2 setup | Backend |
| Calendar Widget | M | Calendar API | Frontend |
| **Weather Multi-City** | M | Weather API key | Backend |
| ✅ Weather Widget (Lyon/Mexico/NYC) | S | Weather API | Frontend |

#### P1 - Should Have  
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| [/] **Slack Integration** | L | Slack Bot setup | Backend |
| ✅ Chat-Slack Sync | M | Slack API | Backend |
| ✅ Slack Notifications | S | Slack API | Backend |
| Event Notifications | M | Calendar API | Frontend |

#### P2 - Nice to Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| Multi-timezone Display | S | - | Frontend |
| Calendar Event Creation | M | Calendar API | Backend |
| Weather Alerts | S | Weather API | Backend |

### Technical Requirements
- Google OAuth2 flow
- Slack Bot Token & Events API
- OpenWeatherMap/WeatherAPI integration
- WebSocket for real-time updates

### Success Criteria
- [ ] Calendar events visible in dashboard
- [ ] Weather updates every 30min
- [ ] Slack messages sync bidirectionally
- [ ] <2s calendar load time

---

## 🚀 Future Release: v1.1.0 - Data & Messaging Monitoring

**Target Date**: March 2026  
**Status**: 📋 Planned  
**Theme**: Persistence, Messaging & Health Verification

### Priority Features

#### P0 - Must Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| **PostgreSQL Health Check** | M | PG Connection | Backend |
| Instance Status Monitor | S | Health Check | Frontend |
| **Database Verification** | M | PG Connection | Backend |
| **MQTT Broker Health** | M | MQTT Connection | Backend |
| Topic Explorer Dashboard | L | MQTT Broker Health | Frontend |

#### P1 - Should Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| Schema Integrity Scan | L | DB Verification | Backend |
| Size Tracking & Bloat | M | DB Verification | Backend |
| Connection Pool Stats | S | Health Check | Backend |
| MQTT Message Preview | M | Topic Explorer | Backend |
| Client Connection Alerts | S | MQTT Broker Health | Backend |

#### P2 - Nice to Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| Dynamic Query Profiling | L | - | Backend |
| Backup Status (WAL-G/PG Backrest) | M | - | Backend |

### Technical Requirements
- Rust PostgreSQL client (tokio-postgres/sqlx)
- Monitoring queries for `pg_stat_database` & `pg_stat_activity`
- MQTT v5.0 client integration (rumqttc)
- Disk usage tracking for PVs
- Cyberpunk-styled database & messaging health cards

### Success Criteria
- [ ] Real-time PostgreSQL & MQTT instance status
- [ ] Schema drift detection
- [ ] MQTT topic tree visualization
- [ ] <1s query time for health metrics

---

## 🚀 Future Release: v1.2.0 - Configuration & Onboarding

**Target Date**: June 2026  
**Status**: 📋 Planned  
**Theme**: Seamless Setup & Feature Discovery

### Priority Features

#### P0 - Must Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| **Interactive Setup Wizard** | L | - | Frontend |
| Environment Interview | M | Setup Wizard | Backend |
| **Format Validation Engine** | M | - | Backend |

#### P1 - Should Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| Feature Activation Dashboard | S | Format Validation | Frontend |
| Integrated Doc Tooltips | S | Setup Wizard | Frontend |
| Secret Masking & Protection | M | Setup Wizard | Backend |

#### P2 - Nice to Have
| Feature | Effort | Dependencies | Owner |
|---------|--------|--------------|-------|
| Auto-discovery of Local Services | L | - | Backend |
| Exportable Config Templates | S | Setup Wizard | Backend |

### Technical Requirements
- Step-by-step UI framework (Cyberpunk theme)
- RegEx & API-call based format validation
- Conditional rendering based on config presence
- Secure buffer for sensitive data entry

### Success Criteria
- [ ] Average setup time < 5 minutes
- [ ] Zero "Missing Env" errors after wizard completion
- [ ] 100% of required variables documented in-app
- [ ] Visual feedback for every successfully configured provider

---

## 🏢 Long-term Vision: v2.0.0 - Enterprise Features

**Target Date**: Q4 2026  
**Status**: 💭 Ideation  
**Theme**: Security, Scale, Intelligence

### Strategic Initiatives

#### Security & Compliance
- [ ] Keycloak/OIDC Authentication
- [ ] RBAC with granular permissions
- [ ] Audit logging (all actions)
- [ ] Secret management (Vault)
- [ ] SOC2 compliance readiness

#### Scalability
- [ ] Multi-cluster support
- [ ] Horizontal scaling (replicas)
- [ ] Redis distributed cache
- [ ] PostgreSQL persistence
- [ ] Load balancer support

#### Intelligence & Automation
- [ ] AI-powered anomaly detection
- [ ] Predictive alerts
- [ ] Auto-remediation engine
- [ ] Runbook automation
- [ ] Cost optimization recommendations

#### Agent Architecture
- [ ] Node agents (DaemonSet)
- [ ] Local diagnostics
- [ ] Self-healing capabilities
- [ ] Distributed tracing

---

## 📊 Feature Prioritization Framework

### Priority Levels
- **P0 (Must Have)**: Core functionality, blocks release
- **P1 (Should Have)**: Important but can be deferred
- **P2 (Nice to Have)**: Enhancement, low impact if missing

### Effort Estimation
- **S (Small)**: 1-2 days
- **M (Medium)**: 3-5 days
- **L (Large)**: 1-2 weeks
- **XL (Extra Large)**: 2+ weeks

### Decision Criteria
1. **User Impact**: How many users benefit?
2. **Strategic Alignment**: Fits product vision?
3. **Technical Feasibility**: Can we build it?
4. **Dependencies**: What's blocking it?
5. **ROI**: Effort vs. value ratio

---

## 🔄 Continuous Improvements (Ongoing)

### Performance
- [ ] Optimize bundle size (<150KB)
- [ ] Reduce API response times (<100ms)
- [ ] Improve cache hit rates (>80%)
- [ ] Monitor memory usage (<100MB)

### Code Quality
- [ ] Rust Architecture Audit (Module boundaries & Clean Code)
- [ ] Implement Performance Profiling (Serialization & K8s API)
- [ ] Add unit tests (>80% coverage)
- [ ] Integration tests for APIs
- [ ] E2E tests for critical flows
- [ ] Reduce technical debt

### UX/UI
- [ ] Accessibility improvements (WCAG AA)
- [ ] Mobile optimization
- [ ] Dark/Light theme refinement
- [ ] Loading state improvements

### DevOps
- [ ] CI/CD pipeline optimization
- [ ] Automated dependency updates (Renovate)
- [ ] Security scanning (Trivy)
- [ ] Performance monitoring (OpenObserve)

---

## 📋 Backlog (Unprioritized)

### Monitoring Enhancements
- [ ] Cost tracking per namespace
- [ ] Certificate expiration monitoring
- [ ] Database backup status
- [ ] Custom dashboards (drag-drop)

### Integrations
- [ ] Jira integration
- [ ] PagerDuty alerts
- [ ] Datadog metrics
- [ ] GitHub Actions status

### Advanced Features
- [ ] Multi-language support (i18n)
- [ ] Custom alert rules
- [ ] Report scheduling
- [ ] API rate limiting

---

## 🎯 OKRs (Objectives & Key Results)

### Q1 2026
**Objective**: Establish Kusanagi as the go-to K8s monitoring tool

**Key Results**:
- ✅ KR1: Ship News Feed feature (v0.8.0)
- 🔄 KR2: Rust Architecture Audit & Cleanup
- 🔄 KR3: Document architecture and roadmap
- 📋 KR4: Achieve 90% uptime in production
- 📋 KR5: Reduce average page load to <1s

### Q2 2026 (Planned)
**Objective**: Expand monitoring coverage beyond K8s

**Key Results**:
- KR1: Ship Proxmox + HA integration (v0.9.0)
- KR2: Ship Database & Storage Monitoring (v1.1.0)
- KR3: Monitor 100% of infrastructure components
- KR4: Achieve <5min MTTR for common issues
- KR5: Onboard 3+ team members

### Q3 2026 (Planned)
**Objective**: Optimize User Onboarding & Experience

**Key Results**:
- KR1: Ship Guided Configuration Interview (v1.2.0)
- KR2: Reduce configuration-related support issues by 50%
- KR3: Implement real-time feature activation dashboard

---

## 📞 Stakeholder Communication

### Weekly Updates
- **What**: Progress on current sprint
- **When**: Every Monday
- **Where**: Slack #kusanagi channel

### Monthly Reviews
- **What**: Demo + roadmap review
- **When**: Last Friday of month
- **Where**: Team meeting

### Quarterly Planning
- **What**: OKR review + next quarter planning
- **When**: End of quarter
- **Where**: Strategic planning session

---

## 🔧 Development Process

### Sprint Cycle
- **Duration**: 2 weeks
- **Planning**: Monday Week 1
- **Review**: Friday Week 2
- **Retrospective**: Friday Week 2

### Definition of Done
- [ ] Code reviewed and approved
- [ ] Tests passing (when applicable)
- [ ] Documentation updated
- [ ] Deployed to production
- [ ] Stakeholders notified

### Release Process
1. Feature freeze (3 days before release)
2. QA testing
3. Release notes drafted
4. Deploy to production
5. Monitor for 24h
6. Announce release

---

## 📚 Resources

- **Repository**: `JZacharie/Kusanagi`
- **Documentation**: `/ARCHITECTURE.md`, `/README.md`
- **Deployment**: ArgoCD (`kusanagi` namespace)
- **Monitoring**: OpenObserve
- **Chat**: Slack #kusanagi

---

**Last Updated**: 2026-01-21  
**Product Manager**: AI Assistant  
**Engineering Lead**: Joseph Zacharie
