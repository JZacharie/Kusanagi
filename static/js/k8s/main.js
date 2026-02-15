/**
 * K8sManager Facade
 * Orchestrates calls to submodule and provides backward compatibility
 */
const K8sManager = {
    init() {
        console.log('🚀 K8s Facade Initialized');
        // Check which modules are available
        console.log('📦 Available modules:', {
            K8sPods: !!window.K8sPods,
            K8sNodes: !!window.K8sNodes,
            K8sArgo: !!window.K8sArgo,
            K8sStorage: !!window.K8sStorage,
            K8sServices: !!window.K8sServices
        });
        // Initialize submodules
        if (window.K8sPods) K8sPods.init();
        if (window.K8sNodes) K8sNodes.init();
        if (window.K8sArgo) K8sArgo.init();
        if (window.K8sStorage) K8sStorage.init();
        if (window.K8sServices) K8sServices.init();

        this.setupEventListeners();

        // Initial fetch
        this.fetchAll();

        // Setup global polling
        // No global polling - fetchAll is startup only
        console.log('✅ K8sManager initialized (Startup fetch only)');
    },

    // Rate limiting: delay between API calls (ms)
    _apiDelay: 2000,
    _lastApiCall: 0,

    async _rateLimitedCall(fn, ...args) {
        const now = Date.now();
        const timeSinceLastCall = now - this._lastApiCall;
        if (timeSinceLastCall < this._apiDelay) {
            await new Promise(r => setTimeout(r, this._apiDelay - timeSinceLastCall));
        }
        this._lastApiCall = Date.now();
        return fn(...args);
    },

    async fetchAll() {
        if (document.hidden) {
            console.log('💤 Tab hidden, skipping K8s fetchAll');
            return;
        }

        // Stagger API calls to avoid 429 Too Many Requests
        const calls = [
            { name: 'ArgoCD', fn: () => window.K8sArgo?.fetchArgoStatus() },
            { name: 'Nodes', fn: () => window.K8sNodes?.fetchNodesStatus() },
            { name: 'Pods', fn: () => window.K8sPods?.fetchPodsStatus() },
            { name: 'Overview', fn: () => this.fetchClusterOverview() },
            { name: 'Storage', fn: () => window.K8sStorage?.fetchStorageStatus() },
            { name: 'Backups', fn: () => window.K8sStorage?.fetchBackupsStatus() },
            { name: 'Services', fn: () => window.K8sServices?.fetchServices() },
            { name: 'Ingress', fn: () => window.K8sServices?.fetchIngress() },
        ];

        for (const { name, fn } of calls) {
            if (fn) {
                try {
                    await this._rateLimitedCall(fn);
                } catch (e) {
                    console.warn(`⚠️ ${name} fetch failed:`, e.message);
                }
            }
        }
    },

    // Cluster overview - basic cluster stats only
    // Note: PVC/Services details are handled by their respective modules
    async fetchClusterOverview() {
        try {
            const data = await api.get('/api/k8s/cluster');

            // Only update elements that this endpoint actually provides
            const stats = {
                'ns-count': data.namespace_count || 0,
                'services-count': data.services || 0,
                'pods-total': data.pods || 0,
                'node-total': data.nodes || 0
            };
            for (const [id, value] of Object.entries(stats)) {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }
        } catch (error) {
            console.error('Failed to fetch cluster overview:', error);
        }
    },

    setupEventListeners() {
        // Refresh on page focus
        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
                const now = Date.now();

                if (activeTab === 'services' && window.K8sServices) {
                    K8sServices.fetchServices();
                } else if (activeTab === 'ingress' && window.K8sServices) {
                    K8sServices.fetchIngress();
                } else if (activeTab === 'argocd' && window.K8sArgo) {
                    K8sArgo.fetchArgoStatus();
                }
            }
        });
    },

    // === FACADE METHODS FOR HTML COMPATIBILITY ===
    // These methods are called by onclick handlers in the HTML
    // We delegate them to the appropriate submodules

    // Storage / Backups
    sortStorage(field) { K8sStorage.sortStorage(field); },
    changeStoragePage(delta) { K8sStorage.changeStoragePage(delta); },
    fetchStorageStatus() { K8sStorage.fetchStorageStatus(); },
    fetchBackupsStatus() { K8sStorage.fetchBackupsStatus(); },
    triggerCronJob(ns, name) { K8sStorage.triggerCronJob(ns, name); },

    // Pods
    fetchPodsStatus() { K8sPods.fetchPodsStatus(); },
    restartAllErrorPods() { K8sPods.restartAllErrorPods(); },
    viewPodLogs(ns, name) { K8sPods.viewPodLogs(ns, name); },
    forceDeletePod(ns, name) { K8sPods.forceDeletePod(ns, name); },
    closeLogsModal() { K8sPods.closeLogsModal(); },

    // Nodes
    fetchNodesStatus() { K8sNodes.fetchNodesStatus(); },
    runNodesDiagnostic() { K8sNodes.runNodesDiagnostic(); },

    // ArgoCD
    syncApp(event, appName) { K8sArgo.syncApp(event, appName); },
    fetchArgoStatus() { K8sArgo.fetchArgoStatus(); },

    // Network
    fetchServices() { K8sServices.fetchServices(); },
    fetchIngress() { K8sServices.fetchIngress(); }
};

window.K8sManager = K8sManager;
