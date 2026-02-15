/**
 * K8sManager Facade
 * Orchestrates calls to submodule - TAB-AWARE architecture
 * Each module only fetches when its tab is active
 */

const K8sManager = {
    // Poll intervals per tab (ms)
    POLL_INTERVALS: {
        argocd: 30000,
        nodes: 60000,
        pods: 30000,
        storage: 60000,
        backups: 60000,
        services: 300000,  // 5 min - services don't change often
        ingress: 300000,   // 5 min
    },

    // Active poll timers
    _pollTimers: {},

    init() {
        console.log('🚀 K8s Facade Initialized (Tab-Aware)');
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

        // Setup tab change detection
        this.setupTabListeners();

        // Note: Initial fetch is handled by TabManager to avoid duplicates
        // const currentTab = window.KusanagiDashboard?.activeTab || 'argocd';
        // this._fetchForTab(currentTab);

        console.log('✅ K8sManager initialized (Tab-Aware)');
    },

    // Fetch data only for a specific tab
    _fetchForTab(tabName) {
        console.log(`🔄 Fetching data for tab: ${tabName}`);

        switch (tabName) {
            case 'argocd':
                if (window.K8sArgo) K8sArgo.fetchArgoStatus();
                break;
            case 'nodes':
                if (window.K8sNodes) K8sNodes.fetchNodesStatus();
                break;
            case 'pods':
                if (window.K8sPods) K8sPods.fetchPodsStatus();
                break;
            case 'storage':
                if (window.K8sStorage) K8sStorage.fetchStorageStatus();
                break;
            case 'backups':
                if (window.K8sStorage) K8sStorage.fetchBackupsStatus();
                break;
            case 'services':
                if (window.K8sServices) K8sServices.fetchServices();
                break;
            case 'ingress':
                if (window.K8sServices) K8sServices.fetchIngress();
                break;
            default:
                // For dashboard/overview, fetch minimal data
                this.fetchClusterOverview();
        }
    },

    // NOTE: Polling is now handled by TabManager only
    // These methods are kept for compatibility but do nothing
    _startPolling(tabName) {
        // Disabled - use TabManager instead
        console.log(`⏸️ K8sManager polling disabled for ${tabName} (using TabManager)`);
    },

    _stopPolling(tabName) {
        // Disabled - use TabManager instead
    },

    onTabChange(newTab) {
        // Disabled - TabManager handles this
        console.log(`🔄 K8sManager: Tab change to ${newTab} ignored (using TabManager)`);
    },

    setupTabListeners() {
        // Disabled - TabManager handles all tab events
        console.log('🔄 K8sManager tab listeners disabled (using TabManager)');
    },

    // Cluster overview - basic cluster stats only
    async fetchClusterOverview() {
        try {
            const data = await api.get('/api/k8s/cluster');
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

    // Manual refresh for current tab
    refreshCurrentTab() {
        const activeTab = window.KusanagiDashboard?.activeTab || 'argocd';
        this._fetchForTab(activeTab);
    },

    // === FACADE METHODS FOR HTML COMPATIBILITY ===
    sortStorage(field) { K8sStorage?.sortStorage(field); },
    changeStoragePage(delta) { K8sStorage?.changeStoragePage(delta); },
    fetchStorageStatus() { K8sStorage?.fetchStorageStatus(); },
    fetchBackupsStatus() { K8sStorage?.fetchBackupsStatus(); },
    triggerCronJob(ns, name) { K8sStorage?.triggerCronJob(ns, name); },
    fetchPodsStatus() { K8sPods?.fetchPodsStatus(); },
    restartAllErrorPods() { K8sPods?.restartAllErrorPods(); },
    viewPodLogs(ns, name) { K8sPods?.viewPodLogs(ns, name); },
    forceDeletePod(ns, name) { K8sPods?.forceDeletePod(ns, name); },
    closeLogsModal() { K8sPods?.closeLogsModal(); },
    fetchNodesStatus() { K8sNodes?.fetchNodesStatus(); },
    runNodesDiagnostic() { K8sNodes?.runNodesDiagnostic(); },
    syncApp(event, appName) { K8sArgo?.syncApp(event, appName); },
    fetchArgoStatus() { K8sArgo?.fetchArgoStatus(); },
    fetchServices() { K8sServices?.fetchServices(); },
    fetchIngress() { K8sServices?.fetchIngress(); }
};

window.K8sManager = K8sManager;
