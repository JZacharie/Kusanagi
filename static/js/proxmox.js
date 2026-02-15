/**
 * Proxmox Dashboard Module with Caching
 * Note: Polling is handled by TabManager (tab-aware)
 */
const ProxmoxDashboard = {
    debug: false,
    initialized: false,
    cache: {
        data: null,
        timestamp: null,
        maxAge: 30000 // 30 seconds
    },

    init() {
        if (this.initialized) {
            this.log('⚠️ Already initialized, skipping...');
            return;
        }
        this.initialized = true;
        this.log('🔧 Proxmox Dashboard initialized (no internal polling)');
        // Ne pas fetch ici - TabManager s'en charge quand l'onglet est actif
    },

    // Alias pour TabManager
    loadData() {
        return this.fetchAndRender();
    },

    log(message, data = null) {
        if (this.debug) {
            console.log(`[PROXMOX DEBUG] ${message}`, data || '');
        }
    },

    async loadDataToCache() {
        try {
            const data = await this.fetchProxmoxData();
            this.cache.data = data;
            this.cache.timestamp = Date.now();
            this.log('📦 Proxmox data cached');
        } catch (error) {
            this.log('Failed to cache Proxmox data:', error);
            console.error('Failed to cache Proxmox data:', error);
        }
    },

    async fetchProxmoxData() {
        this.log('Fetching Proxmox data...');

        // Use apiFetch to get unwrapped data from the standard envelope
        const [vms, containers, nodes] = await Promise.all([
            api.get('/api/proxmox/vms').catch(() => []),
            api.get('/api/proxmox/containers').catch(() => []),
            api.get('/api/proxmox/nodes').catch(() => [])
        ]);

        this.log('Fetched data:', { vms: vms.length, containers: containers.length, nodes: nodes.length });

        return { vms, containers, nodes };
    },

    async activate() {
        this.log('🔄 Proxmox tab activated');
        // Le polling est géré par TabManager
        await this.fetchAndRender();
    },

    deactivate() {
        this.log('⏸️ Proxmox tab deactivated');
        // Le polling est géré par TabManager
    },

    async fetchAndRender() {
        try {
            const data = await this.fetchProxmoxData();
            this.cache.data = data;
            this.cache.timestamp = Date.now();
            this.renderAll(data);
        } catch (error) {
            this.log('Fetch error:', error);
            console.error('Failed to fetch Proxmox data:', error);
            const errorMsg = `Failed to load Proxmox data: ${error.message}`;
            const vmsContent = document.getElementById('proxmox-vms-content');
            const containersContent = document.getElementById('proxmox-containers-content');
            if (vmsContent) vmsContent.innerHTML = `<div class="error">${errorMsg}</div>`;
            if (containersContent) containersContent.innerHTML = `<div class="error">${errorMsg}</div>`;
        }
    },

    renderAll(data) {
        this.renderStats(data.vms, data.containers, data.nodes);
        this.renderVMs(data.vms);
        this.renderContainers(data.containers);
    },

    renderStats(vms, containers, nodes) {
        const nodeCount = Array.isArray(nodes) ? nodes.length : 0;
        const vmCount = Array.isArray(vms) ? vms.length : 0;
        const containerCount = Array.isArray(containers) ? containers.length : 0;

        this.log('Rendering stats:', { nodeCount, vmCount, containerCount });

        const nodeEl = document.getElementById('proxmox-nodes');
        const vmEl = document.getElementById('proxmox-vms');
        const containerEl = document.getElementById('proxmox-containers');

        if (nodeEl) nodeEl.textContent = nodeCount;
        if (vmEl) vmEl.textContent = vmCount;
        if (containerEl) containerEl.textContent = containerCount;
    },

    renderVMs(vms) {
        const container = document.getElementById('proxmox-vms-content');
        const countEl = document.getElementById('proxmox-vms-count');

        if (!Array.isArray(vms)) {
            this.log('VMs data is not an array:', vms);
            if (container) container.innerHTML = '<div class="error">Invalid VMs data received</div>';
            if (countEl) countEl.textContent = '0';
            return;
        }

        if (countEl) countEl.textContent = vms.length;
        this.log('Rendering VMs:', vms.length);

        if (!container) {
            this.log('VM container element not found');
            return;
        }

        if (vms.length === 0) {
            container.innerHTML = '<div class="no-issues">No VMs found</div>';
            return;
        }

        // Group VMs by server
        const groupedVMs = vms.reduce((acc, vm) => {
            const server = vm.server || 'Unknown';
            if (!acc[server]) acc[server] = [];
            acc[server].push(vm);
            return acc;
        }, {});

        let html = '';
        for (const [server, serverVMs] of Object.entries(groupedVMs)) {
            const serverHost = server.split('://')[1] || server;
            html += `
                <h4 class="server-group-title" style="margin-top: 1.5rem; margin-bottom: 0.5rem; color: var(--neon-blue); border-left: 3px solid var(--neon-blue); padding-left: 0.5rem;">
                    Server: ${serverHost}
                </h4>
                <table class="issues-table">
                    <thead>
                        <tr>
                            <th>VMID</th>
                            <th>Name</th>
                            <th>Node</th>
                            <th>Status</th>
                            <th>CPU</th>
                            <th>Memory</th>
                            <th>Uptime</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${serverVMs.map(vm => `
                            <tr>
                                <td>${vm.vmid || 'N/A'}</td>
                                <td><strong>${vm.name || 'VM-' + (vm.vmid || 'Unknown')}</strong></td>
                                <td>${vm.node || 'N/A'}</td>
                                <td><span class="status-badge ${vm.status === 'running' ? 'healthy' : 'unhealthy'}">${vm.status || 'unknown'}</span></td>
                                <td>${vm.cpu ? (vm.cpu * 100).toFixed(1) + '%' : 'N/A'}</td>
                                <td>${vm.mem && vm.maxmem ? this.formatBytes(vm.mem) + ' / ' + this.formatBytes(vm.maxmem) : 'N/A'}</td>
                                <td>${vm.uptime ? this.formatUptime(vm.uptime) : 'N/A'}</td>
                                <td>
                                    <div style="display: flex; gap: 5px;">
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.vmAction(${vm.vmid}, '${vm.node}', '${server}', 'start')" title="Start" ${vm.status === 'running' ? 'disabled' : ''}>▶️</button>
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.vmAction(${vm.vmid}, '${vm.node}', '${server}', 'stop')" title="Stop" ${vm.status !== 'running' ? 'disabled' : ''}>⏹️</button>
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.vmAction(${vm.vmid}, '${vm.node}', '${server}', 'reset')" title="Reset" style="border-color: #ffaa00; color: #ffaa00;">🔄</button>
                                    </div>
                                </td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;
        }

        container.innerHTML = html;
    },

    async vmAction(vmid, node, server, action) {
        try {
            const notify = typeof window.showNotification === 'function' ? window.showNotification : (m) => console.log(m);
            notify(`Sending ${action} order to VM ${vmid} on ${server}...`, 'info');

            const result = await api.post(`/api/proxmox/vm/${vmid}/node/${node}/status/${action}?server=${encodeURIComponent(server)}`);

            notify(result.message, 'success');
            setTimeout(() => this.fetchAndRender(), 2000);
        } catch (error) {
            console.error(`VM action error:`, error);
            const notify = typeof window.showNotification === 'function' ? window.showNotification : (m) => alert(m);
            notify(`Action failed: ${error.message}`, 'error');
        }
    },

    renderContainers(containers) {
        const container = document.getElementById('proxmox-containers-content');
        const countEl = document.getElementById('proxmox-containers-count');

        if (!Array.isArray(containers)) {
            this.log('Containers data is not an array:', containers);
            if (container) container.innerHTML = '<div class="error">Invalid containers data received</div>';
            if (countEl) countEl.textContent = '0';
            return;
        }

        if (countEl) countEl.textContent = containers.length;
        this.log('Rendering containers:', containers.length);

        if (!container) {
            this.log('Container element not found');
            return;
        }

        if (containers.length === 0) {
            container.innerHTML = '<div class="no-issues">No containers found</div>';
            return;
        }

        // Group containers by server
        const groupedCTs = containers.reduce((acc, ct) => {
            const server = ct.server || 'Unknown';
            if (!acc[server]) acc[server] = [];
            acc[server].push(ct);
            return acc;
        }, {});

        let html = '';
        for (const [server, serverCTs] of Object.entries(groupedCTs)) {
            const serverHost = server.split('://')[1] || server;
            html += `
                <h4 class="server-group-title" style="margin-top: 1.5rem; margin-bottom: 0.5rem; color: var(--neon-magenta); border-left: 3px solid var(--neon-magenta); padding-left: 0.5rem;">
                    Server: ${serverHost}
                </h4>
                <table class="issues-table">
                    <thead>
                        <tr>
                            <th>CTID</th>
                            <th>Name</th>
                            <th>Node</th>
                            <th>Status</th>
                            <th>CPU</th>
                            <th>Memory</th>
                            <th>Uptime</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${serverCTs.map(ct => `
                            <tr>
                                <td>${ct.vmid || 'N/A'}</td>
                                <td><strong>${ct.name || 'CT-' + (ct.vmid || 'Unknown')}</strong></td>
                                <td>${ct.node || 'N/A'}</td>
                                <td><span class="status-badge ${ct.status === 'running' ? 'healthy' : 'unhealthy'}">${ct.status || 'unknown'}</span></td>
                                <td>${ct.cpu ? (ct.cpu * 100).toFixed(1) + '%' : 'N/A'}</td>
                                <td>${ct.mem && ct.maxmem ? this.formatBytes(ct.mem) + ' / ' + this.formatBytes(ct.maxmem) : 'N/A'}</td>
                                <td>${ct.uptime ? this.formatUptime(ct.uptime) : 'N/A'}</td>
                                <td>
                                    <div style="display: flex; gap: 5px;">
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.ctAction(${ct.vmid}, '${ct.node}', '${server}', 'start')" title="Start" ${ct.status === 'running' ? 'disabled' : ''}>▶️</button>
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.ctAction(${ct.vmid}, '${ct.node}', '${server}', 'stop')" title="Stop" ${ct.status !== 'running' ? 'disabled' : ''}>⏹️</button>
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.ctAction(${ct.vmid}, '${ct.node}', '${server}', 'reset')" title="Reset" style="border-color: #ffaa00; color: #ffaa00;">🔄</button>
                                    </div>
                                </td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;
        }

        container.innerHTML = html;
    },

    async ctAction(vmid, node, server, action) {
        try {
            const notify = typeof window.showNotification === 'function' ? window.showNotification : (m) => console.log(m);
            notify(`Sending ${action} order to Container ${vmid} on ${server}...`, 'info');

            const result = await api.post(`/api/proxmox/ct/${vmid}/node/${node}/status/${action}?server=${encodeURIComponent(server)}`);

            notify(result.message, 'success');
            setTimeout(() => this.fetchAndRender(), 2000);
        } catch (error) {
            console.error(`Container action error:`, error);
            const notify = typeof window.showNotification === 'function' ? window.showNotification : (m) => alert(m);
            notify(`Action failed: ${error.message}`, 'error');
        }
    },

    formatBytes(bytes) {
        if (!bytes || bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
    },

    formatUptime(seconds) {
        if (!seconds && seconds !== 0) return 'N/A';
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = seconds % 60;

        if (days > 0) return `${days}j ${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}`;
        if (hours > 0) return `${hours}h ${minutes.toString().padStart(2, '0')}m`;
        return `${minutes}m ${secs.toString().padStart(2, '0')}s`;
    }
};

// Global export
window.ProxmoxDashboard = ProxmoxDashboard;
