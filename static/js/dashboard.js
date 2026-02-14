/**
 * Kusanagi Dashboard Manager
 * Handles customizable widgets, layout persistence, and export functionality
 */

/**
 * Kusanagi Localization Manager
 * Handles multi-language support (EN/FR)
 */
const LocaleManager = {
    currentLocale: 'fr',
    translations: {},

    async init() {
        const savedLocale = localStorage.getItem('kusanagi_locale');
        if (savedLocale) {
            this.currentLocale = savedLocale;
        }

        const languageSelect = document.getElementById('language-select');
        if (languageSelect) {
            languageSelect.value = this.currentLocale;
            languageSelect.addEventListener('change', (e) => {
                this.setLocale(e.target.value);
            });
        }

        await this.loadTranslations();
        this.applyTranslations();
        console.log(`✅ Locale Manager initialized (${this.currentLocale})`);
    },

    async loadTranslations() {
        try {
            const response = await fetch(`/static/locales/${this.currentLocale}.json`);
            if (!response.ok) throw new Error(`Failed to load locale: ${this.currentLocale}`);
            this.translations = await response.json();
        } catch (e) {
            console.error('Localization error:', e);
        }
    },

    async setLocale(locale) {
        if (locale === this.currentLocale) return;
        this.currentLocale = locale;
        localStorage.setItem('kusanagi_locale', locale);
        await this.loadTranslations();
        this.applyTranslations();

        // Notify other managers if needed
        if (window.ChatManager) window.ChatManager.updateSystemPrompt();
        if (window.NewsManager) window.NewsManager.renderNews();
    },

    applyTranslations() {
        const elements = document.querySelectorAll('[data-i18n]');
        elements.forEach(el => {
            const key = el.getAttribute('data-i18n');
            if (this.translations[key]) {
                el.textContent = this.translations[key];
            }
        });

        // Update placeholders
        document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
            const key = el.getAttribute('data-i18n-placeholder');
            if (this.translations[key]) {
                el.placeholder = this.translations[key];
            }
        });
    },

    t(key) {
        return this.translations[key] || key;
    }
};

/**
 * Kusanagi System Status Manager
 * Handles uptime, CPU/RAM metrics and auto-refresh on restart
 */
const SystemStatusManager = {
    startTime: null,
    refreshInterval: null,

    init() {
        this.fetchStatus();
        this.refreshInterval = setInterval(() => this.fetchStatus(), 5000);
        console.log('✅ System Status Manager initialized');
    },

    async fetchStatus() {
        try {
            const response = await fetch('/api/system/status');
            if (!response.ok) throw new Error('Status fetch failed');
            const data = await response.json();

            // Check if backend restarted
            if (this.startTime && this.startTime !== data.start_time) {
                console.log('🔄 Backend restart detected! Refreshing data...');
                // Automatically refresh all data if backend restarted
                if (window.refreshAllKusanagiData) refreshAllKusanagiData(false);
            }
            this.startTime = data.start_time;

            this.updateUI(data);
        } catch (e) {
            console.error('System status error:', e);
        }
    },

    updateUI(data) {
        const uptimeEl = document.getElementById('kusanagi-uptime');
        const cpuEl = document.getElementById('kusanagi-cpu');
        const ramEl = document.getElementById('kusanagi-ram');
        const versionEl = document.getElementById('kusanagi-version');
        const dbEl = document.getElementById('kusanagi-db-status');
        const indicator = document.getElementById('kusanagi-refresh-indicator');

        // Use backend provided uptime_secs or fallback to uptime string
        if (uptimeEl) {
            uptimeEl.textContent = this.formatUptime(data.uptime_secs ?? 0);
        }

        // Backend returns cpu_usage (percent) and memory_usage_mb (MB)
        if (cpuEl) cpuEl.textContent = `${(data.cpu_usage ?? data.cpu_usage_percent ?? 0).toFixed(1)}%`;
        if (ramEl) ramEl.textContent = `${(data.memory_usage_mb ?? (data.memory_usage_bytes / 1048576) ?? 0).toFixed(0)} MB`;
        if (versionEl) versionEl.textContent = data.version ?? '0.3.0';
        if (dbEl) dbEl.textContent = 'SQLite';

        // Visual flash on update
        if (indicator) {
            indicator.style.opacity = '1';
            setTimeout(() => { if (indicator) indicator.style.opacity = '0.5'; }, 500);
        }
    },

    formatUptime(seconds) {
        if (!seconds && seconds !== 0) return '--:--:--';
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = seconds % 60;
        return [h, m, s].map(v => v.toString().padStart(2, '0')).join(':');
    }
};

