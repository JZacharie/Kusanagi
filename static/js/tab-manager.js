/**
 * TabManager - Gestionnaire centralisé des onglets
 * Chaque onglet ne charge ses données que quand il est actif
 */

const TabManager = {
    // Configuration des onglets et leur intervalle de rafraîchissement (ms)
    // null = pas de polling automatique
    TABS: {
        // K8s tabs
        argocd: { interval: 30000, module: 'K8sArgo', fetch: 'fetchArgoStatus' },
        nodes: { interval: 60000, module: 'K8sNodes', fetch: 'fetchNodesStatus' },
        pods: { interval: 30000, module: 'K8sPods', fetch: 'fetchPodsStatus' },
        services: { interval: 300000, module: 'K8sServices', fetch: 'fetchServices' },
        ingress: { interval: 300000, module: 'K8sServices', fetch: 'fetchIngress' },
        storage: { interval: 60000, module: 'K8sStorage', fetch: 'fetchStorageStatus' },
        backups: { interval: 60000, module: 'K8sStorage', fetch: 'fetchBackupsStatus' },
        
        // Other tabs
        proxmox: { interval: 30000, module: 'ProxmoxDashboard', fetch: 'loadData' },
        homeassistant: { interval: 60000, module: 'HomeAssistantDashboard', fetch: 'loadData' },
        weather: { interval: 600000, module: 'WeatherDashboard', fetch: 'refreshWeather' },
        monitors: { interval: 30000, module: 'MonitorsManager', fetch: 'fetchMonitors' },
        security: { interval: 300000, module: 'SecurityDashboard', fetch: 'loadSecurityData' },
        mqtt: { interval: 30000, module: 'MqttManager', fetch: 'fetchInitialData' },
        metrics: { interval: 30000, module: 'MetricsManager', fetch: 'loadMetrics' },
        network: { interval: 30000, module: 'CiliumNetwork', fetch: 'fetchNetworkData' },
        
        // No auto-refresh
        chat: { interval: null },
        news: { interval: 60000, module: 'NewsManager', fetch: 'fetchNews' },
        setup: { interval: null },
        docs: { interval: null },
        logs: { interval: null },
        calendar: { interval: null },
        ha: { interval: null },
        system: { interval: 30000, module: 'KusanagiSystem', fetch: 'refresh' },
    },

    // Timers actifs
    _timers: {},
    
    // Onglet actuel
    _currentTab: null,

    init() {
        console.log('🗂️ TabManager initialized (Tab-Aware)');
        
        // Écouter les changements d'onglet
        document.addEventListener('tabChanged', (e) => {
            this.switchToTab(e.detail.tab);
        });

        // Écouter la visibilité de la page
        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible' && this._currentTab) {
                console.log('👁️ Page visible, refreshing current tab:', this._currentTab);
                this._fetchForTab(this._currentTab);
            } else if (document.visibilityState === 'hidden') {
                console.log('💤 Page hidden, pausing all polling');
                this._stopAllPolling();
            }
        });

        // Initialiser avec l'onglet actuel
        const initialTab = window.KusanagiDashboard?.activeTab || 'argocd';
        this.switchToTab(initialTab);
    },

    /**
     * Change vers un nouvel onglet
     */
    switchToTab(tabName) {
        if (this._currentTab === tabName) return;
        
        console.log(`🔄 TabManager: switching from ${this._currentTab} to ${tabName}`);
        
        // Arrêter le polling de l'ancien onglet
        if (this._currentTab) {
            this._stopPolling(this._currentTab);
        }

        this._currentTab = tabName;
        
        // Démarrer le polling du nouvel onglet
        this._startPolling(tabName);
        
        // Fetch immédiat
        this._fetchForTab(tabName);
    },

    /**
     * Démarre le polling pour un onglet
     */
    _startPolling(tabName) {
        const config = this.TABS[tabName];
        if (!config || !config.interval) {
            console.log(`⏸️ No polling for ${tabName}`);
            return;
        }

        // Ne pas démarrer si la page est cachée
        if (document.hidden) {
            console.log(`💤 Page hidden, not starting polling for ${tabName}`);
            return;
        }

        console.log(`⏱️ Starting polling for ${tabName} every ${config.interval}ms`);

        this._timers[tabName] = setInterval(() => {
            // Vérifier qu'on est toujours sur cet onglet et que la page est visible
            if (this._currentTab === tabName && !document.hidden) {
                this._fetchForTab(tabName);
            }
        }, config.interval);
    },

    /**
     * Arrête le polling pour un onglet
     */
    _stopPolling(tabName) {
        if (this._timers[tabName]) {
            clearInterval(this._timers[tabName]);
            delete this._timers[tabName];
            console.log(`🛑 Stopped polling for ${tabName}`);
        }
    },

    /**
     * Arrête tout le polling
     */
    _stopAllPolling() {
        Object.keys(this._timers).forEach(tab => this._stopPolling(tab));
    },

    /**
     * Fetch les données pour un onglet spécifique
     */
    _fetchForTab(tabName) {
        const config = this.TABS[tabName];
        if (!config || !config.module || !config.fetch) {
            console.log(`📭 No fetch configured for ${tabName}`);
            return;
        }

        const module = window[config.module];
        if (!module) {
            console.warn(`⚠️ Module ${config.module} not found for tab ${tabName}`);
            return;
        }

        const fetchFn = module[config.fetch];
        if (typeof fetchFn !== 'function') {
            console.warn(`⚠️ Function ${config.fetch} not found in ${config.module}`);
            return;
        }

        console.log(`📡 Fetching ${tabName} via ${config.module}.${config.fetch}()`);
        
        try {
            const result = fetchFn.call(module);
            if (result && typeof result.then === 'function') {
                result.catch(err => {
                    console.warn(`❌ Error fetching ${tabName}:`, err.message);
                });
            }
        } catch (err) {
            console.warn(`❌ Error fetching ${tabName}:`, err.message);
        }
    },

    /**
     * Force un refresh manuel de l'onglet actuel
     */
    refreshCurrentTab() {
        if (this._currentTab) {
            this._fetchForTab(this._currentTab);
        }
    },

    /**
     * Obtient la liste des onglets actifs
     */
    getActiveTabs() {
        return Object.keys(this._timers);
    }
};

// Initialiser au chargement
document.addEventListener('DOMContentLoaded', () => {
    TabManager.init();
});

window.TabManager = TabManager;
