/**
 * KusanagiSystem - System status and logs
 * Note: Polling is handled by TabManager (tab-aware)
 */
const KusanagiSystem = {
    _logsInterval: null,
    _lastLogsContent: '',

    init: function () {
        console.log("KusanagiSystem initialized (no internal polling)");
    },

    activate: function () {
        console.log("KusanagiSystem activated");
        this.refresh();
        this.startLiveLogs();
    },

    deactivate: function () {
        console.log("KusanagiSystem deactivated");
        this.stopLiveLogs();
    },

    // Alias pour TabManager
    refresh: function () {
        this.fetchSystemStatus();
        this.fetchSystemLogs();
        this.fetchDatabaseHealth();
    },

    // Start live log polling (every 3 seconds when tab is active)
    startLiveLogs: function () {
        if (this._logsInterval) return;
        console.log('📋 Starting live logs polling');
        this._logsInterval = setInterval(() => {
            if (!document.hidden) {
                this.fetchSystemLogs(true); // true = silent mode (no loading indicator)
            }
        }, 3000);
    },

    stopLiveLogs: function () {
        if (this._logsInterval) {
            clearInterval(this._logsInterval);
            this._logsInterval = null;
            console.log('📋 Stopped live logs polling');
        }
    },

    fetchSystemStatus: async function () {
        console.log('🔍 Fetching system status...');
        try {
            // Use apiFetch to get unwrapped data from the standard envelope
            const data = await api.get('/api/system/status');
            console.log('✅ System status data:', data);
            this.updateStatusUI(data);
        } catch (error) {
            console.error("❌ Failed to fetch system status:", error);
            this.updateStatusUI({
                uptime: 'N/A',
                cpu_usage: 0,
                memory_usage_mb: 0,
                version: 'Unknown',
                _warning: 'System status unavailable'
            });
        }
    },

    updateStatusUI: function (data) {
        console.log('🎨 Updating status UI with:', data);

        // Guard against undefined/null data
        if (!data) {
            console.warn('⚠️ updateStatusUI called with no data');
            data = {};
        }

        // Handle uptime - favor uptime_secs for consistency
        const uptimeDisplay = this.formatUptime(data.uptime_secs) || data.uptime || 'N/A';

        console.log('⏱️ Uptime display:', uptimeDisplay);
        setText('sys-tab-uptime', uptimeDisplay);
        setText('sys-tab-cpu', (data.cpu_usage ?? data.cpu_usage_percent) ? `${(data.cpu_usage ?? data.cpu_usage_percent).toFixed(1)}%` : '0%');
        setText('sys-tab-memory', (data.memory_usage_mb ?? (data.memory_usage_bytes / 1048576)) ? `${(data.memory_usage_mb ?? (data.memory_usage_bytes / 1048576)).toFixed(0)} MB` : '0 MB');
        setText('sys-tab-version', data.version || '0.3.0');

        // Also update the header status bar if present
        setText('kusanagi-uptime', uptimeDisplay);
        setText('kusanagi-cpu', data.cpu_usage ? `${data.cpu_usage.toFixed(1)}%` : '--%');
        setText('kusanagi-ram', data.memory_usage_mb ? `${data.memory_usage_mb.toFixed(0)}MB` : '--MB');
        console.log('✅ Status UI updated');
    },

    fetchDatabaseHealth: async function () {
        try {
            // Use apiFetch to get unwrapped data from the standard envelope
            const data = await api.get('/api/database/health');
            // Guard against undefined/null data
            if (!data) {
                console.warn('⚠️ Database health data is undefined');
                throw new Error('No data received');
            }
            const el = document.getElementById('kusanagi-db-status');
            if (el) {
                el.textContent = (data.latency_ms ?? 'N/A') + 'ms';

                if (data.status === 'Healthy') {
                    el.style.color = 'var(--neon-green)';
                    el.title = `Standard: ${data.version || 'Unknown'}`;
                } else {
                    el.style.color = '#ff4444';
                    el.textContent = 'ERR';
                    el.title = data.error || 'Unknown Error';
                }
            }
        } catch (error) {
            console.error("Failed to fetch DB health:", error);
            const el = document.getElementById('kusanagi-db-status');
            if (el) {
                el.textContent = 'OFF';
                el.style.color = '#ff4444';
            }
        }
    },

    fetchSystemLogs: async function (silent = false) {
        try {
            const container = document.getElementById('system-logs-content');
            if (!container) return;

            // Show loading indicator (only if not silent)
            if (!silent && container.textContent === 'Loading logs...') {
                container.textContent = 'Fetching...';
            }

            // Use api.get for consistent JSON envelope handling
            const data = await api.get('/api/system/logs');
            const logs = data?.logs || "No logs available.";

            // Check if content changed (for live indicator)
            const hasNewContent = this._lastLogsContent && logs !== this._lastLogsContent;
            this._lastLogsContent = logs;

            // Parse ANSI codes if present
            if (window.AnsiParser) {
                container.innerHTML = AnsiParser.parseToHtml(logs);
            } else {
                container.textContent = logs;
            }

            // Show live indicator if new content
            if (hasNewContent && silent) {
                this.showLiveIndicator();
            }

            // Auto-scroll to bottom (only if user hasn't scrolled up)
            const logsContainer = document.getElementById('system-logs-container');
            if (logsContainer) {
                const isScrolledToBottom = logsContainer.scrollHeight - logsContainer.clientHeight <= logsContainer.scrollTop + 50;
                if (isScrolledToBottom) {
                    logsContainer.scrollTop = logsContainer.scrollHeight;
                }
            }
        } catch (error) {
            console.error("Failed to fetch logs:", error);
            if (!silent) {
                const container = document.getElementById('system-logs-content');
                if (container) {
                    container.innerHTML = `<div style="color: var(--neon-orange); padding: 1rem;">
                        ⚠️ Logs unavailable: ${error.message}
                        <button onclick="KusanagiSystem.fetchSystemLogs()" class="cyber-btn" style="margin-left: 1rem;">Retry</button>
                    </div>`;
                }
            }
        }
    },

    showLiveIndicator: function () {
        let indicator = document.getElementById('logs-live-indicator');
        if (!indicator) {
            const container = document.getElementById('system-logs-container');
            if (!container) return;
            indicator = document.createElement('div');
            indicator.id = 'logs-live-indicator';
            indicator.style.cssText = `
                position: absolute;
                top: 8px;
                right: 8px;
                background: var(--neon-green);
                color: #000;
                padding: 4px 8px;
                border-radius: 4px;
                font-size: 0.7rem;
                font-weight: bold;
                z-index: 10;
                animation: pulse 1s infinite;
            `;
            indicator.textContent = '● LIVE';
            container.style.position = 'relative';
            container.appendChild(indicator);
        }
        // Clear previous timeout
        if (indicator._timeout) clearTimeout(indicator._timeout);
        indicator.style.display = 'block';
        // Hide after 2 seconds
        indicator._timeout = setTimeout(() => {
            indicator.style.display = 'none';
        }, 2000);
    },

    manualRefresh: function () {
        this.fetchSystemStatus();
        this.fetchSystemLogs();
    },

    formatUptime: function (seconds) {
        if (!seconds && seconds !== 0) return 'N/A';
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = seconds % 60;
        
        if (days > 0) {
            return `${days}j ${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
        }
        return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
};

// Helper for setting text content safely
function setText(id, text) {
    const el = document.getElementById(id);
    if (el) el.textContent = text;
}

window.KusanagiSystem = KusanagiSystem;