const DashboardManager = {
    // Available widgets configuration
    widgets: {
        argocd: { name: 'ArgoCD', icon: '🚀', enabled: true, order: 0 },
        nodes: { name: 'Nodes', icon: '🖥️', enabled: true, order: 1 },
        storage: { name: 'Storage', icon: '💾', enabled: true, order: 2 },
        storage: { name: 'Storage', icon: '💾', enabled: true, order: 2 },
        monitors: { name: 'Monitors', icon: '🛡️', enabled: true, order: 3 },
        pods: { name: 'Pods', icon: '📦', enabled: true, order: 4 },
        network: { name: 'Network', icon: '🌐', enabled: true, order: 5 },
        metrics: { name: 'Metrics', icon: '📊', enabled: true, order: 6 },
        // alerts/events deprecated

        chat: { name: 'Chat', icon: '💬', enabled: true, order: 8 },
        proxmox: { name: 'Proxmox', icon: '🖥️', enabled: true, order: 9 },
        homeassistant: { name: 'Home Assistant', icon: '🏠', enabled: true, order: 10 },
        weather: { name: 'Weather', icon: '🌤️', enabled: true, order: 11 },
        calendar: { name: 'Calendar', icon: '📅', enabled: true, order: 12 },
        mqtt: { name: 'MQTT', icon: '📡', enabled: true, order: 13 }
    },

    storageKey: 'kusanagi_dashboard_layout',

    /**
     * Initialize dashboard manager
     */
    init() {
        SystemStatusManager.init();
        if (window.K8sManager) {
            console.log('🚀 Initializing K8sManager...');
            K8sManager.init();
        }
        this.loadLayout();
        this.setupEventListeners();
        console.log('✅ Dashboard Manager initialized');
    },

    /**
     * Load saved layout from localStorage
     */
    loadLayout() {
        try {
            const saved = localStorage.getItem(this.storageKey);
            if (saved) {
                const layout = JSON.parse(saved);
                Object.keys(layout).forEach(key => {
                    if (this.widgets[key]) {
                        this.widgets[key].enabled = layout[key].enabled;
                        this.widgets[key].order = layout[key].order;
                    }
                });
            }
        } catch (e) {
            console.warn('Failed to load dashboard layout:', e);
        }
    },

    /**
     * Save current layout to localStorage
     */
    saveLayout() {
        try {
            const layout = {};
            Object.keys(this.widgets).forEach(key => {
                layout[key] = {
                    enabled: this.widgets[key].enabled,
                    order: this.widgets[key].order
                };
            });
            localStorage.setItem(this.storageKey, JSON.stringify(layout));
        } catch (e) {
            console.warn('Failed to save dashboard layout:', e);
        }
    },

    /**
     * Toggle widget visibility
     */
    toggleWidget(widgetName) {
        if (this.widgets[widgetName]) {
            this.widgets[widgetName].enabled = !this.widgets[widgetName].enabled;
            this.saveLayout();
            this.applyLayout();
        }
    },

    /**
     * Apply current layout to DOM
     */
    applyLayout() {
        Object.keys(this.widgets).forEach(key => {
            const tabBtn = document.querySelector(`[data-tab="${key}"]`);
            const section = document.getElementById(`${key}-section`);

            if (tabBtn) {
                tabBtn.style.display = this.widgets[key].enabled ? '' : 'none';
            }
            if (section && !this.widgets[key].enabled) {
                section.style.display = 'none';
            }
        });
    },

    /**
     * Setup event listeners
     */
    setupEventListeners() {
        // Export button handlers
        document.addEventListener('click', (e) => {
            if (e.target.matches('.export-btn') || e.target.closest('.export-btn')) {
                const format = e.target.dataset.format || e.target.closest('.export-btn').dataset.format;
                if (format) {
                    this.exportReport(format);
                }
            }
        });
    },

    /**
     * Export cluster report
     */
    async exportReport(format = 'json') {
        try {
            showNotification('Generating report...', 'info');

            const response = await fetch(`/api/export/report?format=${format}`);
            if (!response.ok) {
                throw new Error(`Export failed: ${response.statusText}`);
            }

            const blob = await response.blob();
            const extension = format === 'markdown' ? 'md' : format;
            const filename = `kusanagi-report-${new Date().toISOString().slice(0, 10)}.${extension}`;

            // Create download link
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = filename;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            a.remove();

            showNotification(`Report exported as ${format.toUpperCase()}`, 'success');
        } catch (error) {
            console.error('Export error:', error);
            showNotification(`Export failed: ${error.message}`, 'error');
        }
    },

    /**
     * Get widget settings for display
     */
    getWidgetSettings() {
        return Object.entries(this.widgets).map(([key, widget]) => ({
            id: key,
            ...widget
        }));
    }
};

/**
 * Metrics display manager
 */
