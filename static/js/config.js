/**
 * Kusanagi Configuration Centralisée
 * Tous les paramètres configurables en un seul endroit
 */

const KusanagiConfig = {
    // Version
    version: '0.3.0',

    // API Configuration
    api: {
        baseUrl: '',  // Relative - same origin
        timeout: 30000,  // 30s default timeout
        endpoints: {
            // Kubernetes - canonical paths (/api/k8s/*)
            nodes: '/api/k8s/nodes',
            pods: '/api/k8s/pods',
            services: '/api/services',
            ingress: '/api/ingress',
            storage: '/api/storage',
            backups: '/api/backups',
            argocd: '/api/argocd/status',
            cluster: '/api/k8s/cluster',

            // Infrastructure
            proxmoxVms: '/api/proxmox/vms',
            proxmoxContainers: '/api/proxmox/containers',
            proxmoxNodes: '/api/proxmox/nodes',

            // Integrations
            weather: '/api/weather/current',
            haSensors: '/api/ha/sensors',
            haDevices: '/api/ha/devices',
            haAutomations: '/api/ha/automations',
            security: '/api/security/vulnerabilities',

            // System
            systemStatus: '/api/system/status',
            systemLogs: '/api/system/logs',
            dbHealth: '/api/database/health',
            metrics: '/api/metrics'
        }
    },

    // Refresh Intervals (ms)
    intervals: {
        fast: 10000,      // 10s - System status
        normal: 30000,    // 30s - Most modules
        slow: 300000,     // 5min - Weather
        cache: 180000     // 3min - Cache TTL
    },

    // Cache Configuration
    cache: {
        ttl: 180000,  // 3 minutes
        prefix: 'kusanagi_',
        keys: {
            services: 'services_cache',
            ingress: 'ingress_cache',
            metrics: 'metrics_cache',
            news: 'news_cache'
        }
    },

    // UI Configuration
    ui: {
        themes: ['cyberpunk', 'modern', 'loot-drop', 'fundy', 'white'],
        defaultTheme: 'cyberpunk',
        defaultLocale: 'fr',
        supportedLocales: ['en', 'fr'],
        pagination: {
            storage: 10,
            events: 20
        }
    },

    // Feature Flags
    features: {
        websocket: true,
        pwa: true,
        rum: true,      // Real User Monitoring
        debug: false    // Debug mode
    }
};

// Freeze to prevent accidental modification
Object.freeze(KusanagiConfig);
Object.freeze(KusanagiConfig.api);
Object.freeze(KusanagiConfig.api.endpoints);
Object.freeze(KusanagiConfig.intervals);
Object.freeze(KusanagiConfig.cache);
Object.freeze(KusanagiConfig.ui);
Object.freeze(KusanagiConfig.features);

window.KusanagiConfig = KusanagiConfig;
