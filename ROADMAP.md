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

### v0.5.0 - Kubernetes Events Monitoring
- [x] **Section Events** - Events Kubernetes de la dernière heure
- [x] Stats: total, warnings, normal
- [x] Table avec type, objet, reason, message, age, count
- [x] Warnings affichés en premier
- [x] API `/api/events` pour les events K8s

---

## 🚧 Fonctionnalités Planifiées

### v0.6.0 - RUM & Observabilité (current)
- [x] **Module RUM** (`rum.js`) - Real User Monitoring vanilla JS
- [x] Tracking page load (load time, DOM ready, TTFB)
- [x] Tracking erreurs JavaScript et promesses non gérées
- [x] Tracking interactions utilisateur (clics sur boutons/liens)
- [x] Tracking navigation et visibilité
- [x] Stockage session pour historique des events
- [ ] Intégration OpenObserve (future)

### v0.5.0 - Chatbot & MCP Integration
- [ ] **Chatbot intégré** - Interroger le status du cluster
- [ ] **Stockage conversations S3** - Historique des chats sur MinIO (192.168.0.170) pour analyse et features proactives
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
- [x] Pods en CrashLoopBackOff
- [ ] Events Kubernetes récents
- [ ] Métriques Prometheus embedded
- [ ] Alertes AlertManager

### UX/UI
- [x] Dark/Light mode toggle
- [x] Notifications temps réel (WebSocket)
- [ ] **Tri et recherche sur les tableaux** - Colonnes triables + barre de recherche
- [x] **Liens Ingress clickables** - Hosts en HTTPS cliquables vers les URLs
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

---

## ⚠️ Actions Correctives (Analyse Events Warning - 2026-01-17)

### 🔴 Critique - À corriger immédiatement

#### Redis Sentinel Timeouts (`redis`, `redis-s`)
- **Problème**: Liveness/Readiness probes timeout sur port 26379 (Sentinel)
- **Action**: 
  - [ ] Augmenter les timeouts des probes (timeoutSeconds: 10)
  - [ ] Vérifier la charge CPU/RAM des pods Redis
  - [ ] Valider la configuration Sentinel

#### N8N Pods Unhealthy (`n8n`, `n8n-dev`)  
- **Problème**: Connection refused sur port 5678
- **Action**:
  - [ ] Vérifier les logs N8N pour erreurs de démarrage
  - [ ] Augmenter initialDelaySeconds sur les probes
  - [ ] Vérifier les ressources allouées (OOM?)

#### Guacamole-SBX Sync Failed
- **Problème**: `envFrom` avec configMapRef/secretRef vides
- **Action**:
  - [x] ✅ Corrigé - Commenté la section envFrom dans values.yaml

### 🟠 Important - À planifier

#### ArgoCD HPA Missing Resource Requests
- **Problème**: `FailedGetResourceMetric` - memory request manquant
- **Action**:
  - [ ] Ajouter `resources.requests.memory` sur argocd-repo-server
  - [ ] Ajouter `resources.requests.memory` sur argocd-server

#### Guacamole-SBX HPA Missing CPU Request
- **Problème**: `FailedGetResourceMetric` - CPU request manquant
- **Action**:
  - [ ] Ajouter `resources.requests.cpu` sur guacamole-sbx-client

#### OpenObserve Backup Cluster Not Found
- **Problème**: `FindingCluster - Unknown cluster o2-openobserve-postgres`
- **Action**:
  - [ ] Vérifier la configuration CloudNativePG
  - [ ] Valider le nom du cluster PostgreSQL dans le Backup CRD

### 🟡 Mineur - À surveiller

#### DNS Nameserver Limits Exceeded (ArgoCD)
- **Problème**: Trop de nameservers configurés
- **Action**:
  - [ ] Réduire le nombre de nameservers dans la config DNS
  - [ ] Prioritiser les DNS internes

#### Trivy Scan BackoffLimitExceeded
- **Problème**: Job `scan-vulnerabilityreport` en échec
- **Action**:
  - [ ] Vérifier les logs du job Trivy
  - [ ] Augmenter le backoffLimit si timeout
  - [ ] Vérifier la connectivité au registry

#### Karakeep/Jellyfin Probe Timeouts
- **Problème**: Context deadline exceeded sur probes
- **Action**:
  - [ ] Augmenter timeoutSeconds des probes
  - [ ] Vérifier la performance de l'application