const MetricsManager = {
    refreshInterval: null,

    /**
     * Initialize metrics display
     */
    init() {
        this.loadMetrics();
        // Refresh every 30 seconds
        this.refreshInterval = setInterval(() => this.loadMetrics(), 30000);
    },

    /**
     * Load metrics from localStorage cache
     */
    loadFromCache() {
        try {
            const cached = localStorage.getItem('kusanagi_metrics_cache');
            if (!cached) return null;

            const data = JSON.parse(cached);
            const age = Date.now() - data.timestamp;

            // Use cache if less than 30 seconds old (same as refresh interval)
            if (age < 30000) {
                console.log('📊 Loading metrics from localStorage cache (age: ' + Math.round(age / 1000) + 's)');
                return data.metrics;
            }
        } catch (e) {
            console.error('Metrics cache load error:', e);
        }
        return null;
    },

    /**
     * Save metrics to localStorage cache
     */
    saveToCache(metrics) {
        try {
            localStorage.setItem('kusanagi_metrics_cache', JSON.stringify({
                metrics: metrics,
                timestamp: Date.now()
            }));
            console.log('💾 Metrics saved to localStorage cache');
        } catch (e) {
            console.error('Metrics cache save error:', e);
        }
    },

    /**
     * Load Prometheus metrics
     */
    async loadMetrics() {
        const container = document.getElementById('metrics-content');

        // Try to load from cache first for instant display
        const cached = this.loadFromCache();
        if (cached) {
            this.renderMetrics(cached);
        } else if (container) {
            // Show loading state only if no cached data
            container.innerHTML = `
                <div class="loading-state" style="text-align: center; padding: 3rem;">
                    <div style="font-size: 3rem; animation: pulse 1.5s ease-in-out infinite;">📊</div>
                    <p style="margin-top: 1rem; opacity: 0.7; font-family: 'Orbitron', sans-serif;">LOADING METRICS...</p>
                    <p style="font-size: 0.8rem; opacity: 0.5; font-family: 'JetBrains Mono', monospace;">CONNECTING TO PROMETHEUS</p>
                </div>
            `;
        }

        try {
            const response = await fetch('/api/metrics');
            if (!response.ok) {
                let errorMsg = `Server returned ${response.status}`;
                try {
                    const errorData = await response.json();
                    if (errorData.error) errorMsg = errorData.error;
                } catch (e) {
                    // Not JSON, use status
                }
                throw new Error(errorMsg);
            }

            const metrics = await response.json();

            // Save to cache
            this.saveToCache(metrics);

            this.renderMetrics(metrics);
        } catch (error) {
            console.error('Metrics error:', error);
            this.renderMetricsError(error.message);
        }
    },

    /**
     * Fetch 24h range data for a query
     */
    async fetchRangeData(query, hours = 24) {
        try {
            const end = Math.floor(Date.now() / 1000);
            const start = end - (hours * 3600);
            const step = '15m'; // 15 minute resolution for 24h

            const response = await fetch(`/api/prometheus/range?query=${encodeURIComponent(query)}&start=${start}&end=${end}&step=${step}`);
            if (!response.ok) return null;
            const data = await response.json();
            return data.data?.result?.[0]?.values || null;
        } catch (e) {
            console.error('Range query failed:', e);
            return null;
        }
    },

    /**
     * Render SVG Sparkline for metrics
     */
    renderSparkline(values, width = 300, height = 50, color = 'var(--neon-green)') {
        if (!values || values.length < 2) return '';

        const dataPoints = values.map(v => parseFloat(v[1]));
        const min = Math.min(...dataPoints);
        const max = Math.max(...dataPoints);
        const range = max - min || 1;

        const points = dataPoints.map((v, i) => {
            const x = (i / (dataPoints.length - 1)) * width;
            const y = height - ((v - min) / range) * height;
            return `${x},${y}`;
        }).join(' ');

        return `
            <svg width="${width}" height="${height}" style="overflow: visible; margin-top: 10px;">
                <polyline points="${points}" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
                <path d="M 0 ${height} L ${points} L ${width} ${height} Z" fill="${color}" fill-opacity="0.1" />
            </svg>
        `;
    },

    /**
     * Render metrics to UI
     */
    renderMetrics(metrics) {
        const container = document.getElementById('metrics-content');
        if (!container) return;

        const sectionHeaderStyle = "margin: 2rem 0 1rem; color: var(--neon-cyan); font-family: 'Orbitron', sans-serif; font-size: 1.1rem; border-bottom: 1px solid rgba(0, 255, 249, 0.2); padding-bottom: 0.5rem;";

        container.innerHTML = `
            <h3 style="${sectionHeaderStyle}">🖥️ Cluster Infrastructure</h3>
            <div class="metrics-grid">
                <div class="metric-card">
                    <div class="metric-icon">🔥</div>
                    <div class="metric-value">${metrics.cpu_usage_percent?.toFixed(1) || 0}%</div>
                    <div class="metric-label">CPU Usage</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill cpu" style="width: ${metrics.cpu_usage_percent || 0}%"></div>
                    </div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">💾</div>
                    <div class="metric-value">${metrics.memory_usage_percent?.toFixed(1) || 0}%</div>
                    <div class="metric-label">Memory Usage</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill memory" style="width: ${metrics.memory_usage_percent || 0}%"></div>
                    </div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">📦</div>
                    <div class="metric-value">${metrics.pod_count || 0}</div>
                    <div class="metric-label">Pods</div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">🖥️</div>
                    <div class="metric-value">${metrics.node_count || 0}</div>
                    <div class="metric-label">Nodes</div>
                </div>
                <div class="metric-card ${metrics.alerts_firing > 0 ? 'alert-critical' : ''}">
                    <div class="metric-icon">🔔</div>
                    <div class="metric-value">${metrics.alerts_firing || 0}</div>
                    <div class="metric-label">Firing Alerts</div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">📊</div>
                    <div class="metric-value">${metrics.container_count || 0}</div>
                    <div class="metric-label">Containers</div>
                </div>
            </div>

            <h3 style="${sectionHeaderStyle}">🏎️ GPU Hardware Telemetry</h3>
            <div class="metrics-grid">
                <div class="metric-card">
                    <div class="metric-icon">🏎️</div>
                    <div class="metric-value">${metrics.gpu_utilization?.toFixed(1) || 0}%</div>
                    <div class="metric-label">GPU Utilization</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${metrics.gpu_utilization || 0}%; background: var(--neon-green);"></div>
                    </div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">🌡️</div>
                    <div class="metric-value">${metrics.gpu_temperature?.toFixed(1) || 0}°C</div>
                    <div class="metric-label">GPU Temperature</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${Math.min((metrics.gpu_temperature || 0) / 100 * 100, 100)}%; background: ${metrics.gpu_temperature > 80 ? 'var(--neon-magenta)' : 'var(--neon-cyan)'};"></div>
                    </div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">⚡</div>
                    <div class="metric-value">${metrics.gpu_power_usage?.toFixed(1) || 0}W</div>
                    <div class="metric-label">GPU Power usage</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${Math.min((metrics.gpu_power_usage || 0) / 350 * 100, 100)}%; background: var(--neon-yellow);"></div>
                    </div>
                </div>
            </div>

            <h3 style="${sectionHeaderStyle}">☀️ Enphase Energy Monitoring</h3>
            <div class="metrics-grid">
                <div class="metric-card" style="grid-column: span 2; display: flex; flex-direction: column; align-items: stretch; height: auto;">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px;">
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <div class="metric-icon">☀️</div>
                            <div>
                                <div class="metric-value" id="solar-current-value">${metrics.energy_solar_production?.toFixed(1) || 0}W</div>
                                <div class="metric-label">Enphase Power Production (Live)</div>
                            </div>
                        </div>
                        <div style="text-align: right;">
                            <div style="font-size: 0.7rem; opacity: 0.5; font-family: 'JetBrains Mono';">RANGE: 24H</div>
                            <div style="font-size: 0.8rem; color: var(--neon-green); font-family: 'Orbitron'; font-weight: bold;">LIVE_FEED</div>
                        </div>
                    </div>
                    <div id="solar-24h-graph" style="height: 80px; width: 100%; background: rgba(0,0,0,0.2); border-radius: 8px; border: 1px solid rgba(0,255,249,0.1); display: flex; align-items: center; justify-content: center; overflow: hidden;">
                        <div class="loading-mini" style="font-size: 0.7rem; opacity: 0.5;">FRACTAL_SCAN...</div>
                    </div>
                    <div class="metric-bar" style="margin-top: 15px;">
                        <div class="metric-bar-fill" style="width: ${Math.min((metrics.energy_solar_production || 0) / 3000 * 100, 100)}%; background: var(--neon-green);"></div>
                    </div>
                </div>
                <!-- 🏠 House Consumption (remains) -->
                <div class="metric-card">
                    <div class="metric-icon">🏠</div>
                    <div class="metric-value">${metrics.energy_house_consumption?.toFixed(1) || 0}W</div>
                    <div class="metric-label">House Consumption</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${Math.min((metrics.energy_house_consumption || 0) / 6000 * 100, 100)}%; background: var(--neon-blue);"></div>
                    </div>
                </div>
            </div>

            <h3 style="${sectionHeaderStyle}">🚀 VPS Infrastructure</h3>
            <div class="metrics-grid">
                <div class="metric-card">
                    <div class="metric-icon">🖥️</div>
                    <div class="metric-value">${metrics.vps_cpu_usage?.toFixed(1) || 0}%</div>
                    <div class="metric-label">System CPU</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${metrics.vps_cpu_usage || 0}%; background: var(--neon-blue);"></div>
                    </div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">💽</div>
                    <div class="metric-value">${metrics.vps_disk_usage?.toFixed(1) || 0}%</div>
                    <div class="metric-label">Disk Usage (/sda1)</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${metrics.vps_disk_usage || 0}%; background: var(--neon-yellow);"></div>
                    </div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">🌐</div>
                    <div class="metric-value">${metrics.vps_net_receive?.toFixed(2) || 0}%</div>
                    <div class="metric-label">Net Receive (eth0)</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${metrics.vps_net_receive || 0}%; background: var(--neon-magenta);"></div>
                    </div>
                </div>
            </div>

            <h3 style="${sectionHeaderStyle}">🛡️ Security Vulnerabilities (Trivy Scan)</h3>
            <div class="metrics-grid">
                <div class="metric-card alert-critical">
                    <div class="metric-icon">🔴</div>
                    <div class="metric-value">${metrics.trivy_critical_count || 0}</div>
                    <div class="metric-label">Critical</div>
                </div>
                <div class="metric-card alert-warning" style="border-color: var(--neon-orange);">
                    <div class="metric-icon">🟠</div>
                    <div class="metric-value">${metrics.trivy_high_count || 0}</div>
                    <div class="metric-label">High</div>
                </div>
                <div class="metric-card" style="border-color: var(--neon-yellow);">
                    <div class="metric-icon">🟡</div>
                    <div class="metric-value">${metrics.trivy_medium_count || 0}</div>
                    <div class="metric-label">Medium</div>
                </div>
                <div class="metric-card" style="border-color: var(--neon-blue);">
                    <div class="metric-icon">🔵</div>
                    <div class="metric-value">${metrics.trivy_low_count || 0}</div>
                    <div class="metric-label">Low</div>
                </div>
            </div>
        `;

        // Load 24h graph data asynchronously
        this.loadSolarGraph();
    },

    /**
     * Load Enphase solar graph data
     */
    async loadSolarGraph() {
        const query = 'avg(homeassistant_sensor_unit_w{entity="sensor.envoy_122304017410_current_power_production"}) or vector(0)';
        const values = await this.fetchRangeData(query, 24);
        const container = document.getElementById('solar-24h-graph');

        if (container && values) {
            container.innerHTML = this.renderSparkline(values, container.clientWidth, 80, 'var(--neon-green)');
        } else if (container) {
            container.innerHTML = '<span style="font-size: 0.7rem; opacity: 0.3;">DATA_UNAVAILABLE</span>';
        }
    },

    /**
     * Render error state
     */
    renderMetricsError(message) {
        const container = document.getElementById('metrics-content');
        if (!container) return;

        container.innerHTML = `
            <div class="error-state">
                <span class="error-icon">⚠️</span>
                <p>Failed to load metrics: ${message}</p>
                <button onclick="MetricsManager.loadMetrics()" class="retry-btn">Retry</button>
            </div>
        `;
    }
};

