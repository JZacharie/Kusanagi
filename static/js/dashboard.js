/**
 * Kusanagi Dashboard Manager
 * Handles customizable widgets, layout persistence, and export functionality
 */

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
            /* 
            if (this.startTime && this.startTime !== data.start_time) {
                console.log('🔄 Backend restart detected! Refreshing UI disabled by user request.');
                // showNotification('Kusanagi updated. Refreshing dashboard...', 'info');
                // setTimeout(() => window.location.reload(), 2000);
                // return;
            }
            */
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
        const indicator = document.getElementById('kusanagi-refresh-indicator');

        if (uptimeEl) uptimeEl.textContent = this.formatUptime(data.uptime_secs);
        if (cpuEl) cpuEl.textContent = `${data.cpu_usage_percent.toFixed(1)}%`;
        if (ramEl) ramEl.textContent = `${(data.memory_usage_bytes / (1024 * 1024)).toFixed(0)} MB`;
        if (versionEl) versionEl.textContent = data.version;

        // Visual flash on update
        if (indicator) {
            indicator.style.opacity = '1';
            setTimeout(() => { if (indicator) indicator.style.opacity = '0.5'; }, 500);
        }
    },

    formatUptime(seconds) {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = seconds % 60;
        return [h, m, s].map(v => v < 10 ? '0' + v : v).join(':');
    }
};

