const LimitsDashboard = {
    data: null,
    refreshInterval: null,

    init() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
        }
        this.loadLimits();
        this.refreshInterval = setInterval(() => this.loadLimits(), 30000);
    },

    async loadLimits() {
        try {
            const response = await fetch('/api/k8s/limits');
            const result = await response.json();

            if (!result.success) {
                this.showError(result.error || 'Failed to load limits data');
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

        const apps = this.data.applications || [];
        const total = this.data.total || {};

        document.getElementById('limits-ns-count').textContent = total.namespaces || 0;
        document.getElementById('limits-cpu-usage').textContent = (total.cpu_cores_usage || 0).toFixed(2);
        document.getElementById('limits-cpu-limit').textContent = `${(total.cpu_cores_limit || 0).toFixed(2)} cores limit`;
        document.getElementById('limits-cpu-pct').textContent = `${total.cpu_utilization_percent || 0}%`;
        document.getElementById('limits-cpu-bar').style.width = `${Math.min(total.cpu_utilization_percent || 0, 100)}%`;

        document.getElementById('limits-mem-usage').textContent = `${(total.memory_gb_usage || 0).toFixed(2)} GB`;
        document.getElementById('limits-mem-limit').textContent = `${(total.memory_gb_limit || 0).toFixed(2)} GB limit`;
        document.getElementById('limits-mem-pct').textContent = `${total.memory_utilization_percent || 0}%`;
        document.getElementById('limits-mem-bar').style.width = `${Math.min(total.memory_utilization_percent || 0, 100)}%`;

        const gpuNs = apps.filter(a => a.gpu && a.gpu.available).length;
        document.getElementById('limits-gpu-ns').textContent = gpuNs;

        const tbody = document.getElementById('limits-tbody');
        if (apps.length === 0) {
            tbody.innerHTML = `<tr><td colspan="13" style="text-align: center; padding: 2rem; color: var(--text-secondary);">No applications found</td></tr>`;
            return;
        }

        apps.sort((a, b) => (b.cpu.usage || 0) - (a.cpu.usage || 0));

        tbody.innerHTML = apps.map(app => {
            const cpu = app.cpu || {};
            const mem = app.memory || {};
            const gpu = app.gpu || {};
            const net = app.network || {};

            const cpuPct = cpu.usage_percent || 0;
            const memPct = mem.usage_percent || 0;
            const cpuColor = cpuPct > 80 ? 'var(--neon-red)' : cpuPct > 50 ? 'var(--neon-orange)' : 'var(--neon-green)';
            const memColor = memPct > 80 ? 'var(--neon-red)' : memPct > 50 ? 'var(--neon-orange)' : 'var(--neon-green)';

            const gpuDisplay = gpu.available
                ? `<span style="color: var(--neon-magenta);">${(gpu.usage_percent || 0).toFixed(0)}%</span>`
                : '<span style="opacity: 0.4;">-</span>';

            return `
                <tr class="monitor-row" style="animation: fadeInRow 0.3s ease-out;">
                    <td style="font-weight: 600;">${this.escapeHtml(app.namespace || 'default')}</td>
                    <td style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">${(cpu.usage || 0).toFixed(3)}</td>
                    <td style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">${(cpu.requests || 0).toFixed(3)}</td>
                    <td style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">${(cpu.limits || 0).toFixed(3)}</td>
                    <td>
                        <div class="mini-bar" style="display: flex; align-items: center; gap: 0.5rem;">
                            <div style="flex: 1; height: 6px; background: rgba(255,255,255,0.1); border-radius: 3px;">
                                <div style="height: 100%; background: ${cpuColor}; border-radius: 3px; width: ${Math.min(cpuPct, 100)}%; transition: width 0.5s;"></div>
                            </div>
                            <span style="font-size: 0.75rem; color: ${cpuColor};">${cpuPct.toFixed(0)}%</span>
                        </div>
                    </td>
                    <td style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">${(mem.usage_mb || 0).toFixed(0)} MB</td>
                    <td style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">${(mem.requests_mb || 0).toFixed(0)} MB</td>
                    <td style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">${(mem.limits_mb || 0).toFixed(0)} MB</td>
                    <td>
                        <div class="mini-bar" style="display: flex; align-items: center; gap: 0.5rem;">
                            <div style="flex: 1; height: 6px; background: rgba(255,255,255,0.1); border-radius: 3px;">
                                <div style="height: 100%; background: ${memColor}; border-radius: 3px; width: ${Math.min(memPct, 100)}%; transition: width 0.5s;"></div>
                            </div>
                            <span style="font-size: 0.75rem; color: ${memColor};">${memPct.toFixed(0)}%</span>
                        </div>
                    </td>
                    <td>${gpuDisplay}</td>
                    <td style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">${this.formatBits(net.rx_mbps || 0)}</td>
                    <td style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">${this.formatBits(net.tx_mbps || 0)}</td>
                    <td style="text-align: center;">${app.pod_count || 0}</td>
                </tr>
            `;
        }).join('');
    },

    formatBits(mbps) {
        if (mbps >= 1000) return `${(mbps / 1000).toFixed(2)} Gbps`;
        if (mbps >= 1) return `${mbps.toFixed(2)} Mbps`;
        return `${(mbps * 1000).toFixed(0)} Kbps`;
    },

    escapeHtml(str) {
        if (str == null) return '';
        const div = document.createElement('div');
        div.textContent = String(str);
        return div.innerHTML;
    },

    showError(message) {
        const tbody = document.getElementById('limits-tbody');
        if (tbody) {
            tbody.innerHTML = `<tr><td colspan="13" style="text-align: center; padding: 2rem; color: var(--neon-red);">${this.escapeHtml(message)}</td></tr>`;
        }
    }
};

window.LimitsDashboard = LimitsDashboard;