/**
 * Alerts display manager
 */
const AlertsManager = {
    init() {
        // Delegates to MonitorsManager if available
        if (window.MonitorsManager) {
            console.log('AlertsManager delegating to MonitorsManager');
        }
    },
    loadAlerts() {
        if (window.MonitorsManager && MonitorsManager.loadMonitors) {
            MonitorsManager.loadMonitors();
        }
    }
};

/**
 * Quotas display manager
 */
const QuotasManager = {
    /**
     * Initialize quotas display
     */
    init() {
        // this.fetchQuotas(); // Disabled
        // Refresh every 60 seconds
        // setInterval(() => this.fetchQuotas(), 60000); // Disabled
    },

    /**
     * Fetch quotas from backend
     */
    async fetchQuotas() {
        try {
            const btn = document.querySelector('.quota-footer button');
            if (btn) {
                const originalText = btn.textContent;
                btn.textContent = '⏳ Refreshing...';
                btn.disabled = true;
                setTimeout(() => {
                    btn.textContent = originalText;
                    btn.disabled = false;
                }, 1000);
            }

            const response = await fetch('/api/quotas');
            if (!response.ok) {
                throw new Error('Failed to fetch quotas');
            }

            const data = await response.json();
            this.renderQuotas(data);
        } catch (error) {
            console.error('Quotas error:', error);
            showNotification({
                title: 'Quotas Error',
                message: error.message,
                severity: 'error'
            });
        }
    },

    /**
     * Render quotas to UI
     */
    renderQuotas(data) {
        // Update Antigravity Gauge
        this.updateGauge('antigravity', data.antigravity_percentage);

        // Update NotebookLM Gauge
        this.updateGauge('notebooklm', data.notebooklm_percentage);

        // Update Storage Bar
        this.updateStorage(data);

        // Update Timestamp
        const timestampEl = document.getElementById('quota-updated-at');
        if (timestampEl) {
            timestampEl.textContent = data.last_updated;
        }
    },

    /**
     * Update a circular gauge
     */
    updateGauge(id, percentage) {
        const gauge = document.getElementById(`${id}-gauge`);
        const fill = document.getElementById(`${id}-fill`);
        const value = document.getElementById(`${id}-value`);

        if (!gauge || !fill || !value) return;

        // Cap percentage between 0 and 100
        const p = Math.max(0, Math.min(100, percentage));

        // Update value text
        value.textContent = `${p}%`;

        // Update gauge color based on value
        gauge.setAttribute('data-value', p > 80 ? 'high' : p > 50 ? 'medium' : 'low');

        // Calculate stroke-dasharray
        // Radius is 40, so circumference is 2 * PI * 40 ≈ 251.3
        // We want to show only half circle (180 degrees), so max dash is ~126
        // Wait, the SVG path is `A40,40 0 1,1 80,90` which is a large arc?
        // Let's assume the path length is roughly 251 for a full circle, but we want to fill up to `p` percent.
        // Actually, looking at the SVG path `M20,90 A40,40 0 1,1 80,90`, it starts at 20,90 and ends at 80,90 with radius 40.
        // This is a semi-circle (arc length = PI * r = 3.14 * 40 = 125.6).
        // If stroke-dasharray is used, we need to set (filled_length, gap_length).
        // Max length is ~126.
        const maxLen = 126;
        const fillLen = (p / 100) * maxLen;

        // The stroke-dasharray should be `${fillLen} ${maxLen}`
        fill.style.strokeDasharray = `${fillLen} 251`;
    },

    /**
     * Update storage progress bar
     */
    updateStorage(data) {
        const usedEl = document.getElementById('storage-used');
        const totalEl = document.getElementById('storage-total');
        const fillEl = document.getElementById('storage-fill');

        if (!usedEl || !totalEl || !fillEl) return;

        usedEl.textContent = `${data.storage_used_gb.toFixed(1)} GB`;
        totalEl.textContent = `${data.storage_total_gb.toFixed(1)} GB`;

        const percent = (data.storage_used_gb / data.storage_total_gb) * 100;
        fillEl.style.width = `${Math.min(100, percent)}%`;
    }
};

