// Proxmox Dashboard Module
const ProxmoxDashboard = {
    refreshInterval: null,
    debug: true, // Enable debug mode

    init() {
        this.log('Initializing Proxmox Dashboard...');
        this.fetchAndRender();
        if (this.refreshInterval) clearInterval(this.refreshInterval);
        this.refreshInterval = setInterval(() => this.fetchAndRender(), 30000);
        this.log('✅ Proxmox Dashboard initialized');
    },

    log(message, data = null) {
        if (this.debug) {
            console.log(`[PROXMOX DEBUG] ${message}`, data || '');
        }
    },

    async fetchAndRender() {
        try {
            this.log('Fetching Proxmox data...');
            
            // Fetch all data in parallel
            const [vmsResponse, containersResponse, nodesResponse] = await Promise.all([
                fetch('/api/proxmox/vms'),
                fetch('/api/proxmox/containers'), 
                fetch('/api/proxmox/nodes')
            ]);

            this.log('Response status:', {
                vms: vmsResponse.status,
                containers: containersResponse.status,
                nodes: nodesResponse.status
            });

            const [vms, containers, nodes] = await Promise.all([
                vmsResponse.json(),
                containersResponse.json(),
                nodesResponse.json()
            ]);

            this.log('Fetched data:', { vms: vms.length, containers: containers.length, nodes: nodes.length });

            this.renderStats(vms, containers, nodes);
            this.renderVMs(vms);
            this.renderContainers(containers);
        } catch (error) {
            this.log('Fetch error:', error);
            console.error('Failed to fetch Proxmox data:', error);
            const errorMsg = `Failed to load Proxmox data: ${error.message}`;
            document.getElementById('proxmox-vms-content').innerHTML = `<div class="error">${errorMsg}</div>`;
            document.getElementById('proxmox-containers-content').innerHTML = `<div class="error">${errorMsg}</div>`;
        }
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

            const response = await fetch(`/api/proxmox/vm/${vmid}/node/${node}/status/${action}?server=${encodeURIComponent(server)}`, {
                method: 'POST'
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || `Failed to ${action} VM`);
            }

            const result = await response.json();
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

            const response = await fetch(`/api/proxmox/ct/${vmid}/node/${node}/status/${action}?server=${encodeURIComponent(server)}`, {
                method: 'POST'
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.error || `Failed to ${action} Container`);
            }

            const result = await response.json();
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
        if (!seconds || seconds === 0) return '0s';
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);

        if (days > 0) return `${days}d ${hours}h`;
        if (hours > 0) return `${hours}h ${minutes}m`;
        return `${minutes}m`;
    }
};

// Auto-load when tab is switched
window.ProxmoxDashboard = ProxmoxDashboard;
