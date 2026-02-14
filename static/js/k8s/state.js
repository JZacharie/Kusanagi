const K8sState = {
    // Services & Ingress
    SERVICES_INGRESS_TTL: 180000, // 3 minutes
    lastServicesFetch: 0,
    lastIngressFetch: 0,

    // Storage
    storageData: [],
    storagePage: 1,
    storagePerPage: 10,
    storageSortField: 'usage_percent',
    storageSortDir: 'desc',

    // Events
    currentEventFilter: 'all',
    currentEventPage: 1,
    eventPerPage: 20,

    // Cache Helpers
    loadFromCache(key, ttl = 300000) {
        try {
            const cached = localStorage.getItem(key);
            if (!cached) return null;

            const data = JSON.parse(cached);
            const age = Date.now() - data.timestamp;

            if (age < ttl) {
                console.log(`📋 Loading ${key} from cache (age: ${Math.round(age / 1000)}s)`);
                return data.payload;
            }
        } catch (e) {
            console.error(`Cache load error for ${key}:`, e);
        }
        return null;
    },

    saveToCache(key, payload) {
        try {
            localStorage.setItem(key, JSON.stringify({
                payload: payload,
                timestamp: Date.now()
            }));
            console.log(`💾 Saved ${key} to localStorage`);
        } catch (e) {
            console.error(`Cache save error for ${key}:`, e);
        }
    },

    // UI Helpers
    escapeHtml(text) {
        if (!text) return '';
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    },

    formatBytes(bytes) {
        if (!bytes && bytes !== 0) return null;
        if (bytes === 0) return '0 B';
        const k = 1024; const sizes = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    },

    formatCpu(cores) {
        if (cores === undefined || cores === null) return '-';
        if (cores < 0.001) return '0';
        if (cores < 1) return Math.round(cores * 1000) + 'm';
        return cores.toFixed(2);
    }
};

window.K8sState = K8sState;