/**
 * News Feed Manager
 */
const NewsManager = {
    allNews: [],
    filteredNews: [],
    currentFilter: 'all',
    searchQuery: '',
    viewMode: 'grid', // 'grid' or 'list'

    /**
     * Initialize news feed
     */
    init() {
        this.fetchNews();
        // Auto-refresh every 10 minutes (optimized from 5 minutes)
        setInterval(() => this.fetchNews(), 600000);
    },

    /**
     * Load news from localStorage cache
     */
    loadFromCache() {
        try {
            const cached = localStorage.getItem('kusanagi_news_cache');
            if (!cached) return null;

            const data = JSON.parse(cached);
            const age = Date.now() - new Date(data.cached_at).getTime();

            // Use cache if less than 10 minutes old
            if (age < 600000) {
                console.log('📰 Loading news from localStorage cache (age: ' + Math.round(age / 1000) + 's)');
                return data;
            }
        } catch (e) {
            console.error('Cache load error:', e);
        }
        return null;
    },

    /**
     * Save news to localStorage cache
     */
    saveToCache(data) {
        try {
            localStorage.setItem('kusanagi_news_cache', JSON.stringify(data));
            console.log('💾 News saved to localStorage cache');
        } catch (e) {
            console.error('Cache save error:', e);
        }
    },

    /**
     * Fetch news from API
     */
    async fetchNews() {
        const container = document.getElementById('news-container');

        // Try to load from localStorage first for instant display
        const cached = this.loadFromCache();
        if (cached && cached.items) {
            this.allNews = cached.items;
            this.sources = cached.sources || [...new Set(this.allNews.map(item => item.source))].sort();
            this.renderFilterButtons();
            this.updateStats(cached);
            this.updateTimestamp(cached.cached_at);
            this.applyFilters();
        } else if (container && !this.allNews.length) {
            // Show loading state only if no cached data and no previous data
            container.innerHTML = `
                <div class="loading-state" style="text-align: center; padding: 3rem;">
                    <div style="font-size: 3rem; animation: pulse 1.5s ease-in-out infinite;">⏳</div>
                    <p style="margin-top: 1rem; opacity: 0.7; font-family: 'Orbitron', sans-serif;">LOADING LATEST TECH NEWS...</p>
                    <p style="font-size: 0.8rem; opacity: 0.5; font-family: 'JetBrains Mono', monospace;">FETCHING FROM 13 SOURCES</p>
                </div>
            `;
        }

        try {
            const response = await fetch('/api/news');
            if (!response.ok) {
                throw new Error('Failed to fetch news');
            }

            const data = await response.json();

            // Save to localStorage cache
            this.saveToCache(data);

            this.allNews = data.items || [];

            // Extract sources if available, otherwise derive from items
            this.sources = data.sources || [...new Set(this.allNews.map(item => item.source))].sort();

            this.renderFilterButtons();
            this.updateStats(data);
            this.updateTimestamp(data.cached_at);
            this.applyFilters();
        } catch (error) {
            console.error('News fetch error:', error);
            this.renderError(error.message);
        }
    },

    /**
     * Trigger manual refresh and translation
     */
    async manualRefresh() {
        const btn = document.getElementById('btn-news-refresh');
        if (btn) {
            btn.disabled = true;
            btn.innerHTML = '<span class="spinner"></span> REFRESHING...';
            btn.classList.add('loading');
        }

        try {
            const response = await fetch('/api/news/refresh', { method: 'POST' });
            const data = await response.json();

            if (data.status === 'success') {
                showNotification('News refresh started. Translation running in background.', 'info');
                // Fetch first results immediately
                await this.fetchNews();
            } else {
                throw new Error(data.message || 'Refresh failed');
            }
        } catch (error) {
            console.error('Manual refresh error:', error);
            showNotification(`Refresh failed: ${error.message}`, 'error');
        } finally {
            if (btn) {
                btn.disabled = false;
                btn.innerHTML = '🔄 REFRESH & TRANSLATE';
                btn.classList.remove('loading');
            }
        }
    },

    /**
     * Update news statistics dynamically
     */
    /**
     * Update news statistics dynamically
     */
    updateStats(data) {
        const statsGrid = document.getElementById('news-stats-grid');
        if (!statsGrid) return;

        // Calculate total from items if available, otherwise use provided total
        const items = data.items || [];
        const totalCount = items.length || data.total || 0;

        // "All" filter box
        let html = `
            <div class="stat-box info ${this.currentFilter === 'all' ? 'active-filter' : ''}" 
                 onclick="filterNews('all')" 
                 style="cursor: pointer; transition: all 0.2s; border: 1px solid var(--neon-cyan);">
                <div class="stat-value" id="news-total">${totalCount}</div>
                <div class="stat-label">All News</div>
            </div>
        `;

        // Count per source
        const counts = {};
        items.forEach(item => {
            counts[item.source] = (counts[item.source] || 0) + 1;
        });

        // Get all configured sources to ensure we show 0 counts for important sources
        const allConfigs = this.getAllSourceConfigs();

        // Sort sources: high count first, then alphabetical
        const sortedSources = Object.keys(allConfigs).sort((a, b) => {
            const countA = counts[a] || 0;
            const countB = counts[b] || 0;
            if (countB !== countA) return countB - countA;
            return a.localeCompare(b);
        });

        sortedSources.forEach(source => {
            const count = counts[source] || 0;
            const config = allConfigs[source];
            const isActive = this.currentFilter === source;

            // Only show sources with count > 0 or if important (e.g. Korben)
            // Or just show all? The previous logic showed 0 counts.
            // Let's keep showing 0 counts so user knows it's checked but empty.

            html += `
                <div class="stat-box ${isActive ? 'active-filter' : ''}" 
                     onclick="filterNews('${source}')"
                     style="cursor: pointer; transition: all 0.2s; border-color: ${config.color}; ${isActive ? 'background: rgba(255,255,255,0.1); box-shadow: 0 0 10px ' + config.color : ''}">
                    <div class="stat-value">${count}</div>
                    <div class="stat-label">${config.icon} ${config.label}</div>
                </div>
            `;
        });

        statsGrid.innerHTML = html;
    },

    /**
     * Get all source configurations
     */
    getAllSourceConfigs() {
        return {
            hackernews: { color: '#ff6600', icon: '🟠', label: 'Hacker News' },
            korben: { color: '#4a9eff', icon: '🔵', label: 'Korben' },
            github: { color: '#a371f7', icon: '🟣', label: 'GitHub' },
            cncf: { color: '#0086FF', icon: '📰', label: 'CNCF' },
            aws: { color: '#FF9900', icon: '☁️', label: 'AWS' },
            'aws-new': { color: '#FF9900', icon: '🆕', label: 'AWS New' },
            gcp: { color: '#4285F4', icon: '☁️', label: 'GCP' },
            azure: { color: '#0078D4', icon: '☁️', label: 'Azure' },
            kubernetes: { color: '#326CE5', icon: '☸️', label: 'K8s' },
            fluxcd: { color: '#2d343a', icon: '🔄', label: 'FluxCD' },
            rust: { color: '#DEA584', icon: '🦀', label: 'Rust' },
            'inside-rust': { color: '#DEA584', icon: '🔧', label: 'Inside Rust' },
            twir: { color: '#DEA584', icon: '📰', label: 'This Week in Rust' }
        };
    },

    /**
     * Render dynamic filter buttons
     */
    renderFilterButtons() {
        // Buttons removed - filtering is now done via the stats boxes
        const container = document.getElementById('news-filter-buttons');
        if (container) {
            container.style.display = 'none';
        }
    },

    /**
     * Get source configuration (color, icon, label)
     */
    getSourceConfig(source) {
        const configs = {
            hackernews: { color: '#ff6600', icon: '🟠', label: 'Hacker News' },
            korben: { color: '#4a9eff', icon: '🔵', label: 'Korben' },
            github: { color: '#a371f7', icon: '🟣', label: 'GitHub' },
            cncf: { color: '#0086FF', icon: '📰', label: 'CNCF' },
            aws: { color: '#FF9900', icon: '☁️', label: 'AWS' },
            'aws-new': { color: '#FF9900', icon: '🆕', label: 'AWS New' },
            gcp: { color: '#4285F4', icon: '☁️', label: 'GCP' },
            azure: { color: '#0078D4', icon: '☁️', label: 'Azure' },
            kubernetes: { color: '#326CE5', icon: '☸️', label: 'K8s' },
            fluxcd: { color: '#2d343a', icon: '🔄', label: 'FluxCD' },
            rust: { color: '#DEA584', icon: '🦀', label: 'Rust' },
            'inside-rust': { color: '#DEA584', icon: '🔧', label: 'Inside Rust' },
            twir: { color: '#DEA584', icon: '📰', label: 'This Week in Rust' }
        };

        return configs[source] || {
            color: '#00ff88',
            icon: '📰',
            label: source.charAt(0).toUpperCase() + source.slice(1)
        };
    },

    /**
     * Update last updated timestamp
     */
    updateTimestamp(timestamp) {
        const timestampEl = document.getElementById('news-updated-at');
        if (timestampEl && timestamp) {
            const date = new Date(timestamp);
            const formatted = date.toLocaleString();
            timestampEl.textContent = formatted;
        }
    },

    /**
     * Filter news by source
     */
    filterBySource(source) {
        this.currentFilter = source;

        // Re-render stats to update active state (highlight selected box)
        this.updateStats({ items: this.allNews }); // We pass full list to re-calc counts correctly
        this.applyFilters();
    },

    /**
     * Search news by query
     */
    search(query) {
        this.searchQuery = query.toLowerCase();
        this.applyFilters();
    },

    /**
     * Apply current filters and search
     */
    applyFilters() {
        let filtered = this.allNews;

        // Apply source filter
        if (this.currentFilter !== 'all') {
            filtered = filtered.filter(item => item.source === this.currentFilter);
        }

        // Apply search filter
        if (this.searchQuery) {
            filtered = filtered.filter(item =>
                item.title.toLowerCase().includes(this.searchQuery) ||
                (item.description && item.description.toLowerCase().includes(this.searchQuery))
            );
        }

        this.filteredNews = filtered;
        this.renderNews();
    },

    /**
     * Toggle between grid and list view
     */
    toggleViewMode(mode) {
        this.viewMode = mode;

        // Update button states
        document.getElementById('btn-view-card')?.classList.toggle('active', mode === 'grid');
        document.getElementById('btn-view-list')?.classList.toggle('active', mode === 'list');
        document.getElementById('btn-view-inline')?.classList.toggle('active', mode === 'inline');

        // Update container class
        const container = document.getElementById('news-container');
        if (container) {
            container.classList.remove('list-mode', 'inline-mode');
            if (mode === 'list') container.classList.add('list-mode');
            if (mode === 'inline') container.classList.add('inline-mode');
        }

        this.renderNews();
    },

    /**
     * Render news cards
     */
    renderNews() {
        const container = document.getElementById('news-container');
        if (!container) return;

        if (this.filteredNews.length === 0) {
            container.innerHTML = `
                <div class="no-news" style="text-align: center; padding: 3rem; opacity: 0.6;">
                    <span style="font-size: 3rem;">📭</span>
                    <p>No news items found</p>
                </div>
            `;
            return;
        }

        const html = this.filteredNews.map(item => this.renderNewsCard(item)).join('');
        container.innerHTML = html;

        container.classList.remove('list-mode', 'inline-mode');
        if (this.viewMode === 'list') container.classList.add('list-mode');
        if (this.viewMode === 'inline') container.classList.add('inline-mode');
    },

    /**
     * Render single news card
     */
    renderNewsCard(item) {
        const config = this.getSourceConfig(item.source);

        const color = config.color;
        const icon = config.icon;
        const label = config.label;

        const date = new Date(item.published_at);
        const timeAgo = this.formatTimeAgo(date);

        // Use translated title/description if available for current locale
        const currentLang = typeof LocaleManager !== 'undefined' ? LocaleManager.currentLocale : 'en';
        const translation = item.translations ? item.translations[currentLang] : null;

        const title = translation ? translation.title : item.title;
        const description = (translation && translation.description) ? translation.description : item.description;

        return `
            <div class="news-card ${this.viewMode === 'list' ? 'list-mode' : (this.viewMode === 'inline' ? 'inline-mode' : '')}" style="border-color: ${color};">
                <div class="news-header">
                    <span class="news-source-badge" style="background: ${color};">
                        ${icon} ${label}
                    </span>
                    <span class="news-time">${timeAgo}</span>
                </div>
                <h3 class="news-title">
                    <a href="${item.url}" target="_blank" rel="noopener noreferrer">
                        ${title}
                    </a>
                </h3>
                ${description ? `<p class="news-description">${this.truncate(description, 150)}</p>` : ''}
                <div class="news-footer">
                    ${item.score ? `<span class="news-score">⭐ ${item.score}</span>` : ''}
                    ${item.tags && item.tags.length > 0 ? `
                        <div class="news-tags">
                            ${item.tags.slice(0, 5).map(tag => `<span class="news-tag">#${tag}</span>`).join('')}
                        </div>
                    ` : ''}
                </div>
            </div>
        `;
    },

    /**
     * Format time ago
     */
    formatTimeAgo(date) {
        const now = new Date();
        const diff = now - date;
        const minutes = Math.floor(diff / 60000);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);

        if (days > 7) return date.toLocaleDateString();
        if (days > 0) return `${days}d ago`;
        if (hours > 0) return `${hours}h ago`;
        if (minutes > 0) return `${minutes}m ago`;
        return 'just now';
    },

    /**
     * Truncate text
     */
    truncate(text, maxLength) {
        if (text.length <= maxLength) return text;
        return text.substring(0, maxLength) + '...';
    },

    /**
     * Render error state
     */
    renderError(message) {
        const container = document.getElementById('news-container');
        if (!container) return;

        container.innerHTML = `
            <div class="error-state" style="text-align: center; padding: 3rem;">
                <span class="error-icon" style="font-size: 3rem;">⚠️</span>
                <p>Failed to load news: ${message}</p>
                <button onclick="NewsManager.fetchNews()" class="cyber-btn">Retry</button>
            </div>
        `;
    }
};

