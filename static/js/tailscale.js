const TailscaleDashboard = {
    data: null,
    refreshInterval: null,

    init() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
        }
        this.loadDevices();
        this.refreshInterval = setInterval(() => this.loadDevices(), 30000);
    },

    async loadDevices() {
        try {
            const response = await fetch('/api/tailscale/devices');
            const result = await response.json();

            if (!result.success) {
                this.showError(result.error || 'Failed to load Tailscale devices');
                return;
            }

            this.data = result.data;
            this.render();
        } catch (error) {
            this.showError(`Network error: ${error.message}`);
        }
    },

    render() {
        if (!this.data) return;

        const devices = this.data.devices || [];
        const total = this.data.total || 0;
        const online = this.data.online || 0;
        const offline = total - online;
        const exitNodes = devices.filter(d => d.is_exit_node).length;

        document.getElementById('tailscale-total').textContent = total;
        document.getElementById('tailscale-online').textContent = online;
        document.getElementById('tailscale-offline').textContent = offline;
        document.getElementById('tailscale-exit-nodes').textContent = exitNodes;

        const tbody = document.getElementById('tailscale-tbody');
        if (devices.length === 0) {
            tbody.innerHTML = `<tr><td colspan="6" style="text-align: center; padding: 2rem; color: var(--text-secondary);">No devices found</td></tr>`;
            return;
        }

        tbody.innerHTML = devices.map(device => {
            const name = device.name || device.hostname || 'Unknown';
            const ips = (device.addresses || []).join(', ') || (device.tailscale_ips || []).join(', ') || '-';
            const os = device.os || '-';
            const version = device.version || '-';
            const lastSeen = device.last_seen ? this.formatTime(device.last_seen) : '-';
            const isOnline = device.online;
            const tags = device.tags && device.tags.length > 0
                ? device.tags.join(', ')
                : (device.is_exit_node ? 'exit-node' : '-');
            const exitBadge = device.is_exit_node
                ? '<span class="status-badge warning" style="margin-left: 4px; font-size: 0.65rem;">EXIT</span>'
                : '';

            return `
                <tr class="monitor-row ${isOnline ? 'status-ok' : 'status-warning'}" style="animation: fadeInRow 0.3s ease-out;">
                    <td style="font-weight: 600;">
                        <span class="device-name">${this.escapeHtml(name)}</span>
                        ${exitBadge}
                    </td>
                    <td style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">${this.escapeHtml(ips)}</td>
                    <td>${this.escapeHtml(os)} / ${this.escapeHtml(version)}</td>
                    <td style="font-size: 0.8rem; opacity: 0.8;">${lastSeen}</td>
                    <td>
                        <span class="status-dot ${isOnline ? 'online' : 'offline'}"></span>
                        <span style="color: ${isOnline ? 'var(--neon-green)' : 'var(--neon-red)'}; font-size: 0.8rem;">
                            ${isOnline ? 'Online' : 'Offline'}
                        </span>
                    </td>
                    <td style="font-size: 0.8rem; max-width: 200px; overflow: hidden; text-overflow: ellipsis;">
                        ${this.escapeHtml(tags)}
                    </td>
                </tr>
            `;
        }).join('');
    },

    formatTime(isoString) {
        try {
            const date = new Date(isoString);
            const now = new Date();
            const diffMs = now - date;
            const diffMin = Math.floor(diffMs / 60000);

            if (diffMin < 1) return 'Just now';
            if (diffMin < 60) return `${diffMin}m ago`;

            const diffHours = Math.floor(diffMin / 60);
            if (diffHours < 24) return `${diffHours}h ago`;

            const diffDays = Math.floor(diffHours / 24);
            if (diffDays < 7) return `${diffDays}d ago`;

            return date.toLocaleDateString();
        } catch {
            return isoString;
        }
    },

    escapeHtml(str) {
        if (!str) return '';
        const div = document.createElement('div');
        div.textContent = str;
        return div.innerHTML;
    },

    showError(message) {
        const tbody = document.getElementById('tailscale-tbody');
        if (tbody) {
            tbody.innerHTML = `<tr><td colspan="6" style="text-align: center; padding: 2rem; color: var(--neon-red);">${this.escapeHtml(message)}</td></tr>`;
        }
    }
};

window.TailscaleDashboard = TailscaleDashboard;
