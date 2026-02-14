/**
 * K8sManager Facade
 * Orchestrates calls to submodule and provides backward compatibility
 */
const K8sManager = {
    init() {
        console.log('🚀 K8s Facade Initialized');
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
        setInterval(() => this.fetchAll(), 30000);
    },

    fetchAll() {
        if (window.K8sArgo) K8sArgo.fetchArgoStatus();
        if (window.K8sNodes) K8sNodes.fetchNodesStatus();
        if (window.K8sPods) K8sPods.fetchPodsStatus();
        this.fetchClusterOverview(); // Kept internal or move to separate util
        if (window.K8sStorage) {
            K8sStorage.fetchStorageStatus();
            K8sStorage.fetchBackupsStatus();
        }
        if (window.K8sServices) {
            K8sServices.fetchServices();
            K8sServices.fetchIngress();
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