/**
 * Pod logs display manager
 */
const LogsManager = {
    currentPod: null,
    currentNamespace: null,

    openModal(namespace, podName) {
        this.currentPod = podName;
        this.currentNamespace = namespace;

        document.getElementById('logs-modal-title').textContent = `📄 Pod Logs: ${namespace}/${podName}`;
        document.getElementById('log-content').textContent = 'Loading logs...';
        const modal = document.getElementById('logs-modal');
        if (modal) modal.style.display = 'flex';

        this.refreshLogs();
    },

    closeModal() {
        const modal = document.getElementById('logs-modal');
        if (modal) modal.style.display = 'none';
        this.currentPod = null;
        this.currentNamespace = null;
    },

    async refreshLogs() {
        if (!this.currentPod || !this.currentNamespace) return;

        const tail = document.getElementById('log-tail-select').value;
        const container = document.getElementById('log-content');

        try {
            const response = await fetch(`/api/pods/${this.currentNamespace}/${this.currentPod}/logs?tail=${tail}`);
            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || response.statusText);
            }
            const logs = await response.text();
            container.textContent = logs || 'No logs found.';
            // Scroll to bottom
            container.scrollTop = container.scrollHeight;
        } catch (error) {
            console.error('Failed to refresh logs:', error);
            container.textContent = `Error loading logs: ${error.message}`;
        }
    }
};

