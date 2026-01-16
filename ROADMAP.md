# Kusanagi - Roadmap & Features

## 🎯 Vision
Kusanagi est un dashboard de monitoring et de gestion pour infrastructure Kubernetes, inspiré par Ghost in the Shell.

---

## ✅ Fonctionnalités Implémentées

### v0.1.0 - Base
- [x] Interface web cyberpunk (Ghost in the Shell theme)
- [x] Health check endpoint `/health`
- [x] Serveur Actix-web performant

### v0.2.0 - ArgoCD Monitoring
- [x] Compteur d'applications ArgoCD (OK/Erreurs)
- [x] Liste des applications en erreur
- [x] Durée depuis laquelle une app est en erreur
- [x] Statuts: Healthy, Progressing, Unknown, OutOfSync
- [x] ClusterRole RBAC pour accès aux Applications

### v0.2.1 - Smart Issue Detection (current)
- [x] **Catégorisation intelligente** : Issues réelles vs Upgrades disponibles
- [x] **Bouton Sync** pour déclencher la synchronisation ArgoCD
- [x] **Liens directs ArgoCD** vers chaque application
- [x] Dual tables: Issues et Upgrades séparés
- [x] RBAC avec permission `patch` pour le sync

### v0.3.0 - Node Monitoring
- [x] Section Cluster Nodes avec métriques par node
- [x] CPU / RAM capacity affichés
- [x] Nombre de Pods par node
- [x] Uptime du node
- [x] Pods en erreur sur chaque node
- [x] **Badge architecture** avec couleurs différentes :
  - AMD64 = Violet/Purple
  - ARM64 = Rose/Pink

### v0.4.0 - Enhanced Dashboard (current)
- [x] **Quick Navigation Bar** - Stats cluster et liens externes
- [x] Compteur de namespaces
- [x] Compteur de PVCs + capacité totale
- [x] **Liens externes** : Grafana, ArgoCD, Homepage, OpenObserve
- [x] **Section PVC Monitoring** - Table des PVCs avec capacité et status
- [x] API `/api/cluster/overview` pour stats cluster

### v0.4.0 - RUM & Observabilité
- [ ] **Real User Monitoring (RUM)** - Intégration OpenObserve
  - [ ] Tracking des actions utilisateur
  - [ ] Session replay
  - [ ] Performance monitoring
  - [ ] Error tracking
- [ ] Inspiré de demo-RUM

### v0.5.0 - Chatbot & MCP Integration
- [ ] **Chatbot intégré** - Interroger le status du cluster
- [ ] **MCP Kubernetes** - Accès aux ressources K8s
- [ ] **MCP Cilium** - Monitoring réseau et policies
- [ ] **MCP Steampipe** - Requêtes SQL sur l'infrastructure
- [ ] **MCP Trivy S3** - Lecture des alertes Trivy stockées en S3

---

## 📋 Backlog

### Sécurité
- [ ] Authentification (Keycloak/OIDC)
- [ ] RBAC granulaire
- [ ] Audit logging

### Monitoring Additionnel
- [ ] Pods en CrashLoopBackOff
- [ ] Events Kubernetes récents
- [ ] Métriques Prometheus embedded
- [ ] Alertes AlertManager

### UX/UI
- [ ] Dark/Light mode toggle
- [ ] Notifications temps réel (WebSocket)
- [ ] Export de rapports
- [ ] Dashboard personnalisables

---

## 🔧 Stack Technique

- **Backend**: Rust + Actix-web
- **Frontend**: Vanilla JS + CSS (Cyberpunk theme)
- **Kubernetes Client**: kube-rs
- **Observabilité**: OpenObserve RUM
- **Deployment**: Helm Chart + ArgoCD

---

## 📝 Notes

- Déployé sur namespace `kusanagi`
- Accessible via `kusanagi.p.zacharie.org`
- Intégré à Homepage via annotations gethomepage.dev
