/**
 * Kusanagi Monitors Manager
 * Handles the unified view of Alerts and Events
 * Note: Polling is handled by TabManager (tab-aware)
 */
const MonitorsManager = {
    data: [],
    currentFilter: 'all',

    init() {
        console.log('✅ Monitors Manager initialized (no internal polling)');
        // Ne pas fetch ici - TabManager s'en charge quand l'onglet est actif
    },

    async fetchMonitors() {
        try {
            const container = document.getElementById('monitors-content');
            if (this.data.length === 0 && container) {
                container.innerHTML = '<div class="loading">Loading monitors...</div>';
            }

            const response = await fetch('/api/fusion');
            if (!response.ok) throw new Error(`HTTP ${response.status}`);

            const result = await response.json();

            if (result.status === 'success') {
                this.data = result.data || [];
                this.updateStats();
                this.render();
            } else {
                throw new Error(result.message || 'Unknown error');
            }
        } catch (error) {
            console.error('Monitors fetch error:', error);
            this.renderError(error.message);
        }
    },

    updateStats() {
        const stats = {
            total: this.data.length,
            critical: this.data.filter(e => e.severity === 'critical').length,
            warning: this.data.filter(e => e.severity === 'warning').length,
            info: this.data.filter(e => e.severity !== 'critical' && e.severity !== 'warning').length
        };

        const setStat = (id, value) => {
            const el = document.getElementById(id);
            if (el) el.textContent = value;
        };

        setStat('monitors-total', stats.total);
        setStat('monitors-critical', stats.critical);
        setStat('monitors-warning', stats.warning);
        setStat('monitors-info', stats.info);

        // Update badge in sidebar if it exists
        const badge = document.getElementById('monitors-badge');
        if (badge) {
            const issues = stats.critical + stats.warning;
            badge.textContent = issues;
            badge.style.display = issues > 0 ? 'inline-block' : 'none';
            if (stats.critical > 0) badge.style.background = '#ff4444';
            else if (stats.warning > 0) badge.style.background = '#ffaa00';
            else badge.style.background = '';
        }
    },

    filter(type) {
        this.currentFilter = type;

        // Update buttons
        document.querySelectorAll('.filter-controls .cyber-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        const btn = document.getElementById(`btn-monitor-${type}`);
        if (btn) btn.classList.add('active');

        this.render();
    },

    render() {
        const container = document.getElementById('monitors-content');
        if (!container) return; // Not on the page

        let filtered = this.data;
        if (this.currentFilter !== 'all') {
            if (this.currentFilter === 'info') {
                filtered = this.data.filter(e => e.severity !== 'critical' && e.severity !== 'warning');
            } else {
                filtered = this.data.filter(e => e.severity === this.currentFilter);
            }
        }

        const countEl = document.getElementById('monitors-table-count');
        if (countEl) countEl.textContent = filtered.length;

        if (filtered.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="text-align: center; padding: 3rem;">
                    <div style="font-size: 3rem; margin-bottom: 1rem;">✅</div>
                    <div>No monitors found for this filter</div>
                </div>
            `;
            return;
        }

        let html = '<div class="monitors-list" style="display: flex; flex-direction: column; gap: 10px;">';

        filtered.forEach(item => {
            html += this.renderCard(item);
        });

        html += '</div>';
        container.innerHTML = html;
    },

    renderCard(item) {
        const isCritical = item.severity === 'critical';
        const isWarning = item.severity === 'warning';
        const severityClass = isCritical ? 'critical' : (isWarning ? 'warning' : 'info');
        const borderColor = isCritical ? '#ff4444' : (isWarning ? '#ffaa00' : 'var(--neon-blue)');
        const bg = isCritical ? 'rgba(255, 68, 68, 0.05)' : (isWarning ? 'rgba(255, 170, 0, 0.05)' : 'rgba(0, 100, 255, 0.05)');

        const sourceIcon = item.source === 'alertmanager' ? '🚨' : '☸️';
        const date = new Date(item.timestamp);
        const timeStr = date.toLocaleTimeString();
        const dateStr = date.toLocaleDateString();

        return `
            <div class="monitor-card" style="
                background: ${bg}; 
                border-left: 4px solid ${borderColor}; 
                border-radius: 4px; 
                padding: 15px; 
                position: relative;
                transition: transform 0.2s;
                border: 1px solid rgba(255,255,255,0.05);
                border-left: 4px solid ${borderColor};
            ">
                <div style="display: flex; justify-content: space-between; margin-bottom: 5px;">
                    <div style="display: flex; align-items: center; gap: 10px;">
                        <span style="font-size: 1.2rem;">${sourceIcon}</span>
                        <span style="font-weight: bold; color: ${borderColor}; font-family: 'Orbitron', sans-serif;">
                            ${item.name}
                        </span>
                        <span class="status-badge ${severityClass}" style="
                            font-size: 0.7rem; 
                            background: ${borderColor}; 
                            color: #000;
                            border: none;
                        ">${item.severity.toUpperCase()}</span>
                    </div>
                    <div style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem; opacity: 0.7;">
                        ${dateStr} ${timeStr}
                    </div>
                </div>
                
                <div style="margin-bottom: 8px; color: var(--text-color);">
                    ${item.message}
                </div>
                
                <div style="display: flex; gap: 15px; font-size: 0.8rem; opacity: 0.7; font-family: 'JetBrains Mono', monospace;">
                    <span>📁 ${item.namespace}</span>
                    <span>🏷️ ${item.event_type}</span>
                </div>
            </div>
        `;
    },

    renderError(msg) {
        const container = document.getElementById('monitors-content');
        if (container) {
            container.innerHTML = `
                <div class="error-state" style="text-align: center; padding: 2rem; border: 1px solid #ff4444; border-radius: 8px;">
                    <h3 style="color: #ff4444;">⚠️ Error Loading Monitors</h3>
                    <p>${msg}</p>
                    <button class="cyber-btn" onclick="MonitorsManager.fetchMonitors()">Retry</button>
                </div>
            `;
        }
    },

    exportMonitors() {
        const json = JSON.stringify(this.data, null, 2);
        const blob = new Blob([json], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `monitors-export-${new Date().toISOString()}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
    }
};

// Expose globally
window.MonitorsManager = MonitorsManager;