window.LogsManager = LogsManager;

async function exportAlertsForAgent() {
    try {
        const response = await fetch('/api/export/alerts');
        if (!response.ok) throw new Error('Failed to export alerts');

        const blob = await response.blob();
        const url = window.URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'agent-remediation-context.md';
        document.body.appendChild(a);
        a.click();
        window.URL.revokeObjectURL(url);
        document.body.removeChild(a);
    } catch (error) {
        console.error('Export failed:', error);
        alert('Failed to export agent remediation context');
    }
}

window.exportAlertsForAgent = exportAlertsForAgent;

// Global functions for HTML onclick handlers
window.fetchNews = () => NewsManager.fetchNews();
window.filterNews = (source) => NewsManager.filterBySource(source);
window.searchNews = (query) => NewsManager.search(query);

// Global function for button click
window.fetchQuotas = () => QuotasManager.fetchQuotas();

// Auto-initialize on load
if (typeof window !== 'undefined') {
    window.DashboardManager = DashboardManager;
    window.MetricsManager = MetricsManager;
    window.AlertsManager = AlertsManager;
    window.QuotasManager = QuotasManager;
    window.NewsManager = NewsManager;

    document.addEventListener('DOMContentLoaded', () => {
        // Initialize Core UI systems first
        if (window.LocaleManager) LocaleManager.init();
        if (window.ThemeManager) ThemeManager.init();

        // Initialize Feature Managers
        DashboardManager.init();
        MetricsManager.init();
        AlertsManager.init();
        QuotasManager.init();
        NewsManager.init();

        // Initialize Specialized Managers
        if (window.K8sManager) K8sManager.init();
        if (window.ProxmoxDashboard) ProxmoxDashboard.init();
        if (window.HomeAssistantDashboard) HomeAssistantDashboard.init();
        if (window.CalendarManager) CalendarManager.init();
        if (window.WeatherManager) WeatherManager.init();
        if (window.MqttManager) MqttManager.init();
        if (window.SecurityManager) SecurityManager.init();
        if (window.NetworkManager) NetworkManager.init();
        if (window.SetupManager) SetupManager.init();

        // Initial Data Sync - DISABLED: Now manual only via refresh button
        // if (window.refreshAllKusanagiData) {
        //     refreshAllKusanagiData();
        //     // Global Refresh Interval (30s)
        //     setInterval(refreshAllKusanagiData, 30000);
        // }
    });
}


