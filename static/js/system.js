const KusanagiSystem = {
    refreshInterval: null,

    init: function () {
        console.log("KusanagiSystem initialized");
        // Initial setup only, actual data fetching happens in activate()
    },

    activate: function () {
        console.log("KusanagiSystem activated");
        this.fetchSystemStatus();
        this.fetchSystemLogs();
        this.fetchDatabaseHealth();

        // Clear any existing interval
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
        }

        // Refresh every 10 seconds while active
        this.refreshInterval = setInterval(() => {
            this.fetchSystemStatus();
            this.fetchSystemLogs();
            this.fetchDatabaseHealth();
        }, 10000);
    },

    deactivate: function () {
        console.log("KusanagiSystem deactivated");
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
            this.refreshInterval = null;
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
            const el = document.getElementById('kusanagi-db-status');
            if (el) {
                el.textContent = data.latency_ms + 'ms';

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

    fetchSystemLogs: async function () {
        try {
            const container = document.getElementById('system-logs-content');
            if (!container) return;

            // Show loading state if empty
            if (container.textContent === 'Loading logs...') {
                container.textContent = 'Fetching...';
            }

            // Note: System logs endpoint returns raw text, not JSON envelope
            const response = await fetch('/api/system/logs');
            if (response.ok) {
                const logs = await response.text();

                // Parse ANSI codes if present
                if (window.AnsiParser) {
                    container.innerHTML = AnsiParser.parseToHtml(logs);
                } else {
                    container.textContent = logs || "No logs available.";
                }
            } else {
                // Handle non-ok response
                container.innerHTML = `<div style="color: var(--neon-orange); padding: 1rem;">
                    ⚠️ Logs unavailable. 
                    <button onclick="KusanagiSystem.fetchSystemLogs()" class="cyber-btn" style="margin-left: 1rem;">Retry</button>
                </div>`;
            }
            // Auto-scroll to bottom
            const logsContainer = document.getElementById('system-logs-container');
            if (logsContainer) {
                logsContainer.scrollTop = logsContainer.scrollHeight;
            }
        } catch (error) {
            console.error("Failed to fetch logs:", error);
            const container = document.getElementById('system-logs-content');
            if (container) {
                container.innerHTML = `<div style="color: var(--neon-orange); padding: 1rem;">
                    ⚠️ Error loading logs. 
                    <button onclick="KusanagiSystem.fetchSystemLogs()" class="cyber-btn" style="margin-left: 1rem;">Retry</button>
                </div>`;
            }
        }
    },

    manualRefresh: function () {
        this.fetchSystemStatus();
        this.fetchSystemLogs();
    },

    formatUptime: function (seconds) {
        if (!seconds) return 'N/A';
        const hours = Math.floor(seconds / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = seconds % 60;
        return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    }
};

// Helper for setting text content safely
function setText(id, text) {
    const el = document.getElementById(id);
    if (el) el.textContent = text;
}

window.KusanagiSystem = KusanagiSystem;
