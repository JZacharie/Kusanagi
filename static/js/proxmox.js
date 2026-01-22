// Proxmox Dashboard Module
const ProxmoxDashboard = {
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

        const table = `
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
                    </tr>
                </thead>
                <tbody>
                    ${vms.map(vm => `
                        <tr>
                            <td>${vm.vmid}</td>
                            <td><strong>${vm.name || 'VM-' + vm.vmid}</strong></td>
                            <td>${vm.node}</td>
                            <td><span class="status-badge ${vm.status === 'running' ? 'healthy' : 'unhealthy'}">${vm.status}</span></td>
                            <td>${(vm.cpu * 100).toFixed(1)}%</td>
                            <td>${this.formatBytes(vm.mem)} / ${this.formatBytes(vm.maxmem)}</td>
                            <td>${this.formatBytes(vm.disk)} / ${this.formatBytes(vm.maxdisk)}</td>
                            <td>${this.formatUptime(vm.uptime)}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;

        container.innerHTML = table;
    },

    renderContainers(containers) {
        const container = document.getElementById('proxmox-containers-content');
        document.getElementById('proxmox-containers-count').textContent = containers.length;

        if (!containers || containers.length === 0) {
            container.innerHTML = '<div class="no-issues">No containers found</div>';
            return;
        }

        const table = `
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
                    </tr>
                </thead>
                <tbody>
                    ${containers.map(ct => `
                        <tr>
                            <td>${ct.vmid}</td>
                            <td><strong>${ct.name || 'CT-' + ct.vmid}</strong></td>
                            <td>${ct.node}</td>
                            <td><span class="status-badge ${ct.status === 'running' ? 'healthy' : 'unhealthy'}">${ct.status}</span></td>
                            <td>${(ct.cpu * 100).toFixed(1)}%</td>
                            <td>${this.formatBytes(ct.mem)} / ${this.formatBytes(ct.maxmem)}</td>
                            <td>${this.formatBytes(ct.disk)} / ${this.formatBytes(ct.maxdisk)}</td>
                            <td>${this.formatUptime(ct.uptime)}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;

        container.innerHTML = table;
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