const DashboardManager = {
    // Available widgets configuration
    widgets: {
        argocd: { name: 'ArgoCD', icon: '🚀', enabled: true, order: 0 },
        nodes: { name: 'Nodes', icon: '🖥️', enabled: true, order: 1 },
        storage: { name: 'Storage', icon: '💾', enabled: true, order: 2 },
        events: { name: 'Events', icon: '🔔', enabled: true, order: 3 },
        pods: { name: 'Pods', icon: '📦', enabled: true, order: 4 },
        network: { name: 'Network', icon: '🌐', enabled: true, order: 5 },
        metrics: { name: 'Metrics', icon: '📊', enabled: true, order: 6 },
        alerts: { name: 'Alerts', icon: '⚠️', enabled: true, order: 7 },
        chat: { name: 'Chat', icon: '💬', enabled: true, order: 8 },
        proxmox: { name: 'Proxmox', icon: '🖥️', enabled: true, order: 9 },
        homeassistant: { name: 'Home Assistant', icon: '🏠', enabled: true, order: 10 },
        weather: { name: 'Weather', icon: '🌤️', enabled: true, order: 11 },
        calendar: { name: 'Calendar', icon: '📅', enabled: true, order: 12 }
    },

    storageKey: 'kusanagi_dashboard_layout',

    /**
     * Initialize dashboard manager
     */
    init() {
        SystemStatusManager.init();
        this.loadLayout();
        this.setupEventListeners();
        this.updateClocks();
        setInterval(() => this.updateClocks(), 1000);
        console.log('✅ Dashboard Manager initialized');
    },

    updateClocks() {
        try {
            const now = new Date();
            const clockLyon = document.getElementById('clock-lyon');
            const clockMexico = document.getElementById('clock-mexico');
            const clockNYC = document.getElementById('clock-nyc');

            if (clockLyon) clockLyon.textContent = now.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit', second: '2-digit' });

            if (clockMexico) clockMexico.textContent = now.toLocaleTimeString('en-GB', {
                hour: '2-digit', minute: '2-digit', second: '2-digit',
                timeZone: 'America/Mexico_City'
            });

            if (clockNYC) clockNYC.textContent = now.toLocaleTimeString('en-GB', {
                hour: '2-digit', minute: '2-digit', second: '2-digit',
                timeZone: 'America/New_York'
            });
        } catch (e) {
            console.error('Failed to update clocks:', e);
        }
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
     * Load Prometheus metrics
     */
    async loadMetrics() {
        try {
            const response = await fetch('/api/prometheus/metrics');
            if (!response.ok) {
                throw new Error('Failed to fetch metrics');
            }

            const metrics = await response.json();
            this.renderMetrics(metrics);
        } catch (error) {
            console.error('Metrics error:', error);
            this.renderMetricsError(error.message);
        }
    },

    /**
     * Render metrics to UI
     */
    renderMetrics(metrics) {
        const container = document.getElementById('metrics-content');
        if (!container) return;

        container.innerHTML = `
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
                <!-- Custom Dashboards -->
                <div class="metric-card">
                    <div class="metric-icon">🎮</div>
                    <div class="metric-value">${metrics.sim_app_status?.toFixed(0) || 0}%</div>
                    <div class="metric-label">SIM App Health</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${metrics.sim_app_status || 0}%; background: var(--neon-cyan);"></div>
                    </div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">⚡</div>
                    <div class="metric-value">${metrics.sim_realtime_status?.toFixed(0) || 0}%</div>
                    <div class="metric-label">SIM Realtime</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${metrics.sim_realtime_status || 0}%; background: var(--neon-magenta);"></div>
                    </div>
                </div>
                <div class="metric-card">
                    <div class="metric-icon">🏎️</div>
                    <div class="metric-value">${metrics.gpu_utilization?.toFixed(1) || 0}%</div>
                    <div class="metric-label">GPU Utilization</div>
                    <div class="metric-bar">
                        <div class="metric-bar-fill" style="width: ${metrics.gpu_utilization || 0}%; background: var(--neon-green);"></div>
                    </div>
                </div>
            </div>
        `;
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
    refreshInterval: null,

    /**
     * Initialize alerts display
     */
    init() {
        this.loadAlerts();
        // Refresh every 30 seconds
        this.refreshInterval = setInterval(() => this.loadAlerts(), 30000);
    },

    /**
     * Load alerts from Alertmanager
     */
    async loadAlerts() {
        try {
            const response = await fetch('/api/alerts');
            if (!response.ok) {
                throw new Error('Failed to fetch alerts');
            }

            const alerts = await response.json();
            this.renderAlerts(alerts);
            this.updateAlertBadge(alerts.total);
        } catch (error) {
            console.error('Alerts error:', error);
            this.renderAlertsError(error.message);
        }
    },

    /**
     * Render alerts to UI
     */
    renderAlerts(data) {
        const container = document.getElementById('alerts-content');
        if (!container) return;

        if (data.total === 0) {
            container.innerHTML = `
                <div class="no-alerts">
                    <span class="success-icon">✅</span>
                    <p>No active alerts</p>
                </div>
            `;
            return;
        }

        let html = '<div class="alerts-list">';

        // Critical alerts
        if (data.critical.length > 0) {
            html += '<div class="alert-group critical">';
            html += '<h4>🔴 Critical (${data.critical.length})</h4>';
            data.critical.forEach(alert => {
                html += this.renderAlertCard(alert, 'critical');
            });
            html += '</div>';
        }

        // Warning alerts
        if (data.warning.length > 0) {
            html += '<div class="alert-group warning">';
            html += `<h4>🟠 Warning (${data.warning.length})</h4>`;
            data.warning.forEach(alert => {
                html += this.renderAlertCard(alert, 'warning');
            });
            html += '</div>';
        }

        // Info alerts
        if (data.info.length > 0) {
            html += '<div class="alert-group info">';
            html += `<h4>🔵 Info (${data.info.length})</h4>`;
            data.info.forEach(alert => {
                html += this.renderAlertCard(alert, 'info');
            });
            html += '</div>';
        }

        html += '</div>';
        container.innerHTML = html;
    },

    /**
     * Render single alert card
     */
    renderAlertCard(alert, severity) {
        const age = this.formatAge(new Date(alert.started_at));
        return `
            <div class="alert-card ${severity}">
                <div class="alert-header">
                    <span class="alert-name">${alert.name}</span>
                    <span class="alert-state ${alert.state}">${alert.state}</span>
                </div>
                <div class="alert-summary">${alert.summary}</div>
                <div class="alert-meta">
                    ${alert.namespace ? `<span class="alert-ns">📁 ${alert.namespace}</span>` : ''}
                    ${alert.pod ? `<span class="alert-pod">📦 ${alert.pod}</span>` : ''}
                    <span class="alert-age">⏱️ ${age}</span>
                </div>
            </div>
        `;
    },

    /**
     * Format alert age
     */
    formatAge(date) {
        const now = new Date();
        const diff = now - date;
        const minutes = Math.floor(diff / 60000);
        const hours = Math.floor(minutes / 60);
        const days = Math.floor(hours / 24);

        if (days > 0) return `${days}d ${hours % 24}h`;
        if (hours > 0) return `${hours}h ${minutes % 60}m`;
        return `${minutes}m`;
    },

    /**
     * Update alert badge in navigation
     */
    updateAlertBadge(count) {
        const badge = document.getElementById('alerts-badge');
        if (badge) {
            badge.textContent = count;
            badge.style.display = count > 0 ? 'inline-block' : 'none';
        }
    },

    /**
     * Render error state
     */
    renderAlertsError(message) {
        const container = document.getElementById('alerts-content');
        if (!container) return;

        container.innerHTML = `
            <div class="error-state">
                <span class="error-icon">⚠️</span>
                <p>Failed to load alerts: ${message}</p>
                <button onclick="AlertsManager.loadAlerts()" class="retry-btn">Retry</button>
            </div>
        `;
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
        this.fetchQuotas();
        // Refresh every 60 seconds
        setInterval(() => this.fetchQuotas(), 60000);
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
        document.getElementById('quota-updated-at').textContent = data.last_updated;
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
        // Auto-refresh every 5 minutes
        setInterval(() => this.fetchNews(), 300000);
    },

    /**
     * Fetch news from API
     */
    async fetchNews() {
        try {
            const response = await fetch('/api/news');
            if (!response.ok) {
                throw new Error('Failed to fetch news');
            }

            const data = await response.json();
            this.allNews = data.items || [];
            this.updateStats(data);
            this.updateTimestamp(data.cached_at);
            this.applyFilters();
        } catch (error) {
            console.error('News fetch error:', error);
            this.renderError(error.message);
        }
    },

    /**
     * Update news statistics
     */
    updateStats(data) {
        const hnCount = data.items.filter(n => n.source === 'hackernews').length;
        const korbenCount = data.items.filter(n => n.source === 'korben').length;
        const ghCount = data.items.filter(n => n.source === 'github').length;

        document.getElementById('news-total').textContent = data.total || 0;
        document.getElementById('news-hn').textContent = hnCount;
        document.getElementById('news-korben').textContent = korbenCount;
        document.getElementById('news-github').textContent = ghCount;
    },

    /**
     * Update last updated timestamp
     */
    updateTimestamp(timestamp) {
        const date = new Date(timestamp);
        const formatted = date.toLocaleString();
        document.getElementById('news-updated-at').textContent = formatted;
    },

    /**
     * Filter news by source
     */
    filterBySource(source) {
        this.currentFilter = source;

        // Update button states
        document.querySelectorAll('.filter-controls .cyber-btn').forEach(btn => {
            btn.classList.remove('active');
        });

        const btnId = source === 'all' ? 'btn-news-all' : `btn-news-${source}`;
        document.getElementById(btnId)?.classList.add('active');

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
        const sourceColors = {
            hackernews: '#ff6600',
            korben: '#4a9eff',
            github: '#a371f7'
        };

        const sourceIcons = {
            hackernews: '🟠',
            korben: '🔵',
            github: '🟣'
        };

        const sourceLabels = {
            hackernews: 'Hacker News',
            korben: 'Korben',
            github: 'GitHub'
        };

        const color = sourceColors[item.source] || '#00ff88';
        const icon = sourceIcons[item.source] || '📰';
        const label = sourceLabels[item.source] || item.source;

        const date = new Date(item.published_at);
        const timeAgo = this.formatTimeAgo(date);

        // Use translated title/description if available
        const title = item.translated_title || item.title;
        const description = item.translated_description || item.description;

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
                            ${item.tags.map(tag => `<span class="news-tag">#${tag}</span>`).join('')}
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
        DashboardManager.init();
        // Initialize other managers as needed
        MetricsManager.init();
        AlertsManager.init();
        QuotasManager.init();
        NewsManager.init();
    });
}
