# Kusanagi - Roadmap & Features

## 🎯 Vision
Kusanagi est un dashboard de monitoring et de gestion pour infrastructure Kubernetes, inspiré par Ghost in the Shell.

---

## ✅ Fonctionnalités Implémentées

### v0.1.0 - Base
- [x] Interface web cyberpunk (Ghost in the Shell theme)
- [x] Health check endpoint `/health`
- [x] Serveur Actix-web performant

### v0.2.0 - ArgoCD Monitoring (en cours)
- [x] Compteur d'applications ArgoCD (OK/Erreurs)
- [x] Liste des applications en erreur
- [x] Durée depuis laquelle une app est en erreur
- [x] Statuts: Healthy, Progressing, Unknown, OutOfSync
- [x] ClusterRole RBAC pour accès aux Applications

---

## 🚧 Fonctionnalités Planifiées

### v0.3.0 - Enhanced Dashboard
- [ ] **Logo personnalisé** - Ajouter logo.png dans l'application
- [ ] **Menu latéral gauche** - Navigation pour les différentes sections
- [ ] **Compteur de namespaces** - Nombre total de namespaces
- [ ] **Liens externes** :
  - [ ] Lien vers Homepage
  - [ ] Lien vers Grafana
- [ ] **PVC Monitoring** :
  - [ ] Identifier les PVC qui gaspillent de la place (sous-utilisés)
  - [ ] Identifier les PVC qui saturent (presque pleins)

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
