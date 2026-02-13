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
    lastServicesFetch: 0,
    lastIngressFetch: 0,
    // 3 minutes TTL for services and ingress (in milliseconds)
    SERVICES_INGRESS_TTL: 180000,

    init() {
        this.fetchAll();
        // Set up intervals if needed (or move to main init)
        setInterval(() => this.fetchAll(), 30000);

        // Refresh on page focus (for services and ingress)
        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
                const now = Date.now();

                // Refresh services if on services tab and TTL expired
                if (activeTab === 'services' && now - this.lastServicesFetch >= this.SERVICES_INGRESS_TTL) {
                    console.log('🔄 Page focused, refreshing services...');
                    this.fetchServices();
                }

                // Refresh ingress if on ingress tab and TTL expired
                if (activeTab === 'ingress' && now - this.lastIngressFetch >= this.SERVICES_INGRESS_TTL) {
                    console.log('🔄 Page focused, refreshing ingress...');
                    this.fetchIngress();
                }
            }
        });

        console.log('✅ K8s Manager initialized (Services/Ingress cache: 3min TTL, refresh on focus)');
    },

    fetchAll() {
        this.fetchArgoStatus();
        this.fetchNodesStatus();
        this.fetchPodsStatus();
        this.fetchClusterOverview();
        this.fetchClusterOverview();
        // this.fetchEvents(this.currentEventFilter, this.currentEventPage); // REPLACED BY MONITORS
        this.fetchBackupsStatus();
        this.fetchBackupsStatus();
        this.fetchStorageStatus();
        this.fetchServices();
        this.fetchIngress();
    },

    // === ARGOCD STATUS ===
    async fetchArgoStatus() {
        // Only fetch if we're on the ArgoCD tab
        const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
        if (activeTab !== 'argocd') {
            return; // Skip fetch if not on ArgoCD tab
        }

        try {
            const response = await fetch('/api/argocd/status');
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }
            const data = await response.json();

            if (data.error) {
                this.showArgoError(data.error);
                return;
            }

            this.updateArgoStats(data);
            this.updateArgoIssuesTable(data.apps_with_issues || []);
            this.updateArgoUpgradesTable(data.apps_with_upgrades || []);
        } catch (error) {
            console.error('ArgoCD fetch error:', error);
            this.showArgoError(`Failed to connect to ArgoCD API: ${error.message}`);
        }
    },

    updateArgoStats(data) {
        console.time('ArgoCD Render');
        if (window.performance && window.performance.memory) {
            console.log(`🧠 JS Memory: ${(window.performance.memory.usedJSHeapSize / 1024 / 1024).toFixed(2)} MB`);
        }

        const stats = {
            'stat-total': data.total || 0,
            'stat-healthy': data.healthy || 0,
            'stat-unhealthy': data.unhealthy || 0,
            'stat-synced': data.synced || 0,
            'stat-outofsync': data.out_of_sync || 0,
            'stat-progressing': data.progressing || 0,
            'stat-upgrades': data.upgrades_available || 0,
            'issues-count': (data.apps_with_issues || []).length,
            'upgrades-count': (data.apps_with_upgrades || []).length
        };
        for (const [id, value] of Object.entries(stats)) {
            const el = document.getElementById(id);
            if (el) el.textContent = value;
        }

        // Show message if provided
        if (data.message) {
            console.info('ArgoCD:', data.message);
        }
        console.timeEnd('ArgoCD Render');
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
        console.log('📦 Fetching pods status...');
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 30000); // 30s timeout
        const startTime = performance.now();

        try {
            const response = await fetch('/api/pods/status', { signal: controller.signal });
            clearTimeout(timeoutId);
            const duration = performance.now() - startTime;

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();

            // Debug logging
            if (window.KusanagiDebug) {
                KusanagiDebug.logApiResponse('/api/pods/status', data, duration);
                KusanagiDebug.validatePodsData(data);
            }

            if (data.error) {
                const el = document.getElementById('pods-content');
                if (el) el.innerHTML = `<div class="loading" style="color: #ff4444;">Error: ${data.error}</div>`;
                return;
            }

            // Safely access data with defaults
            const stats = {
                'pods-total': data.total_pods ?? 0,
                'pods-running': data.running_pods ?? 0,
                'pods-pending': data.pending_pods ?? 0,
                'pods-error': data.error_pods ?? 0,
                'pods-error-count': data.pods_in_error?.length ?? 0
            };

            for (const [id, value] of Object.entries(stats)) {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }
            const podsColumns = [
                { key: 'name', label: 'Pod Name' },
                { key: 'namespace', label: 'Namespace' },
                { key: 'status', label: 'Status' },
                { key: 'usage', label: 'Usage (CPU / Mem)' },
                { key: 'limits', label: 'Limits (CPU / Mem)' },
                { key: 'reason', label: 'Reason' },
                { key: 'restart_count', label: 'Restarts' },
                { key: 'age', label: 'Age' },
                { key: 'node', label: 'Node' },
                { key: 'actions', label: 'Actions' }
            ];

            // Combine error pods and pending pods
            const allIssues = [
                ...(data.pods_in_error || []),
                ...(data.pending_pods_list || [])
            ];

            if (window.TableManager && typeof TableManager.init === 'function') {
                TableManager.init('pods', allIssues, (pods) => this.renderPodsTableContent(pods), podsColumns);
                this.renderPodsTable(allIssues);
            } else {
                // Fallback rendering if TableManager is missing
                this.renderPodsTable(allIssues);
            }
            console.log('Pods status fetched successfully');
        } catch (error) {
            console.error('Failed to fetch pods status:', error);
            const el = document.getElementById('pods-content');

            let errorMessage = 'Failed to fetch pods status.';
            if (error.name === 'AbortError') {
                errorMessage = 'Request timed out (backend too slow).';
            } else if (error.message.includes('HTTP error')) {
                errorMessage = `Server Error: ${error.message}`;
            }

            if (el) el.innerHTML = `<div class="loading" style="color: #ff4444;">Error: ${errorMessage} <br> <button class="cyber-btn" onclick="K8sManager.fetchPodsStatus()">Retry</button></div>`;
        } finally {
            clearTimeout(timeoutId);
        }
    },

    renderPodsTable(pods) {
        const container = document.getElementById('pods-content');
        if (!container) return;
        if (!pods || pods.length === 0) {
            container.innerHTML = '<div class="no-issues" style="color: var(--neon-green);">✓ No pods in error or pending state!</div>';
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
        const searchHtml = (window.TableManager && typeof TableManager.createSearchInput === 'function') ?
            TableManager.createSearchInput('pods', 'Search pods...') :
            '<input type="text" placeholder="Search pods..." style="width: 100%; padding: 0.5rem; margin-bottom: 1rem; background: rgba(0,0,0,0.3); border: 1px solid var(--neon-cyan); color: var(--text-color);">';
        const headerHtml = (window.TableManager && TableManager.createSortableHeader) ? TableManager.createSortableHeader('pods', podsColumns) :
            podsColumns.map(col => `<th>${col.label}</th>`).join('');

        // Check if we have error pods to enable the bulk action
        const hasErrorPods = pods && pods.length > 0;
        const restartBtnHtml = hasErrorPods ?
            `<button class="cyber-btn" onclick="K8sManager.restartAllErrorPods()" style="margin-left: 10px; border-color: #ff4444; color: #ff4444;">
                🔥 Restart All Issues
            </button>` : '';

        container.innerHTML = `
            <div style="display: flex; align-items: center; justify-content: space-between;">
                ${searchHtml}
                ${restartBtnHtml}
            </div>
            <table class="issues-table" id="pods-table">
                <thead><tr>${headerHtml}</tr></thead>
                <tbody id="pods-table-body">${this.renderPodsRows(pods)}</tbody>
            </table>
        `;
    },

    async restartAllErrorPods() {
        const confirmed = confirm(`⚠️ RESTART ALL ERROR PODS\n\nAre you sure you want to force delete ALL pods currently in error state?\n\nThis will trigger a bulk delete operation.`);
        if (!confirmed) return;

        const btn = document.querySelector('button[onclick="K8sManager.restartAllErrorPods()"]');
        if (btn) {
            btn.disabled = true;
            btn.textContent = '⏳ Processing...';
        }

        try {
            if (window.showNotification) showNotification({ title: 'Processing', message: 'Starting bulk restart of error pods...', severity: 'info' });

            const response = await fetch('/api/pods/delete-error-pods', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' }
            });
            const data = await response.json();

            if (data.success) {
                if (window.showNotification) showNotification({
                    title: 'Batch Operation Complete',
                    message: data.message || `Successfully processed ${data.deleted_count} pods`,
                    severity: 'success'
                });
                // Refresh list after a short delay
                setTimeout(() => this.fetchPodsStatus(), 2000);
            } else {
                if (window.showNotification) showNotification({ title: 'Operation Failed', message: data.message || 'Unknown error', severity: 'error' });
                if (btn) {
                    btn.disabled = false;
                    btn.textContent = '🔥 Restart All Issues';
                }
            }
        } catch (error) {
            console.error('Failed to mass delete pods:', error);
            if (window.showNotification) showNotification({ title: 'Error', message: 'Failed to communicate with server', severity: 'error' });
            if (btn) {
                btn.disabled = false;
                btn.textContent = '🔥 Restart All Issues';
            }
        }
    },

    renderPodsTableContent(pods) {
        const tbody = document.getElementById('pods-table-body');
        if (tbody) tbody.innerHTML = this.renderPodsRows(pods);
    },

    renderPodsRows(pods) {
        if (!pods || pods.length === 0) return '<tr><td colspan="10" style="text-align:center;">No pods in error state</td></tr>';

        return pods.map(pod => {
            // Safely access pod properties with defaults
            const name = pod.name || 'Unknown';
            const namespace = pod.namespace || 'default';
            const status = pod.status || 'Unknown';
            const statusClass = this.getK8sStatusClass(status);

            // Format CPU Usage / Limit
            const cpuUsage = this.formatCpu(pod.cpu_usage);
            const cpuLimit = this.formatCpu(pod.cpu_limit);
            const cpuDisplay = (cpuUsage !== '-' || cpuLimit !== '-') ?
                `${cpuUsage} / <span style="opacity:0.6">${cpuLimit}</span>` : '-';

            // Format Memory Usage / Limit
            const memUsage = this.formatMemory(pod.memory_usage);
            const memLimit = this.formatMemory(pod.memory_limit);
            const memDisplay = (memUsage !== '-' || memLimit !== '-') ?
                `${memUsage} / <span style="opacity:0.6">${memLimit}</span>` : '-';

            return `
            <tr>
                <td class="col-name" style="font-weight: bold;">${this.escapeHtml(name)}</td>
                <td>${this.escapeHtml(namespace)}</td>
                <td><span class="status-badge ${statusClass}">${status}</span></td>
                <td style="font-family: monospace; font-size: 0.85em;">${cpuDisplay}</td>
                <td style="font-family: monospace; font-size: 0.85em;">${memDisplay}</td>
                <td style="color: #ff4444;">${pod.reason ? this.escapeHtml(pod.reason) : '-'}</td>
                <td style="text-align: center;">${pod.restart_count ?? 0}</td>
                <td>${pod.age || '-'}</td>
                <td style="font-size: 0.8em; opacity: 0.8;">${pod.node ? this.escapeHtml(pod.node) : '-'}</td>
                <td>
                    <div style="display: flex; gap: 5px;">
                        <button class="cyber-btn sm" onclick="K8sManager.viewPodLogs('${this.escapeHtml(namespace)}', '${this.escapeHtml(name)}')" title="View Logs">📄</button>
                        <button class="cyber-btn sm" onclick="K8sManager.forceDeletePod('${this.escapeHtml(namespace)}', '${this.escapeHtml(name)}')" title="Delete Pod" style="border-color: #ff4444; color: #ff4444;">🗑️</button>
                    </div>
                </td>
            </tr>
        `}).join('');
    },

    escapeHtml(text) {
        if (!text) return '';
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    },

    formatCpu(cores) {
        if (cores === undefined || cores === null) return '-';
        if (cores < 0.001) return '0';
        if (cores < 1) return Math.round(cores * 1000) + 'm';
        return cores.toFixed(2);
    },

    formatMemory(bytes) {
        if (bytes === undefined || bytes === null) return '-';
        if (bytes === 0) return '0';
        const units = ['B', 'Ki', 'Mi', 'Gi', 'Ti'];
        let i = 0;
        while (bytes >= 1024 && i < units.length - 1) {
            bytes /= 1024;
            i++;
        }
        return bytes.toFixed(1) + units[i];
    },

    getK8sStatusClass(status) {
        if (!status) return 'unknown';
        switch (status.toLowerCase()) {
            case 'running':
            case 'succeeded': return 'healthy';
            case 'pending': return 'progressing';
            case 'failed': return 'unhealthy';
            default: return 'unknown';
        }
    },

    // === POD LOGS ===
    async viewPodLogs(namespace, podName) {
        console.log(`📄 Fetching logs for ${namespace}/${podName}`);
        const modal = document.getElementById('logs-modal');
        const title = document.getElementById('logs-modal-title');
        const content = document.getElementById('logs-modal-content');

        if (!modal || !title || !content) {
            console.error('Logs modal elements not found');
            return;
        }

        title.textContent = `📄 Pod Logs: ${namespace}/${podName}`;
        content.innerHTML = '<div class="loading">Loading logs...</div>';
        modal.style.display = 'flex';

        try {
            const response = await fetch(`/api/pods/${namespace}/${podName}/logs?tail=500`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);

            const logs = await response.text();

            // Parse ANSI codes if parser is available
            if (window.AnsiParser) {
                const parsedLogs = AnsiParser.parse(logs);
                content.innerHTML = `<div class="ansi-log" style="max-height: 70vh; overflow-y: auto;">${parsedLogs}</div>`;
            } else {
                content.innerHTML = `<pre style="white-space: pre-wrap; word-wrap: break-word; max-height: 70vh; overflow-y: auto; font-family: monospace; font-size: 12px; line-height: 1.5;">${this.escapeHtml(logs)}</pre>`;
            }
        } catch (error) {
            console.error('Failed to fetch logs:', error);
            content.innerHTML = `<div style="color: #ff4444;">Error loading logs: ${error.message}</div>`;
        }
    },

    closeLogsModal() {
        const modal = document.getElementById('logs-modal');
        if (modal) modal.style.display = 'none';
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
                this.renderNodesError(data.error);
                return;
            }
            if (data._warning) {
                console.warn('Nodes warning:', data._warning);
            }

            // Update stats
            const stats = {
                'node-total': data.total_nodes,
                'node-ready': data.ready_nodes,
                'node-notready': data.not_ready_nodes,
                'node-cpu-total': data.total_cpu || '-',
                'node-memory-total': data.total_memory || '-'
            };
            for (const [id, value] of Object.entries(stats)) {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }
            this.renderNodes(data);
        } catch (error) {
            console.error('Nodes error:', error);
            this.renderNodesError('Failed to fetch nodes status from server');
        }
    },

    renderNodes(data) {
        const container = document.getElementById('nodes-container');
        if (!container) return;

        const nodes = data.nodes || [];
        const warningMsg = data._warning || data.warning_message;

        if (!nodes || nodes.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 2rem; text-align: center;">
                    <span style="font-size: 2rem;">🖥️</span>
                    <p>No nodes found</p>
                    ${warningMsg ? `<p style="color: var(--neon-orange); margin-top: 1rem; font-size: 0.9rem;">⚠️ ${warningMsg}</p>` : ''}
                    <button onclick="K8sManager.fetchNodesStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                </div>
            `;
            return;
        }

        nodes.sort((a, b) => a.name.localeCompare(b.name));

        container.innerHTML = nodes.map(node => {
            const isReady = node.status === 'Ready';
            const cpuPercent = node.cpu_usage_percent || 0;
            const memPercent = node.memory_usage_percent || 0;
            const podPercent = node.pod_capacity ? Math.round((node.pod_count / parseInt(node.pod_capacity)) * 100) : 0;

            const getCpuClass = (p) => p > 90 ? 'bar-danger' : p > 75 ? 'bar-warning' : 'bar-ok';
            const getMemClass = (p) => p > 90 ? 'bar-danger' : p > 75 ? 'bar-warning' : 'bar-ok';
            const getPodClass = (p) => p > 90 ? 'bar-danger' : p > 75 ? 'bar-warning' : 'bar-ok';

            return `
                <div class="node-card ${isReady ? 'ready' : 'not-ready'}">
                    <div class="node-header">
                        <div class="node-name">${node.name}</div>
                        <div class="node-status ${isReady ? 'ready' : 'not-ready'}">${node.status}</div>
                    </div>
                    
                    <div class="node-info-grid">
                        <div class="node-info-item">
                            <div class="node-info-label">Architecture</div>
                            <div class="node-info-value">${node.architecture || 'N/A'}</div>
                        </div>
                        <div class="node-info-item">
                            <div class="node-info-label">OS</div>
                            <div class="node-info-value">${node.os || 'N/A'}</div>
                        </div>
                        <div class="node-info-item">
                            <div class="node-info-label">Kernel</div>
                            <div class="node-info-value">${node.kernel_version || 'N/A'}</div>
                        </div>
                        <div class="node-info-item">
                            <div class="node-info-label">Kubelet</div>
                            <div class="node-info-value">${node.kubelet_version || 'N/A'}</div>
                        </div>
                    </div>
                    
                    <div class="node-resources">
                        <div>
                            <div class="resource-bar">
                                <div class="resource-label">
                                    <span>CPU</span>
                                    <span>${cpuPercent.toFixed(1)}% (${node.cpu_capacity || 'N/A'})</span>
                                </div>
                                <div class="pod-bar-container">
                                    <div class="pod-bar ${getCpuClass(cpuPercent)}" style="width: ${Math.min(cpuPercent, 100)}%"></div>
                                </div>
                            </div>
                            
                            <div class="resource-bar">
                                <div class="resource-label">
                                    <span>Pods</span>
                                    <span>${node.pod_count || 0} / ${node.pod_capacity || 'N/A'}</span>
                                </div>
                                <div class="pod-bar-container">
                                    <div class="pod-bar ${getPodClass(podPercent)}" style="width: ${Math.min(podPercent, 100)}%"></div>
                                </div>
                            </div>
                        </div>
                        
                        <div>
                            <div class="resource-bar">
                                <div class="resource-label">
                                    <span>Memory</span>
                                    <span>${memPercent.toFixed(1)}% (${node.memory_allocatable || 'N/A'})</span>
                                </div>
                                <div class="pod-bar-container">
                                    <div class="pod-bar ${getMemClass(memPercent)}" style="width: ${Math.min(memPercent, 100)}%"></div>
                                </div>
                            </div>
                            
                            <div class="resource-bar">
                                <div class="resource-label">
                                    <span>Age</span>
                                    <span>${node.age || 'N/A'}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    ${node.conditions ? `
                    <div class="node-conditions">
                        ${Object.entries(node.conditions).map(([key, value]) =>
                `<span class="condition-item ${value === 'True' ? 'true' : 'false'}">${key}: ${value}</span>`
            ).join('')}
                    </div>
                    ` : ''}
                </div>
            `;
        }).join('');
    },

    renderNodesError(message) {
        const container = document.getElementById('nodes-container');
        if (!container) return;
        container.innerHTML = `
            <div class="error-state" style="padding: 2rem; text-align: center;">
                <span style="font-size: 2rem;">⚠️</span>
                <p style="color: #ff4444;">Failed to load nodes</p>
                <p style="color: var(--text-secondary); font-size: 0.9rem;">${message}</p>
                <button onclick="K8sManager.fetchNodesStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
            </div>
        `;
        // Show diagnostic tool on error
        const diagnosticTool = document.getElementById('nodes-diagnostic-tool');
        if (diagnosticTool) diagnosticTool.style.display = 'block';
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
            if (!response.ok) {
                throw new Error(`Server returned ${response.status}`);
            }
            const data = await response.json();

            // Update stats grid
            const stats = {
                'pvc-total-count': data.pvc_count || 0,
                'pvc-bound-count': (data.pvcs || []).filter(p => p.status === 'Bound').length,
                'pvc-pending-count': (data.pvcs || []).filter(p => p.status !== 'Bound').length,
                'pvc-total-storage': data.pvc_total_capacity || '-'
            };
            for (const [id, value] of Object.entries(stats)) {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }

            if (data.error) {
                console.warn('Storage API returned error:', data.error);
                this.renderStorageError(data.error);
                return;
            }

            this.storageData = data.pvcs || [];
            const countEl = document.getElementById('pvc-table-count');
            if (countEl) countEl.textContent = this.storageData.length;
            this.renderStorageTable(data);
        } catch (error) {
            console.error('Failed to fetch storage status:', error);
            this.renderStorageError('Failed to fetch storage data from server');
        }
    },

    renderStorageError(message) {
        const container = document.getElementById('pvc-content');
        if (!container) return;
        container.innerHTML = `
            <div class="error-state" style="padding: 2rem; text-align: center;">
                <span style="font-size: 2rem;">💾</span>
                <p style="color: #ff4444;">Storage data unavailable</p>
                <p style="color: var(--text-secondary); font-size: 0.9rem;">${message}</p>
                <button onclick="K8sManager.fetchStorageStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
            </div>
        `;
    },

    renderStorageTable(data = null) {
        const container = document.getElementById('pvc-content');
        if (!container) return;

        const warningMsg = data?._warning || data?.warning_message;

        if (!this.storageData || this.storageData.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 2rem; text-align: center;">
                    <span style="font-size: 2rem;">💾</span>
                    <p>No PVCs found</p>
                    ${warningMsg ? `<p style="color: var(--neon-orange); margin-top: 1rem; font-size: 0.9rem;">⚠️ ${warningMsg}</p>` : ''}
                </div>
            `;
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
                    <th onclick="K8sManager.sortStorage('storage_class')" class="sortable">Class ${this.getSortArrow('storage_class')}</th>
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
    // === EVENTS (MOVED TO MONITORS) ===
    // fetchEvents, filterEvents, renderEventsTable removed.


    // === BACKUPS ===
    async fetchBackupsStatus() {
        try {
            const response = await fetch('/api/backups');
            const data = await response.json();

            // Handle explicit error
            if (data.error) {
                console.error('Backups API error:', data.error);
                const container = document.getElementById('backups-content');
                if (container) {
                    container.innerHTML = `
                        <div class="error-state" style="padding: 2rem; text-align: center;">
                            <span style="font-size: 2rem;">⚠️</span>
                            <p>Failed to load backups</p>
                            <p style="color: var(--neon-orange); font-size: 0.9rem;">${data.error}</p>
                            <button onclick="K8sManager.fetchBackupsStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                        </div>
                    `;
                }
                return;
            }

            // Handle warning (e.g., Kubernetes unavailable)
            if (data._warning) {
                console.warn('Backups API warning:', data._warning);
            }

            // Update stats (with fallback to 0)
            const stats = {
                'backup-cronjobs': data.total_cronjobs ?? 0,
                'backup-active': data.active_jobs ?? 0,
                'backup-succeeded': data.succeeded_jobs ?? 0,
                'backup-failed': data.failed_jobs ?? 0,
                'backups-count': data.total_cronjobs ?? 0
            };
            for (const [id, value] of Object.entries(stats)) {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }

            // Render table with warning message if present
            this.renderBackupsTable(data.cronjobs || [], data._warning);
        } catch (error) {
            console.error('Failed to fetch backups:', error);
            const container = document.getElementById('backups-content');
            if (container) {
                container.innerHTML = `
                    <div class="error-state" style="padding: 2rem; text-align: center;">
                        <span style="font-size: 2rem;">⚠️</span>
                        <p>Failed to connect to backups API</p>
                        <p style="color: var(--neon-orange); font-size: 0.9rem;">${error.message}</p>
                        <button onclick="K8sManager.fetchBackupsStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                    </div>
                `;
            }
        }
    },

    renderBackupsTable(cronjobs, warningMsg = null) {
        const container = document.getElementById('backups-content');
        if (!container) return;

        if (!cronjobs || cronjobs.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 2rem; text-align: center;">
                    <span style="font-size: 2rem;">💾</span>
                    <p>No backup cronjobs found</p>
                    ${warningMsg ? `<p style="color: var(--neon-orange); margin-top: 1rem; font-size: 0.9rem;">⚠️ ${warningMsg}</p>` : ''}
                    <button onclick="K8sManager.fetchBackupsStatus()" class="cyber-btn" style="margin-top: 1rem;">Refresh</button>
                </div>
            `;
            return;
        }

        container.innerHTML = `
            <table class="issues-table">
                <thead><tr>
                    <th>CronJob</th>
                    <th>Namespace</th>
                    <th>Schedule</th>
                    <th>Last Run</th>
                    <th>Status</th>
                    <th>Recent Jobs</th>
                    <th>Actions</th>
                </tr></thead>
                <tbody>${cronjobs.map(cj => {
            const statusClass = cj.suspend ? 'suspended' : (cj.active_jobs > 0 ? 'running' : 'healthy');
            const statusText = cj.suspend ? 'Suspended' : (cj.active_jobs > 0 ? 'Running' : 'Idle');

            // Render recent jobs dots
            const recentJobsHtml = (cj.recent_jobs || []).map(job => {
                let color = 'var(--neon-blue)'; // Default/Running
                if (job.status === 'Succeeded') color = 'var(--neon-green)';
                if (job.status === 'Failed') color = 'var(--neon-orange)';

                return `<span title="${job.name} (${job.status}) - ${job.age}" 
                        style="display: inline-block; width: 10px; height: 10px; border-radius: 50%; background-color: ${color}; margin-right: 4px; box-shadow: 0 0 5px ${color};"></span>`;
            }).join('');

            return `<tr>
                        <td class="app-name">${cj.name}</td>
                        <td>${cj.namespace}</td>
                        <td><code>${cj.schedule}</code></td>
                        <td>${cj.last_schedule_age || '-'}</td>
                        <td><span class="status-badge ${statusClass}">${statusText}</span></td>
                        <td>
                            ${recentJobsHtml || '<span style="opacity: 0.5;">No runs</span>'}
                        </td>
                        <td>
                            <button class="cyber-btn small" onclick="K8sManager.triggerCronJob('${cj.namespace}', '${cj.name}')">
                                ▶ Run
                            </button>
                        </td>
                    </tr>`;
        }).join('')}</tbody>
            </table>
        `;
    },

    async triggerCronJob(namespace, name) {
        if (!confirm(`Are you sure you want to trigger backup '${name}'?`)) return;

        try {
            const response = await fetch('/api/backups/trigger', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ namespace, cronjob_name: name })
            });

            if (!response.ok) {
                const err = await response.json();
                throw new Error(err.message || 'Failed to trigger backup');
            }

            const result = await response.json();
            // Show toast or alert
            if (window.showNotification) {
                window.showNotification(result.message, 'success');
            } else {
                alert(result.message);
            }

            // Refresh list after a short delay
            setTimeout(() => this.fetchBackupsStatus(), 1000);
        } catch (error) {
            console.error('Trigger error:', error);
            if (window.showNotification) {
                window.showNotification(`Error: ${error.message}`, 'error');
            } else {
                alert(`Error: ${error.message}`);
            }
        }
    },

    // === SERVICES & INGRESS ===
    async fetchServices() {
        const now = Date.now();
        // Allow initial fetch (lastServicesFetch === 0)
        // Otherwise, check if 3 minutes have passed AND we are on the services tab
        if (this.lastServicesFetch !== 0) {
            const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
            if (activeTab !== 'services') return;
            if (now - this.lastServicesFetch < this.SERVICES_INGRESS_TTL) return;
        }

        this.lastServicesFetch = now;

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
        const now = Date.now();
        // Allow initial fetch (lastIngressFetch === 0)
        // Otherwise, check if 3 minutes have passed AND we are on the ingress tab
        if (this.lastIngressFetch !== 0) {
            const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
            if (activeTab !== 'ingress') return;
            if (now - this.lastIngressFetch < this.SERVICES_INGRESS_TTL) return;
        }

        this.lastIngressFetch = now;

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
