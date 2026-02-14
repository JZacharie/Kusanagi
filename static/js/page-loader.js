/**
 * Page Loader - Dynamic partial loading system
 * Loads page sections on demand to reduce initial load time
 */

const PageLoader = {
    // Cache for loaded partials
    cache: new Map(),

    // Base path for partials
    partialsBase: '/static/partials/',

    // Mapping of tab names to partial files
    partials: {
        'nodes': 'nodes.html',
        'storage': 'storage.html',
        'services': 'services.html',
        'ingress': 'ingress.html',
        'events': 'events.html',
        'pods': 'pods.html',
        'chat': 'chat.html',
        'news': 'news.html',
        'backups': 'backups.html',
        'network': 'network.html',
        'metrics': 'metrics.html',
        'system': 'system.html',
        'alerts': 'alerts.html',
        'security': 'security.html',
        'mqtt': 'mqtt.html',
        'proxmox': 'proxmox.html',
        'homeassistant': 'homeassistant.html',
        'weather': 'weather.html',
        'setup': 'setup.html',
        'monitors': 'monitors.html',
        'docs': 'docs.html'
    },

    /**
     * Load a partial for a given tab
     */
    async loadPartial(tabName) {
        const section = document.querySelector(`section[data-tab="${tabName}"]`);
        if (!section) return false;

        // Check if already loaded (has content other than empty or loading state)
        if (section.dataset.loaded === 'true') return true;

        const partialFile = this.partials[tabName];
        if (!partialFile) return false; // Static content, no partial needed

        try {
            // Check cache first
            if (this.cache.has(tabName)) {
                section.innerHTML = this.cache.get(tabName);
                section.dataset.loaded = 'true';
                this.initScripts(tabName);
                return true;
            }

            // Show loading state
            section.innerHTML = '<div class="loading">Loading...</div>';

            // Fetch partial
            const response = await fetch(`${this.partialsBase}${partialFile}`);
            if (!response.ok) {
                console.warn(`Failed to load partial for ${tabName}:`, response.status);
                section.innerHTML = `
                    <div style="padding: 2rem; text-align: center;">
                        <p style="color: #ff4444;">⚠️ Failed to load section</p>
                        <button onclick="PageLoader.loadPartial('${tabName}')" class="cyber-btn">Retry</button>
                    </div>
                `;
                return false;
            }

            const html = await response.text();

            // Cache and inject
            this.cache.set(tabName, html);
            section.innerHTML = html;
            section.dataset.loaded = 'true';

            // Initialize any scripts for this section
            this.initScripts(tabName);

            return true;
        } catch (error) {
            console.error(`Error loading partial for ${tabName}:`, error);
            section.innerHTML = `
                <div style="padding: 2rem; text-align: center;">
                    <p style="color: #ff4444;">⚠️ Failed to load section</p>
                    <button onclick="PageLoader.loadPartial('${tabName}')" class="cyber-btn">Retry</button>
                </div>
            `;
            return false;
        }
    },

    /**
     * Initialize scripts for a loaded section
     */
    initScripts(tabName) {
        // Trigger initialization based on tab
        switch (tabName) {
            case 'proxmox':
                if (window.ProxmoxDashboard) ProxmoxDashboard.init();
                break;
            case 'homeassistant':
                if (window.HomeAssistantDashboard) HomeAssistantDashboard.init();
                break;
            case 'weather':
                if (window.WeatherDashboard) WeatherDashboard.init();
                break;
            case 'security':
                if (window.SecurityDashboard) SecurityDashboard.init();
                break;
            case 'alerts':
                if (window.AlertsManager) AlertsManager.init();
                break;
            case 'system':
                if (window.KusanagiSystem) KusanagiSystem.activate();
                break;
            case 'nodes':
                if (window.K8sManager && K8sManager.fetchNodesStatus) {
                    K8sManager.fetchNodesStatus();
                }
                break;
            case 'events':
                if (window.K8sManager && K8sManager.fetchEvents) {
                    K8sManager.fetchEvents('all', 1);
                }
                break;
            case 'storage':
                if (window.K8sManager && K8sManager.fetchStorageStatus) {
                    K8sManager.fetchStorageStatus();
                }
                break;
            case 'pods':
                if (window.K8sManager && K8sManager.fetchPodsStatus) {
                    K8sManager.fetchPodsStatus();
                }
                break;
            case 'backups':
                if (window.K8sManager && K8sManager.fetchBackupsStatus) {
                    K8sManager.fetchBackupsStatus();
                }
                break;
            case 'services':
                if (window.K8sManager && K8sManager.fetchServices) {
                    K8sManager.fetchServices();
                }
                break;
            case 'ingress':
                if (window.K8sManager && K8sManager.fetchIngress) {
                    K8sManager.fetchIngress();
                }
                break;
            case 'monitors':
                if (window.MonitorsManager) MonitorsManager.init();
                break;
        }
    },

    /**
     * Preload critical partials
     */
    preloadCritical() {
        // Preload frequently accessed sections
        const critical = ['alerts', 'events', 'nodes'];
        critical.forEach(tab => {
            if (this.partials[tab] && !this.cache.has(tab)) {
                fetch(`${this.partialsBase}${this.partials[tab]}`)
                    .then(r => r.text())
                    .then(html => this.cache.set(tab, html))
                    .catch(() => { });
            }
        });
    },

    /**
     * Clear cache for a specific tab or all tabs
     */
    clearCache(tabName = null) {
        if (tabName) {
            this.cache.delete(tabName);
            const section = document.querySelector(`section[data-tab="${tabName}"]`);
            if (section) {
                section.dataset.loaded = 'false';
            }
        } else {
            this.cache.clear();
            document.querySelectorAll('section[data-tab]').forEach(section => {
                section.dataset.loaded = 'false';
            });
        }
    }
};

// Expose globally
window.PageLoader = PageLoader;

// Auto-preload after initial load
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => PageLoader.preloadCritical());
} else {
    PageLoader.preloadCritical();
}
