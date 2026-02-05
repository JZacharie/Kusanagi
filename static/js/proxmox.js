// Proxmox Dashboard Module
const ProxmoxDashboard = {
    refreshInterval: null,

    init() {
        this.fetchAndRender();
        if (this.refreshInterval) clearInterval(this.refreshInterval);
        this.refreshInterval = setInterval(() => this.fetchAndRender(), 30000);
        console.log('✅ Proxmox Dashboard initialized');
    },

    async fetchAndRender() {
        try {
            // Fetch all data in parallel
            const [vms, containers, nodes] = await Promise.all([
                fetch('/api/proxmox/vms').then(r => r.json()),
                fetch('/api/proxmox/containers').then(r => r.json()),
                fetch('/api/proxmox/nodes').then(r => r.json())
            ]);

            this.renderStats(vms, containers, nodes);
            this.renderVMs(vms);
            this.renderContainers(containers);
        } catch (error) {
            console.error('Failed to fetch Proxmox data:', error);
            document.getElementById('proxmox-vms-content').innerHTML =
                `<div class="error">Failed to load Proxmox data: ${error.message}</div>`;
        }
    },

    renderStats(vms, containers, nodes) {
        document.getElementById('proxmox-nodes').textContent = nodes.length || '0';
        document.getElementById('proxmox-vms').textContent = vms.length || '0';
        document.getElementById('proxmox-containers').textContent = containers.length || '0';
    },

    renderVMs(vms) {
        const container = document.getElementById('proxmox-vms-content');
        document.getElementById('proxmox-vms-count').textContent = vms.length;

        if (!vms || vms.length === 0) {
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
                            <th>Disk</th>
                            <th>Uptime</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${serverVMs.map(vm => `
                            <tr>
                                <td>${vm.vmid}</td>
                                <td><strong>${vm.name || 'VM-' + vm.vmid}</strong></td>
                                <td>${vm.node}</td>
                                <td><span class="status-badge ${vm.status === 'running' ? 'healthy' : 'unhealthy'}">${vm.status}</span></td>
                                <td>${(vm.cpu * 100).toFixed(1)}%</td>
                                <td>${this.formatBytes(vm.mem)} / ${this.formatBytes(vm.maxmem)}</td>
                                <td>${this.formatBytes(vm.disk)} / ${this.formatBytes(vm.maxdisk)}</td>
                                <td>${this.formatUptime(vm.uptime)}</td>
                                <td>
                                    <div class="vm-actions" style="display: flex; gap: 0.5rem;">
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.vmAction(${vm.vmid}, '${vm.node}', '${vm.server}', 'start')" ${vm.status === 'running' ? 'disabled' : ''} title="Start VM" style="padding: 2px 8px; font-size: 0.8rem; border-color: var(--neon-green); color: var(--neon-green);">▶</button>
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.vmAction(${vm.vmid}, '${vm.node}', '${vm.server}', 'shutdown')" ${vm.status !== 'running' ? 'disabled' : ''} title="Shutdown VM" style="padding: 2px 8px; font-size: 0.8rem; border-color: var(--neon-yellow); color: var(--neon-yellow);">⏹</button>
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.vmAction(${vm.vmid}, '${vm.node}', '${vm.server}', 'stop')" ${vm.status !== 'running' ? 'disabled' : ''} title="Force Stop VM" style="padding: 2px 8px; font-size: 0.8rem; border-color: var(--neon-magenta); color: var(--neon-magenta);">⚡</button>
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
        document.getElementById('proxmox-containers-count').textContent = containers.length;

        if (!containers || containers.length === 0) {
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
                            <th>Disk</th>
                            <th>Uptime</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${serverCTs.map(ct => `
                            <tr>
                                <td>${ct.vmid}</td>
                                <td><strong>${ct.name || 'CT-' + ct.vmid}</strong></td>
                                <td>${ct.node}</td>
                                <td><span class="status-badge ${ct.status === 'running' ? 'healthy' : 'unhealthy'}">${ct.status}</span></td>
                                <td>${(ct.cpu * 100).toFixed(1)}%</td>
                                <td>${this.formatBytes(ct.mem)} / ${this.formatBytes(ct.maxmem)}</td>
                                <td>${this.formatBytes(ct.disk)} / ${this.formatBytes(ct.maxdisk)}</td>
                                <td>${this.formatUptime(ct.uptime)}</td>
                                <td>
                                    <div class="ct-actions" style="display: flex; gap: 0.5rem;">
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.ctAction(${ct.vmid}, '${ct.node}', '${ct.server}', 'start')" ${ct.status === 'running' ? 'disabled' : ''} title="Start Container" style="padding: 2px 8px; font-size: 0.8rem; border-color: var(--neon-green); color: var(--neon-green);">▶</button>
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.ctAction(${ct.vmid}, '${ct.node}', '${ct.server}', 'shutdown')" ${ct.status !== 'running' ? 'disabled' : ''} title="Shutdown Container" style="padding: 2px 8px; font-size: 0.8rem; border-color: var(--neon-yellow); color: var(--neon-yellow);">⏹</button>
                                        <button class="cyber-btn sm" onclick="ProxmoxDashboard.ctAction(${ct.vmid}, '${ct.node}', '${ct.server}', 'stop')" ${ct.status !== 'running' ? 'disabled' : ''} title="Force Stop Container" style="padding: 2px 8px; font-size: 0.8rem; border-color: var(--neon-magenta); color: var(--neon-magenta);">⚡</button>
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
