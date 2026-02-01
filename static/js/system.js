const KusanagiSystem = {
    refreshInterval: null,
    isInitialized: false,

    init: function () {
        if (this.isInitialized) return;
        this.isInitialized = true;

        console.log("KusanagiSystem initialized");
        this.fetchSystemStatus();
        this.fetchSystemLogs();
        this.fetchDatabaseHealth();

        // Refresh logs every 10 seconds
        this.refreshInterval = setInterval(() => {
            if (document.querySelector('.tab-content[data-tab="system"]').style.display !== 'none') {
                this.fetchSystemStatus();
                this.fetchSystemLogs();
                this.fetchDatabaseHealth();
            }
        }, 10000);
    },

    fetchSystemStatus: async function () {
        try {
            const response = await fetch('/api/system/status');
            if (response.ok) {
                const data = await response.json();
                this.updateStatusUI(data);
            }
        } catch (error) {
            console.error("Failed to fetch system status:", error);
        }
    },

    updateStatusUI: function (data) {
        setText('sys-tab-uptime', data.uptime || 'N/A');
        setText('sys-tab-cpu', data.cpu_usage ? `${data.cpu_usage.toFixed(1)}%` : '0%');
        setText('sys-tab-memory', data.memory_usage_mb ? `${data.memory_usage_mb.toFixed(0)} MB` : '0 MB');
        setText('sys-tab-version', data.version || 'Unknown');

        // Also update the header status bar if present
        setText('kusanagi-uptime', data.uptime || '--:--:--');
        setText('kusanagi-cpu', data.cpu_usage ? `${data.cpu_usage.toFixed(1)}%` : '--%');
        setText('kusanagi-ram', data.memory_usage_mb ? `${data.memory_usage_mb.toFixed(0)}MB` : '--MB');
    },

    fetchDatabaseHealth: async function () {
        try {
            const response = await fetch('/api/database/health');
            const el = document.getElementById('kusanagi-db-status');
            if (response.ok && el) {
                const data = await response.json();
                el.textContent = data.latency_ms + 'ms';

                if (data.status === 'Healthy') {
                    el.style.color = 'var(--neon-green)';
                    el.title = `Standard: ${data.version || 'Unknown'}`;
                } else {
                    el.style.color = '#ff4444';
                    el.textContent = 'ERR';
                    el.title = data.error || 'Unknown Error';
                }
            } else if (el) {
                el.textContent = 'ERR';
                el.style.color = '#ff4444';
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

            const response = await fetch('/api/system/logs');
            if (response.ok) {
                const logs = await response.text();
                
                // Parse ANSI codes if present
                if (window.AnsiParser) {
                    container.innerHTML = AnsiParser.parseToHtml(logs);
                } else {
                    container.textContent = logs || "No logs available.";
                }
                // Auto-scroll to bottom
                const logsContainer = document.getElementById('system-logs-container');
                if (logsContainer) {
                    logsContainer.scrollTop = logsContainer.scrollHeight;
                }
            } else {
                container.textContent = "Failed to load logs.";
            }
        } catch (error) {
            console.error("Failed to fetch logs:", error);
            const container = document.getElementById('system-logs-content');
            if (container) container.textContent = "Error loading logs.";
        }
    },

    manualRefresh: function () {
        this.fetchSystemStatus();
        this.fetchSystemLogs();
    }
};

// Helper for setting text content safely
function setText(id, text) {
    const el = document.getElementById(id);
    if (el) el.textContent = text;
}
