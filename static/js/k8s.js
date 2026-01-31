/**
 * KUSANAGI Kubernetes Management Module
 * Handles ArgoCD, Nodes, Pods, Storage, and Events
 */

const K8sManager = {
    storageData: [],
    storagePage: 1,
    storagePerPage: 10,
    storageSortField: 'usage_percent',
    storageSortDir: 'desc',
    currentEventFilter: 'all',
    currentEventPage: 1,
    eventPerPage: 20,

    init() {
        this.fetchAll();
        // Set up intervals if needed (or move to main init)
        setInterval(() => this.fetchAll(), 30000);
        console.log('✅ K8s Manager initialized');
    },

    fetchAll() {
        this.fetchArgoStatus();
        this.fetchNodesStatus();
        this.fetchPodsStatus();
        this.fetchClusterOverview();
        this.fetchEvents(this.currentEventFilter, this.currentEventPage);
        this.fetchBackupsStatus();
        this.fetchStorageStatus();
        this.fetchServices();
        this.fetchIngress();
    },

    // === ARGOCD STATUS ===
    async fetchArgoStatus() {
        try {
            const response = await fetch('/api/argocd/status');
            const data = await response.json();
            if (data.error) {
                this.showArgoError(data.error);
                return;
            }
            this.updateArgoStats(data);
            this.updateArgoIssuesTable(data.apps_with_issues);
            this.updateArgoUpgradesTable(data.apps_with_upgrades);
        } catch (error) {
            this.showArgoError('Failed to connect to ArgoCD API');
        }
    },

    updateArgoStats(data) {
        const stats = {
            'stat-total': data.total,
            'stat-healthy': data.healthy,
            'stat-unhealthy': data.unhealthy,
            'stat-synced': data.synced,
            'stat-outofsync': data.out_of_sync,
            'stat-progressing': data.progressing,
            'stat-upgrades': data.upgrades_available || 0,
            'issues-count': data.apps_with_issues.length,
            'upgrades-count': (data.apps_with_upgrades || []).length
        };
        for (const [id, value] of Object.entries(stats)) {
            const el = document.getElementById(id);
            if (el) el.textContent = value;
        }
    },

    renderAppRow(app, showSync = false) {
        return `
            <tr>
                <td class="app-name">
                    <a href="${app.argocd_url}" target="_blank" title="Open in ArgoCD" style="color: var(--neon-green); text-decoration: none;">
                        ${app.name} 🔗
                    </a>
                </td>
                <td>${app.namespace || '-'}</td>
                <td><span class="status-badge ${app.health_status.toLowerCase()}">${app.health_status}</span></td>
                <td><span class="status-badge ${app.sync_status.toLowerCase().replace(' ', '')}">${app.sync_status}</span></td>
                <td class="revision-hash" title="${app.current_revision || 'Unknown'}">
                    <code>${app.current_revision ? app.current_revision.substring(0, 7) : '-'}</code>
                </td>
                <td class="error-duration">${app.error_duration || '-'}</td>
                ${showSync && app.can_sync ? `
                    <td>
                        <button class="sync-btn" onclick="K8sManager.syncApp(event, '${app.name}')" title="Sync ${app.name}">
                            ⟳ Sync
                        </button>
                    </td>
                ` : `<td class="error-message" title="${app.message || ''}">${app.message || '-'}</td>`}
            </tr>
        `;
    },

    updateArgoIssuesTable(issues) {
        const container = document.getElementById('issues-content');
        if (!container) return;
        if (!issues || issues.length === 0) {
            container.innerHTML = '<div class="no-issues">All applications are healthy!</div>';
            return;
        }
        container.innerHTML = `
            <table class="issues-table">
                <thead>
                    <tr>
                        <th>Application</th>
                        <th>Namespace</th>
                        <th>Health</th>
                        <th>Sync</th>
                        <th>Revision</th>
                        <th>Duration</th>
                        <th>Message</th>
                    </tr>
                </thead>
                <tbody>
                    ${issues.map(app => this.renderAppRow(app, false)).join('')}
                </tbody>
            </table>
        `;
    },

    updateArgoUpgradesTable(upgrades) {
        const container = document.getElementById('upgrades-content');
        if (!container) return;
        if (!upgrades || upgrades.length === 0) {
            container.innerHTML = '<div class="no-issues" style="color: var(--neon-cyan);">No upgrades available</div>';
            return;
        }
        container.innerHTML = `
            <table class="issues-table">
                <thead>
                    <tr>
                        <th>Application</th>
                        <th>Namespace</th>
                        <th>Health</th>
                        <th>Sync</th>
                        <th>Revision</th>
                        <th>Duration</th>
                        <th>Action</th>
                    </tr>
                </thead>
                <tbody>
                    ${upgrades.map(app => this.renderAppRow(app, true)).join('')}
                </tbody>
            </table>
        `;
    },

    async syncApp(event, appName) {
        const btn = event.currentTarget || event.target;
        const originalText = btn.textContent;
        btn.textContent = '⏳ Syncing...';
        btn.disabled = true;
        try {
            const response = await fetch('/api/argocd/sync', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ app_name: appName })
            });
            const data = await response.json();
            if (data.success) {
                btn.textContent = '✓ Done';
                btn.style.background = 'var(--neon-green)';
                setTimeout(() => this.fetchArgoStatus(), 2000);
            } else {
                btn.textContent = '✗ Failed';
                btn.style.background = '#ff4444';
            }
        } catch (error) {
            btn.textContent = '✗ Error';
            btn.style.background = '#ff4444';
        }
        setTimeout(() => {
            btn.textContent = originalText;
            btn.disabled = false;
            btn.style.background = '';
        }, 3000);
    },

    showArgoError(message) {
        const container = document.getElementById('issues-content');
        if (container) {
            container.innerHTML = `<div class="loading" style="color: #ff4444;">Error: ${message}</div>`;
        }
    },

    // === PODS MONITORING ===
    async fetchPodsStatus() {
        console.log('Fetching pods status...');
        try {
            const response = await fetch('/api/pods/status');
            const data = await response.json();
            if (data.error) {
                const el = document.getElementById('pods-content');
                if (el) el.innerHTML = `<div class="loading" style="color: #ff4444;">Error: ${data.error}</div>`;
                return;
            }
            const stats = {
                'pods-total': data.total_pods,
                'pods-running': data.running_pods,
                'pods-pending': data.pending_pods,
                'pods-error': data.error_pods,
                'pods-error-count': data.pods_in_error.length
            };
            for (const [id, value] of Object.entries(stats)) {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }
            const podsColumns = [
                { key: 'name', label: 'Pod Name' },
                { key: 'namespace', label: 'Namespace' },
                { key: 'status', label: 'Status' },
                { key: 'reason', label: 'Reason' },
                { key: 'restart_count', label: 'Restarts' },
                { key: 'age', label: 'Age' },
                { key: 'node', label: 'Node' },
                { key: 'actions', label: 'Actions' }
            ];
            if (window.TableManager) {
                TableManager.init('pods', data.pods_in_error, (pods) => this.renderPodsTableContent(pods), podsColumns);
                this.renderPodsTable(data.pods_in_error);
            }
            console.log('Pods status fetched successfully');
        } catch (error) {
            console.error('Failed to fetch pods status:', error);
            const el = document.getElementById('pods-content');
            if (el) el.innerHTML = `<div class="loading" style="color: #ff4444;">Error: Failed to fetch pods status. Check if Kusanagi agent is reachable.</div>`;
        }
    },

    renderPodsTable(pods) {
        const container = document.getElementById('pods-content');
        if (!container) return;
        if (!pods || pods.length === 0) {
            container.innerHTML = '<div class="no-issues" style="color: var(--neon-green);">✓ No pods in error state!</div>';
            return;
        }
        const podsColumns = [
            { key: 'name', label: 'Pod Name' },
            { key: 'namespace', label: 'Namespace' },
            { key: 'status', label: 'Status' },
            { key: 'reason', label: 'Reason' },
            { key: 'restart_count', label: 'Restarts' },
            { key: 'age', label: 'Age' },
            { key: 'node', label: 'Node' },
            { key: 'actions', label: 'Actions' }
        ];
        const searchHtml = window.TableManager ? TableManager.createSearchInput('pods', 'Search pods...') : '';
        const headerHtml = window.TableManager ? TableManager.createSortableHeader('pods', podsColumns) : '';
        container.innerHTML = `
            ${searchHtml}
            <table class="issues-table" id="pods-table">
                <thead><tr>${headerHtml}</tr></thead>
                <tbody id="pods-table-body">${this.renderPodsRows(pods)}</tbody>
            </table>
        `;
    },

    renderPodsTableContent(pods) {
        const tbody = document.getElementById('pods-table-body');
        if (tbody) tbody.innerHTML = this.renderPodsRows(pods);
    },

    renderPodsRows(pods) {
        if (!pods || pods.length === 0) {
            return '<tr><td colspan="8" style="text-align: center; color: var(--neon-green);">✓ No matching pods</td></tr>';
        }
        return pods.map(pod => `
            <tr>
                <td class="app-name" title="${pod.message || ''}">${pod.name}</td>
                <td>${pod.namespace}</td>
                <td><span class="status-badge ${this.getK8sStatusClass(pod.status)}">${pod.status}</span></td>
                <td class="error-message" title="${pod.message || ''}">${pod.reason || '-'}</td>
                <td style="color: ${pod.restart_count > 5 ? '#ff4444' : 'inherit'}; font-weight: ${pod.restart_count > 5 ? 'bold' : 'normal'};">${pod.restart_count}</td>
                <td>${pod.age}</td>
                <td>${pod.node || '-'}</td>
                <td>
                    <div style="display: flex; gap: 5px;">
                        <button class="action-btn" onclick="LogsManager.openModal('${pod.namespace}', '${pod.name}')" title="View Pod Logs" style="background: var(--neon-blue); color: #fff;">📄 Logs</button>
                        <button class="delete-btn" onclick="K8sManager.forceDeletePod('${pod.namespace}', '${pod.name}')" title="Force delete this pod">🗑️ Delete</button>
                    </div>
                </td>
            </tr>
        `).join('');
    },

    getK8sStatusClass(status) {
        switch (status.toLowerCase()) {
            case 'running':
            case 'succeeded': return 'healthy';
            case 'pending': return 'progressing';
            case 'failed': return 'unhealthy';
            default: return 'unknown';
        }
    },

    async forceDeletePod(namespace, podName) {
        const confirmed = confirm(`⚠️ Force Delete Pod\n\nAre you sure you want to force delete:\n\n${namespace}/${podName}\n\nThis action cannot be undone!`);
        if (!confirmed) return;
        try {
            const response = await fetch('/api/pods/force-delete', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ namespace, pod_name: podName })
            });
            const data = await response.json();
            if (data.success) {
                if (window.showNotification) showNotification({ title: 'Pod Deleted', message: `Successfully force deleted ${podName}`, severity: 'success' });
                setTimeout(() => this.fetchPodsStatus(), 1000);
            } else {
                if (window.showNotification) showNotification({ title: 'Delete Failed', message: data.message || 'Unknown error', severity: 'error' });
            }
        } catch (error) {
            console.error('Failed to delete pod:', error);
        }
    },

    // === NODES MONITORING ===
    async fetchNodesStatus() {
        try {
            const response = await fetch('/api/nodes/status');
            const data = await response.json();
            if (data.error) {
                const el = document.getElementById('nodes-container');
                if (el) el.innerHTML = `<div class="loading" style="color: #ff4444;">Error: ${data.error}</div>`;
                return;
            }
            const stats = { 'node-total': data.total_nodes, 'node-ready': data.ready_nodes, 'node-notready': data.not_ready_nodes };
            for (const [id, value] of Object.entries(stats)) {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }
            this.renderNodes(data.nodes);
        } catch (error) {
            console.error('Nodes error:', error);
            const el = document.getElementById('nodes-container');
            if (el) el.innerHTML = `<div class="loading" style="color: #ff4444;">Error: Failed to fetch nodes status.</div>`;
            const diagnosticTool = document.getElementById('nodes-diagnostic-tool');
            if (diagnosticTool) diagnosticTool.style.display = 'block';
        }
    },

    renderNodes(nodes) {
        const container = document.getElementById('nodes-container');
        if (!container) return;
        if (!nodes || nodes.length === 0) {
            container.innerHTML = '<div class="no-issues">No nodes found</div>';
            return;
        }
        nodes.sort((a, b) => a.name.localeCompare(b.name));
        container.innerHTML = nodes.map(node => {
            const archClass = node.architecture === 'arm64' ? 'arch-arm' : 'arch-amd';
            const statusClass = node.status === 'Ready' ? 'node-ready' : 'node-notready';
            const nodeType = node.labels?.type || node.labels?.['node.kubernetes.io/instance-type'] || 'node';
            const podPercent = Math.round((node.pod_count / parseInt(node.pod_capacity)) * 100);
            const podBarClass = podPercent > 80 ? 'bar-danger' : podPercent > 60 ? 'bar-warning' : 'bar-ok';
            const cpuPercent = node.cpu_usage_percent || 0;
            const cpuBarClass = cpuPercent > 80 ? 'bar-danger' : cpuPercent > 60 ? 'bar-warning' : 'bar-ok';
            const cpuDisplay = node.cpu_usage_percent != null ? `${node.cpu_usage_percent.toFixed(1)}%` : 'N/A';
            const memPercent = node.memory_usage_percent || 0;
            const memBarClass = memPercent > 80 ? 'bar-danger' : memPercent > 60 ? 'bar-warning' : 'bar-ok';
            const memDisplay = node.memory_usage_percent != null ? `${node.memory_usage_percent.toFixed(1)}%` : 'N/A';
            return `
                <div class="node-card ${statusClass} ${node.pods_in_error > 0 ? 'has-errors' : ''}">
                    <div class="node-header">
                        <div class="node-title"><span class="node-name">${node.name}</span><span class="node-type">${nodeType}</span></div>
                        <span class="arch-badge ${archClass}">${node.architecture}</span>
                    </div>
                    <div class="node-resources">
                        <div class="resource-row"><span class="resource-icon">⚡</span><span class="resource-label">CPU</span>
                            <div class="pod-bar-container" title="Capacity: ${node.cpu_capacity} cores"><div class="pod-bar ${cpuBarClass}" style="width: ${Math.min(cpuPercent, 100)}%"></div><span class="pod-text">${cpuDisplay}</span></div>
                        </div>
                        <div class="resource-row"><span class="resource-icon">🧠</span><span class="resource-label">RAM</span>
                            <div class="pod-bar-container" title="Allocatable: ${node.memory_allocatable}"><div class="pod-bar ${memBarClass}" style="width: ${Math.min(memPercent, 100)}%"></div><span class="pod-text">${memDisplay}</span></div>
                        </div>
                        <div class="resource-row pods-row"><span class="resource-icon">📦</span><span class="resource-label">Pods</span>
                            <div class="pod-bar-container"><div class="pod-bar ${podBarClass}" style="width: ${podPercent}%"></div><span class="pod-text">${node.pod_count} / ${node.pod_capacity}</span></div>
                        </div>
                    </div>
                </div>
            `;
        }).join('');
    },

    async runNodesDiagnostic() {
        const btn = document.getElementById('btn-run-diagnostic');
        const resultDiv = document.getElementById('nodes-diagnostic-result');
        if (!btn || !resultDiv) return;
        btn.disabled = true;
        btn.textContent = '⏳ Analyzing...';
        resultDiv.style.display = 'block';
        resultDiv.innerHTML = 'Testing components...\n';
        try {
            const response = await fetch('/api/debug/nodes');
            const data = await response.json();
            let report = '🔍 DIAGNOSTIC REPORT\n====================\n\n';
            report += `[K8s Nodes] ${data.k8s_nodes_ok ? '✅ OK' : '❌ FAIL'}\n[K8s Pods]  ${data.k8s_pods_ok ? '✅ OK' : '❌ FAIL'}\n[Prometheus] ${data.prometheus_ok ? '✅ OK' : '❌ FAIL'}\n`;
            resultDiv.innerHTML = report;
        } catch (e) {
            resultDiv.innerHTML += `\n❌ Critical error: ${e.message}`;
        } finally {
            btn.disabled = false;
            btn.textContent = 'Run Deep Diagnostic';
        }
    },

    // === CLUSTER OVERVIEW ===
    async fetchClusterOverview() {
        try {
            const response = await fetch('/api/cluster/overview');
            const data = await response.json();
            if (!data.error) {
                const stats = {
                    'ns-count': data.namespace_count || 0,
                    'pvc-count': data.pvc_count || 0,
                    'pvc-capacity': data.pvc_total_capacity || '-',
                    'pvc-total-count': data.pvc_count || 0,
                    'pvc-bound-count': (data.pvcs || []).filter(p => p.status === 'Bound').length,
                    'pvc-pending-count': (data.pvcs || []).filter(p => p.status !== 'Bound').length,
                    'pvc-total-storage': data.pvc_total_capacity || '-',
                    'pvc-table-count': (data.pvcs || []).length
                };
                for (const [id, value] of Object.entries(stats)) {
                    const el = document.getElementById(id);
                    if (el) el.textContent = value;
                }
            }
        } catch (error) {
            console.error('Failed to fetch cluster overview:', error);
            // Non-critical background update, no need to show error in main UI unless pods/nodes are failing
        }
    },

    // === STORAGE ===
    async fetchStorageStatus() {
        try {
            const response = await fetch('/api/storage');
            const data = await response.json();
            if (!data.error) {
                this.storageData = data.pvcs || [];
                const countEl = document.getElementById('pvc-table-count');
                if (countEl) countEl.textContent = this.storageData.length;
                this.renderStorageTable();
            }
        } catch (error) {
            console.error('Failed to fetch storage status:', error);
            const container = document.getElementById('pvc-content');
            if (container) container.innerHTML = `<div class="loading" style="color: #ff4444;">Error: Failed to fetch storage data.</div>`;
        }
    },

    renderStorageTable() {
        const container = document.getElementById('pvc-content');
        if (!container) return;
        if (!this.storageData || this.storageData.length === 0) {
            container.innerHTML = '<div class="no-issues" style="color: var(--neon-cyan);">No PVCs found</div>';
            return;
        }
        const sortedData = [...this.storageData].sort((a, b) => {
            let valA = a[this.storageSortField] ?? -1;
            let valB = b[this.storageSortField] ?? -1;
            if (typeof valA === 'string') { valA = valA.toLowerCase(); valB = valB.toLowerCase(); }
            if (valA < valB) return this.storageSortDir === 'asc' ? -1 : 1;
            if (valA > valB) return this.storageSortDir === 'asc' ? 1 : -1;
            return 0;
        });
        const totalPages = Math.ceil(sortedData.length / this.storagePerPage);
        if (this.storagePage > totalPages) this.storagePage = totalPages || 1;
        const start = (this.storagePage - 1) * this.storagePerPage;
        const pageData = sortedData.slice(start, start + this.storagePerPage);
        container.innerHTML = `
            <table class="issues-table">
                <thead><tr>
                    <th onclick="K8sManager.sortStorage('name')" class="sortable">Name ${this.getSortArrow('name')}</th>
                    <th onclick="K8sManager.sortStorage('namespace')" class="sortable">Namespace ${this.getSortArrow('namespace')}</th>
                    <th onclick="K8sManager.sortStorage('capacity_bytes')" class="sortable">Capacity ${this.getSortArrow('capacity_bytes')}</th>
                    <th onclick="K8sManager.sortStorage('usage_percent')" class="sortable">Usage ${this.getSortArrow('usage_percent')}</th>
                    <th onclick="K8sManager.sortStorage('status')" class="sortable">Status ${this.getSortArrow('status')}</th>
                    <th>Class</th>
                </tr></thead>
                <tbody>${pageData.map(pvc => {
            const percent = pvc.usage_percent || 0;
            const barClass = percent > 90 ? 'bar-danger' : percent > 75 ? 'bar-warning' : 'bar-ok';
            return `<tr>
                        <td class="app-name">${pvc.name}</td><td>${pvc.namespace}</td><td>${pvc.capacity}</td>
                        <td><div class="usage-cell"><div class="pod-bar-container" title="${this.formatBytes(pvc.used_bytes) || '?'} / ${pvc.capacity}"><div class="pod-bar ${barClass}" style="width: ${Math.min(percent, 100)}%"></div></div><span class="usage-text">${percent.toFixed(1)}%</span></div></td>
                        <td><span class="status-badge ${pvc.status.toLowerCase()}">${pvc.status}</span></td><td class="storage-class">${pvc.storage_class || '-'}</td>
                    </tr>`;
        }).join('')}</tbody>
            </table>
            <div class="pagination-controls">
                <button ${this.storagePage === 1 ? 'disabled' : ''} onclick="K8sManager.changeStoragePage(-1)" class="page-btn">◀</button>
                <span class="page-info">Page ${this.storagePage} of ${totalPages}</span>
                <button ${this.storagePage === totalPages ? 'disabled' : ''} onclick="K8sManager.changeStoragePage(1)" class="page-btn">▶</button>
            </div>
        `;
    },

    formatBytes(bytes) {
        if (!bytes && bytes !== 0) return null;
        if (bytes === 0) return '0 B';
        const k = 1024; const sizes = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
    },

    sortStorage(field) {
        if (this.storageSortField === field) this.storageSortDir = this.storageSortDir === 'asc' ? 'desc' : 'asc';
        else { this.storageSortField = field; this.storageSortDir = 'desc'; }
        this.renderStorageTable();
    },

    getSortArrow(field) {
        if (this.storageSortField !== field) return '';
        return this.storageSortDir === 'asc' ? '▲' : '▼';
    },

    changeStoragePage(delta) { this.storagePage += delta; this.renderStorageTable(); },

    // === EVENTS ===
    async fetchEvents(typeFilter = 'all', page = 1) {
        try {
            this.currentEventFilter = typeFilter; this.currentEventPage = page;
            const url = new URL('/api/events', window.location.origin);
            if (typeFilter !== 'all') url.searchParams.append('event_type', typeFilter);
            url.searchParams.append('page', page); url.searchParams.append('per_page', this.eventPerPage);
            const response = await fetch(url.toString());
            const data = await response.json();
            if (!data.error) {
                const stats = { 'events-total': data.total_events, 'events-warnings': data.warning_count, 'events-normal': data.normal_count, 'events-table-count': data.total_events };
                for (const [id, value] of Object.entries(stats)) { const el = document.getElementById(id); if (el) el.textContent = value; }
                this.renderEventsTable(data);
            }
        } catch (error) {
            console.error('Failed to fetch events:', error);
            const container = document.getElementById('events-content');
            if (container) container.innerHTML = `<div class="loading" style="color: #ff4444;">Error: Failed to fetch events.</div>`;
        }
    },

    filterEvents(type) {
        document.getElementById('btn-show-all')?.classList.toggle('active', type === 'all');
        document.getElementById('btn-show-warnings')?.classList.toggle('active', type === 'Warning');
        this.fetchEvents(type, 1);
    },

    changeEventsPage(delta) { const newPage = this.currentEventPage + delta; if (newPage >= 1) this.fetchEvents(this.currentEventFilter, newPage); },

    renderEventsTable(data) {
        const container = document.getElementById('events-content');
        if (!container || !data.events || data.events.length === 0) return;
        container.innerHTML = `
            <table class="issues-table">
                <thead><tr><th>Type</th><th>Object</th><th>Reason</th><th>Message</th><th>Age</th><th>Count</th></tr></thead>
                <tbody>${data.events.map(evt => `
                    <tr class="${evt.event_type.toLowerCase()}">
                        <td><span class="event-type ${evt.event_type.toLowerCase()}">${evt.event_type}</span></td>
                        <td class="event-object">${evt.involved_object_kind}/${evt.involved_object_name.substring(0, 30)}</td>
                        <td class="event-reason">${evt.reason}</td>
                        <td class="event-message" title="${evt.message}">${evt.message.substring(0, 60)}</td>
                        <td class="event-age">${evt.age || '-'}</td><td class="event-count">${evt.count}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
            <div class="pagination-controls">
                <button class="cyber-btn" onclick="K8sManager.changeEventsPage(-1)">PREV</button>
                <span>Page ${data.page} of ${data.total_pages}</span>
                <button class="cyber-btn" onclick="K8sManager.changeEventsPage(1)">NEXT</button>
            </div>
        `;
    },

    // === BACKUPS ===
    async fetchBackupsStatus() {
        try {
            const response = await fetch('/api/backups');
            const data = await response.json();
            if (!data.error) {
                const stats = { 'backup-cronjobs': data.total_cronjobs, 'backup-active': data.active_jobs, 'backup-succeeded': data.succeeded_jobs, 'backup-failed': data.failed_jobs, 'backups-count': data.total_cronjobs };
                for (const [id, value] of Object.entries(stats)) { const el = document.getElementById(id); if (el) el.textContent = value; }
                this.renderBackupsTable(data.cronjobs || []);
            }
        } catch (error) {
            console.error('Failed to fetch backups:', error);
            const container = document.getElementById('backups-content');
            if (container) container.innerHTML = `<div class="loading" style="color: #ff4444;">Error: Failed to fetch backup status.</div>`;
        }
    },

    renderBackupsTable(cronjobs) {
        const container = document.getElementById('backups-content');
        if (!container || !cronjobs || cronjobs.length === 0) return;
        container.innerHTML = `
            <table class="issues-table">
                <thead><tr><th>CronJob</th><th>Namespace</th><th>Schedule</th><th>Last Run</th><th>Status</th><th>Recent Jobs</th></tr></thead>
                <tbody>${cronjobs.map(cj => {
            const statusClass = cj.suspend ? 'suspended' : (cj.active_jobs > 0 ? 'running' : 'healthy');
            const statusText = cj.suspend ? 'Suspended' : (cj.active_jobs > 0 ? 'Running' : 'Idle');
            return `<tr>
                        <td class="app-name">${cj.name}</td><td>${cj.namespace}</td><td><code>${cj.schedule}</code></td>
                        <td>${cj.last_schedule_age || '-'}</td><td><span class="status-badge ${statusClass}">${statusText}</span></td>
                        <td>${(cj.recent_jobs || []).length} jobs</td>
                    </tr>`;
        }).join('')}</tbody>
            </table>
        `;
    },

    // === SERVICES & INGRESS ===
    async fetchServices() {
        try {
            const response = await fetch('/api/services');
            const data = await response.json();
            const countEl = document.getElementById('services-count');
            if (countEl) countEl.textContent = data.length;
            if (window.TableManager) {
                TableManager.init('services', data, (svc) => this.renderServicesRows(svc), [
                    { key: 'name', label: 'Name' }, { key: 'namespace', label: 'Namespace' }, { key: 'type_', label: 'Type' },
                    { key: 'cluster_ip', label: 'Cluster IP' }, { key: 'external_ip', label: 'External IP' }, { key: 'ports', label: 'Ports' }, { key: 'age', label: 'Age' }
                ]);
                this.renderServicesStructure();
            }
        } catch (error) { console.error('Error fetching services:', error); }
    },

    renderServicesStructure() {
        const container = document.getElementById('services-content');
        if (!container) return;
        const searchHtml = window.TableManager ? TableManager.createSearchInput('services', 'Search services...') : '';
        container.innerHTML = `
            <div class="table-controls">${searchHtml}</div>
            <table class="data-table">
                <thead><tr>${TableManager.createSortableHeader('services', [
            { key: 'name', label: 'Name' }, { key: 'namespace', label: 'Namespace' }, { key: 'type_', label: 'Type' },
            { key: 'cluster_ip', label: 'Cluster IP' }, { key: 'external_ip', label: 'External IP' }, { key: 'ports', label: 'Ports' }, { key: 'age', label: 'Age' }
        ])}</tr></thead>
                <tbody id="services-table-body"></tbody>
            </table>
        `;
        this.renderServicesRows(TableManager.tables['services'].filtered);
    },

    renderServicesRows(services) {
        const tbody = document.getElementById('services-table-body');
        if (tbody) tbody.innerHTML = services.map(svc => `
            <tr><td class="app-name">${svc.name}</td><td>${svc.namespace}</td><td>${svc.type_}</td><td>${svc.cluster_ip}</td><td>${svc.external_ip || '-'}</td><td>${svc.ports}</td><td>${svc.age}</td></tr>
        `).join('');
    },

    async fetchIngress() {
        try {
            const response = await fetch('/api/ingress');
            const data = await response.json();
            const countEl = document.getElementById('ingress-count');
            if (countEl) countEl.textContent = data.length;
            this.renderIngressTable(data);
        } catch (error) { console.error('Error fetching ingress:', error); }
    },

    renderIngressTable(ingresses) {
        const container = document.getElementById('ingress-content');
        if (!container || ingresses.length === 0) return;
        container.innerHTML = `
            <table class="data-table">
                <thead><tr><th>Name</th><th>Namespace</th><th>Rules</th><th>Age</th></tr></thead>
                <tbody>${ingresses.map(ing => `
                    <tr><td class="app-name">${ing.name}</td><td>${ing.namespace}</td><td>${ing.rules.join(', ')}</td><td>${ing.age}</td></tr>
                `).join('')}</tbody>
            </table>
        `;
    }
};

window.K8sManager = K8sManager;
