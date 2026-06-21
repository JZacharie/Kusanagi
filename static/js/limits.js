const LimitsDashboard = {
    data: null,
    refreshInterval: null,
    sortKey: 'cpu_usage',
    sortAsc: false,
    filterText: '',

    init() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
        }
        this.loadLimits();
        this.refreshInterval = setInterval(() => this.loadLimits(), 30000);
    },

    async loadLimits(forceRefresh = false) {
        try {
            const url = forceRefresh ? '/api/k8s/limits?refresh=true' : '/api/k8s/limits';
            const response = await fetch(url);
            const result = await response.json();
 
            if (!result.success) {
                this.showError(result.error || 'Failed to load limits data');
                return;
            }
 
            this.data = result.data;
            setTimeout(() => this.render(), 10);
        } catch (error) {
            this.showError(`Network error: ${error.message}`);
        }
    },
 
    async refreshData() {
        const btn = document.getElementById('btn-limits-refresh');
        if (btn) {
            btn.disabled = true;
            btn.style.opacity = 0.5;
            btn.innerHTML = 'Refreshing...';
        }
        await this.loadLimits(true);
        if (btn) {
            btn.disabled = false;
            btn.style.opacity = 1;
            btn.innerHTML = `
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 4 23 10 17 10"></polyline><polyline points="1 20 1 14 7 14"></polyline><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path></svg>
                Refresh
            `;
        }
    },

    setFilter(value) {
        this.filterText = value.toLowerCase();
        setTimeout(() => this.render(), 10);
    },
 
    sortBy(key) {
        if (this.sortKey === key) {
            this.sortAsc = !this.sortAsc;
        } else {
            this.sortKey = key;
            this.sortAsc = key === 'namespace';
        }
        document.querySelectorAll('#limits-table thead th.sortable').forEach(th => {
            th.classList.toggle('active', th.dataset.sort === key);
        });
        this.updateSortIndicator();
        setTimeout(() => this.render(), 10);
    },

    updateSortIndicator() {
        const el = document.getElementById('limits-sort-indicator');
        if (el) {
            const labels = {
                namespace: 'Namespace',
                cpu_usage: 'CPU Usage',
                cpu_requests: 'CPU Requests',
                cpu_limits: 'CPU Limits',
                cpu_pct: 'CPU %',
                mem_usage: 'Memory Usage',
                mem_requests: 'Memory Requests',
                mem_limits: 'Memory Limits',
                mem_pct: 'Memory %',
                gpu: 'GPU',
                net_rx: 'Network RX',
                net_tx: 'Network TX',
                pods: 'Pods',
            };
            el.textContent = `${labels[this.sortKey] || this.sortKey} ${this.sortAsc ? '▲' : '▼'}`;
        }
    },

    getSortValue(app, key) {
        switch (key) {
            case 'namespace': return (app.namespace || '').toLowerCase();
            case 'cpu_usage': return app.cpu?.usage ?? 0;
            case 'cpu_requests': return app.cpu?.requests ?? 0;
            case 'cpu_limits': return app.cpu?.limits ?? 0;
            case 'cpu_pct': return app.cpu?.usage_percent ?? 0;
            case 'mem_usage': return app.memory?.usage_mb ?? 0;
            case 'mem_requests': return app.memory?.requests_mb ?? 0;
            case 'mem_limits': return app.memory?.limits_mb ?? 0;
            case 'mem_pct': return app.memory?.usage_percent ?? 0;
            case 'gpu': return app.gpu?.usage_percent ?? -1;
            case 'net_rx': return app.network?.rx_mbps ?? 0;
            case 'net_tx': return app.network?.tx_mbps ?? 0;
            case 'pods': return app.pod_count ?? 0;
            default: return 0;
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

        let filtered = apps;
        if (this.filterText) {
            filtered = apps.filter(a => (a.namespace || '').toLowerCase().includes(this.filterText));
        }

        if (filtered.length === 0) {
            tbody.innerHTML = `<tr><td colspan="13" style="text-align: center; padding: 2rem; color: var(--text-secondary);">${this.filterText ? 'No namespaces match filter' : 'No applications found'}</td></tr>`;
            return;
        }

        const sorted = [...filtered].sort((a, b) => {
            const va = this.getSortValue(a, this.sortKey);
            const vb = this.getSortValue(b, this.sortKey);
            if (typeof va === 'string') {
                return this.sortAsc ? va.localeCompare(vb) : vb.localeCompare(va);
            }
            return this.sortAsc ? va - vb : vb - va;
        });

        tbody.innerHTML = sorted.map(app => {
            const cpu = app.cpu || {};
            const mem = app.memory || {};
            const gpu = app.gpu || {};
            const net = app.network || {};

            const cpuPct = cpu.usage_percent || 0;
            const memPct = mem.usage_percent || 0;
            const cpuColor = cpuPct > 80 ? 'var(--neon-red, #ff3333)' : cpuPct > 50 ? 'var(--neon-orange, #ff9900)' : 'var(--neon-green)';
            const memColor = memPct > 80 ? 'var(--neon-red, #ff3333)' : memPct > 50 ? 'var(--neon-orange, #ff9900)' : 'var(--neon-green)';

            const gpuDisplay = gpu.available
                ? `<span style="color: var(--neon-magenta);">${(gpu.usage_percent || 0).toFixed(0)}%</span>`
                : '<span style="opacity: 0.4;">-</span>';

            return `
                <tr class="monitor-row" style="animation: fadeInRow 0.3s ease-out;">
                    <td style="font-weight: 600; text-align: left;">${this.escapeHtml(app.namespace || 'default')}</td>
                    <td class="td-num">${(cpu.usage || 0).toFixed(3)}</td>
                    <td class="td-num">${(cpu.requests || 0).toFixed(3)}</td>
                    <td class="td-num">${(cpu.limits || 0).toFixed(3)}</td>
                    <td class="td-center">
                        <div class="mini-bar" style="display: flex; align-items: center; gap: 0.5rem;">
                            <div style="flex: 1; height: 6px; background: rgba(255,255,255,0.1); border-radius: 3px;">
                                <div style="height: 100%; background: ${cpuColor}; border-radius: 3px; width: ${Math.min(cpuPct, 100)}%; transition: width 0.5s;"></div>
                            </div>
                            <span style="font-size: 0.75rem; color: ${cpuColor};">${cpuPct.toFixed(0)}%</span>
                        </div>
                    </td>
                    <td class="td-num">${(mem.usage_mb || 0).toFixed(0)} MB</td>
                    <td class="td-num">${(mem.requests_mb || 0).toFixed(0)} MB</td>
                    <td class="td-num">${(mem.limits_mb || 0).toFixed(0)} MB</td>
                    <td class="td-center">
                        <div class="mini-bar" style="display: flex; align-items: center; gap: 0.5rem;">
                            <div style="flex: 1; height: 6px; background: rgba(255,255,255,0.1); border-radius: 3px;">
                                <div style="height: 100%; background: ${memColor}; border-radius: 3px; width: ${Math.min(memPct, 100)}%; transition: width 0.5s;"></div>
                            </div>
                            <span style="font-size: 0.75rem; color: ${memColor};">${memPct.toFixed(0)}%</span>
                        </div>
                    </td>
                    <td class="td-center">${gpuDisplay}</td>
                    <td class="td-num">${this.formatBits(net.rx_mbps || 0)}</td>
                    <td class="td-num">${this.formatBits(net.tx_mbps || 0)}</td>
                    <td class="td-center">${app.pod_count || 0}</td>
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
