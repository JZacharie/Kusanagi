/**
 * Kusanagi Dashboard Manager
 * Handles customizable widgets, layout persistence, and export functionality
 */

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
        chat: { name: 'Chat', icon: '💬', enabled: true, order: 8 }
    },

    storageKey: 'kusanagi_dashboard_layout',

    /**
     * Initialize dashboard manager
     */
    init() {
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

// Auto-initialize on load
if (typeof window !== 'undefined') {
    window.DashboardManager = DashboardManager;
    window.MetricsManager = MetricsManager;
    window.AlertsManager = AlertsManager;

    document.addEventListener('DOMContentLoaded', () => {
        DashboardManager.init();
    });
}
