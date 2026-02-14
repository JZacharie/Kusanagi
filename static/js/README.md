# Kusanagi JavaScript Architecture

## 📁 Structure des Fichiers

```
js/
├── config.js              # Configuration centralisée (endpoints, intervals, TTLs)
│
├── Core (Fondation)
│   ├── api.js            # Client HTTP avec enveloppe standardisée
│   ├── core.js           # WebSocket et navigation
│   ├── utils.js          # Utilitaires partagés
│   ├── theme.js          # Gestion des 3 thèmes visuels
│   ├── debug.js          # Outils de développement
│   ├── ansi-parser.js    # Parsing des logs ANSI
│   └── error-boundary.js # Gestion globale des erreurs
│
├── Infrastructure
│   ├── api-tracker.js    # Performance monitoring
│   ├── rum.js            # OpenObserve RUM
│   ├── page-loader.js    # Chargement lazy des contenus
│   └── pwa.js            # Service Worker
│
├── k8s/                  # Modules Kubernetes
│   ├── state.js          # Cache localStorage avec TTL
│   ├── pods.js           # Monitoring pods (K8sPods)
│   ├── nodes.js          # Monitoring nœuds (K8sNodes)
│   ├── services.js       # Services & Ingress (K8sServices)
│   ├── storage.js        # PVC & Backups (K8sStorage)
│   ├── argocd.js         # Intégration ArgoCD (K8sArgo)
│   └── main.js           # Façade K8sManager
│
└── Dashboard Modules
    ├── dashboard.js      # Orchestrateur principal + i18n
    ├── sidebar.js        # Navigation
    ├── proxmox.js        # VMs Proxmox
    ├── homeassistant.js  # Home Assistant
    ├── weather.js        # Météo
    ├── monitors.js       # Alertes
    ├── security.js       # Scan Trivy
    ├── cilium-network.js # Visualisation Cilium (anciennement network.js)
    ├── mqtt.js           # MQTT
    ├── setup.js          # Configuration wizard
    └── system.js         # Logs système
```

## 🔄 Ordre de Chargement

1. **config.js** - Doit être chargé en premier
2. **Core** - Fondation (api, utils, theme, etc.)
3. **Infrastructure** - Monitoring et tracking
4. **k8s/state.js** - État avant les autres modules k8s
5. **k8s/*.js** - Modules Kubernetes
6. **Dashboard Modules** - Intégrations métier
7. **dashboard.js, sidebar.js** - UI principale

## 🎯 Patterns

### API Standardisée
```javascript
// Utiliser toujours api.get/post/delete
const data = await api.get(KusanagiConfig.api.endpoints.nodes);
```

### Configuration
```javascript
// Accès aux constantes via KusanagiConfig
const ttl = KusanagiConfig.cache.ttl;
const interval = KusanagiConfig.intervals.normal;
```

### Module Pattern
```javascript
const MonModule = {
    init() { /* ... */ },
    async fetchData() {
        try {
            const data = await api.get(KusanagiConfig.api.endpoints.xxx);
            this.render(data);
        } catch (error) {
            console.error('Error:', error);
        }
    },
    render(data) { /* ... */ }
};
window.MonModule = MonModule;
```

## ⚠️ Renommages Récents

- `network.js` → `cilium-network.js` (visualisation réseau Cilium)
- `k8s/network.js` → `k8s/services.js` (Services/Ingress K8s)
- `K8sNetwork` → `K8sServices` (nom de l'objet global)

## 📊 Modules K8s (Façade)

Le `K8sManager` expose une API unifiée qui délègue aux sous-modules:

```javascript
// Appel via la façade
K8sManager.fetchServices();     // → K8sServices.fetchServices()
K8sManager.fetchPodsStatus();   // → K8sPods.fetchPodsStatus()
K8sManager.sortStorage(field);  // → K8sStorage.sortStorage(field)
```
