const K8sPods = {
    init() {
        console.log('📦 K8s Pods Module Initialized');
    },

    async fetchPodsStatus() {
        console.log('📦 Fetching pods status...');
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort('timeout'), 60000); // 60s timeout
        const startTime = performance.now();

        try {
            const data = await apiFetch('/api/k8s/pods', { signal: controller.signal });
            clearTimeout(timeoutId);
            const duration = performance.now() - startTime;

            if (window.KusanagiDebug) {
                KusanagiDebug.logApiResponse('/api/k8s/pods', data, duration);
                KusanagiDebug.validatePodsData(data);
            }

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

            const allIssues = [
                ...(data.pods_in_error || []),
                ...(data.pending_pods_list || [])
            ];

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

            if (window.TableManager && typeof TableManager.init === 'function') {
                TableManager.init('pods', allIssues, (pods) => this.renderPodsTableContent(pods), podsColumns);
                this.renderPodsTable(allIssues);
            } else {
                this.renderPodsTable(allIssues);
            }
            console.log('Pods status fetched successfully');
        } catch (error) {
            console.error('Failed to fetch pods status:', error);
            const el = document.getElementById('pods-content');

            let errorMessage = 'Failed to fetch pods status.';
            if (error.name === 'AbortError' || error.message === 'timeout') {
                errorMessage = 'Request timed out (backend too slow).';
            } else if (error.message.includes('HTTP error')) {
                errorMessage = `Server Error: ${error.message}`;
            }

            if (el) el.innerHTML = `<div class="loading" style="color: #ff4444;">Error: ${errorMessage} <br> <button class="cyber-btn" onclick="K8sPods.fetchPodsStatus()">Retry</button></div>`;
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
            TableManager.createSearchInput('pods', 'Search pods...') : '';

        const headerHtml = (window.TableManager && TableManager.createSortableHeader) ? TableManager.createSortableHeader('pods', podsColumns) :
            podsColumns.map(col => `<th>${col.label}</th>`).join('');

        const hasErrorPods = pods && pods.length > 0;
        const restartBtnHtml = hasErrorPods ?
            `<button class="cyber-btn" onclick="K8sPods.restartAllErrorPods()" style="margin-left: 10px; border-color: #ff4444; color: #ff4444;">
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

    renderPodsTableContent(pods) {
        const tbody = document.getElementById('pods-table-body');
        if (tbody) tbody.innerHTML = this.renderPodsRows(pods);
    },

    renderPodsRows(pods) {
        if (!pods || pods.length === 0) return '<tr><td colspan="10" style="text-align:center;">No pods in error state</td></tr>';

        return pods.map(pod => {
            const name = pod.name || 'Unknown';
            const namespace = pod.namespace || 'default';
            const status = pod.status || 'Unknown';
            const statusClass = this.getK8sStatusClass(status);

            const cpuDisplay = (K8sState.formatCpu(pod.cpu_usage) !== '-' || K8sState.formatCpu(pod.cpu_limit) !== '-') ?
                `${K8sState.formatCpu(pod.cpu_usage)} / <span style="opacity:0.6">${K8sState.formatCpu(pod.cpu_limit)}</span>` : '-';

            const memDisplay = (K8sState.formatBytes(pod.memory_usage) !== '-' || K8sState.formatBytes(pod.memory_limit) !== '-') ?
                `${K8sState.formatBytes(pod.memory_usage)} / <span style="opacity:0.6">${K8sState.formatBytes(pod.memory_limit)}</span>` : '-';

            return `
            <tr>
                <td class="col-name" style="font-weight: bold;">${K8sState.escapeHtml(name)}</td>
                <td>${K8sState.escapeHtml(namespace)}</td>
                <td><span class="status-badge ${statusClass}">${status}</span></td>
                <td style="font-family: monospace; font-size: 0.85em;">${cpuDisplay}</td>
                <td style="font-family: monospace; font-size: 0.85em;">${memDisplay}</td>
                <td style="color: #ff4444;">${pod.reason ? K8sState.escapeHtml(pod.reason) : '-'}</td>
                <td style="text-align: center;">${pod.restart_count ?? 0}</td>
                <td>${pod.age || '-'}</td>
                <td style="font-size: 0.8em; opacity: 0.8;">${pod.node ? K8sState.escapeHtml(pod.node) : '-'}</td>
                <td>
                    <div style="display: flex; gap: 5px;">
                        <button class="cyber-btn sm" onclick="K8sPods.viewPodLogs('${K8sState.escapeHtml(namespace)}', '${K8sState.escapeHtml(name)}')" title="View Logs">📄</button>
                        <button class="cyber-btn sm" onclick="K8sPods.forceDeletePod('${K8sState.escapeHtml(namespace)}', '${K8sState.escapeHtml(name)}')" title="Delete Pod" style="border-color: #ff4444; color: #ff4444;">🗑️</button>
                    </div>
                </td>
            </tr>
        `}).join('');
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

    async restartAllErrorPods() {
        if (!confirm(`⚠️ RESTART ALL ERROR PODS\n\nAre you sure you want to delete ALL pods in error state?`)) return;

        const btn = document.querySelector('button[onclick="K8sPods.restartAllErrorPods()"]');
        if (btn) { btn.disabled = true; btn.textContent = '⏳ Processing...'; }

        try {
            if (window.showNotification) showNotification({ title: 'Processing', message: 'Starting bulk restart...', severity: 'info' });
            const data = await api.post('/api/pods/delete-error-pods');

            if (data.success) {
                if (window.showNotification) showNotification({ title: 'Complete', message: data.message, severity: 'success' });
                setTimeout(() => this.fetchPodsStatus(), 2000);
            } else {
                if (window.showNotification) showNotification({ title: 'Failed', message: data.message, severity: 'error' });
                if (btn) { btn.disabled = false; btn.textContent = '🔥 Restart All Issues'; }
            }
        } catch (error) {
            console.error('Failed to mass delete pods:', error);
            if (btn) { btn.disabled = false; btn.textContent = '🔥 Restart All Issues'; }
        }
    },

    async forceDeletePod(namespace, podName) {
        if (!confirm(`⚠️ Force Delete Pod: ${namespace}/${podName}?`)) return;
        try {
            const data = await api.post('/api/pods/force-delete', { namespace, pod_name: podName });
            if (data.success) {
                if (window.showNotification) showNotification({ title: 'Pod Deleted', message: `Deleted ${podName}`, severity: 'success' });
                setTimeout(() => this.fetchPodsStatus(), 1000);
            } else {
                if (window.showNotification) showNotification({ title: 'Delete Failed', message: data.message, severity: 'error' });
            }
        } catch (error) {
            console.error('Failed to delete pod:', error);
        }
    },

    async viewPodLogs(namespace, podName) {
        console.log(`📄 Fetching logs for ${namespace}/${podName}`);
        const modal = document.getElementById('logs-modal');
        const title = document.getElementById('logs-modal-title');
        const content = document.getElementById('logs-modal-content');

        if (!modal || !title || !content) return;

        title.textContent = `📄 Pod Logs: ${namespace}/${podName}`;
        content.innerHTML = '<div class="loading">Loading logs...</div>';
        modal.style.display = 'flex';

        try {
            const response = await fetch(`/api/k8s/pods/${namespace}/${podName}/logs?tail=500`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            const logs = await response.text();

            if (window.AnsiParser) {
                content.innerHTML = `<div class="ansi-log" style="max-height: 70vh; overflow-y: auto;">${AnsiParser.parse(logs)}</div>`;
            } else {
                content.innerHTML = `<pre style="white-space: pre-wrap; word-wrap: break-word; max-height: 70vh; overflow-y: auto; font-family: monospace;">${K8sState.escapeHtml(logs)}</pre>`;
            }
        } catch (error) {
            content.innerHTML = `<div style="color: #ff4444;">Error loading logs: ${error.message}</div>`;
        }
    },

    closeLogsModal() {
        const modal = document.getElementById('logs-modal');
        if (modal) modal.style.display = 'none';
    }
};

window.K8sPods = K8sPods;
