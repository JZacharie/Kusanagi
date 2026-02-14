const K8sStorage = {
    init() {
        console.log('💾 K8s Storage Module Initialized');
    },

    // === PVCs ===
    async fetchStorageStatus() {
        // Only fetch if we're on the Storage tab (or unknown tab for initial load)
        const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
        if (activeTab && activeTab !== 'storage') {
            return;
        }

        try {
            console.log('🔍 Fetching storage status...');
            const data = await api.get('/api/storage');
            console.log('📡 Storage data received:', data);

            K8sState.storageData = data.pvcs || [];

            const stats = {
                'pvc-total-count': data.pvc_count ?? K8sState.storageData.length,
                'pvc-bound-count': K8sState.storageData.filter(p => p.status === 'Bound').length,
                'pvc-pending-count': K8sState.storageData.filter(p => p.status !== 'Bound').length,
                'pvc-total-storage': data.pvc_total_capacity || '-'
            };

            for (const [id, value] of Object.entries(stats)) {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }

            const countEl = document.getElementById('pvc-table-count');
            if (countEl) countEl.textContent = K8sState.storageData.length;

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
                <button onclick="K8sStorage.fetchStorageStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
            </div>
        `;
    },

    renderStorageTable(data = null) {
        const container = document.getElementById('pvc-content');
        if (!container) return;

        const warningMsg = data?._warning || data?.warning_message;

        if (!K8sState.storageData || K8sState.storageData.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 2rem; text-align: center;">
                    <span style="font-size: 2rem;">💾</span>
                    <p>No PVCs found</p>
                    ${warningMsg ? `<p style="color: var(--neon-orange); margin-top: 1rem; font-size: 0.9rem;">⚠️ ${warningMsg}</p>` : ''}
                </div>
            `;
            return;
        }

        const sortedData = [...K8sState.storageData].sort((a, b) => {
            let valA = a[K8sState.storageSortField] ?? -1;
            let valB = b[K8sState.storageSortField] ?? -1;
            if (typeof valA === 'string') { valA = valA.toLowerCase(); valB = valB.toLowerCase(); }
            if (valA < valB) return K8sState.storageSortDir === 'asc' ? -1 : 1;
            if (valA > valB) return K8sState.storageSortDir === 'asc' ? 1 : -1;
            return 0;
        });

        const totalPages = Math.ceil(sortedData.length / K8sState.storagePerPage);
        if (K8sState.storagePage > totalPages) K8sState.storagePage = totalPages || 1;

        const start = (K8sState.storagePage - 1) * K8sState.storagePerPage;
        const pageData = sortedData.slice(start, start + K8sState.storagePerPage);

        container.innerHTML = `
            <table class="issues-table">
                <thead><tr>
                    <th onclick="K8sStorage.sortStorage('name')" class="sortable">Name ${this.getSortArrow('name')}</th>
                    <th onclick="K8sStorage.sortStorage('namespace')" class="sortable">Namespace ${this.getSortArrow('namespace')}</th>
                    <th onclick="K8sStorage.sortStorage('capacity_bytes')" class="sortable">Capacity ${this.getSortArrow('capacity_bytes')}</th>
                    <th onclick="K8sStorage.sortStorage('usage_percent')" class="sortable">Usage ${this.getSortArrow('usage_percent')}</th>
                    <th onclick="K8sStorage.sortStorage('status')" class="sortable">Status ${this.getSortArrow('status')}</th>
                    <th onclick="K8sStorage.sortStorage('storage_class')" class="sortable">Class ${this.getSortArrow('storage_class')}</th>
                </tr></thead>
                <tbody>${pageData.map(pvc => {
            const percent = pvc.usage_percent ?? 0;
            const barClass = percent > 90 ? 'bar-danger' : percent > 75 ? 'bar-warning' : 'bar-ok';
            const displayPercent = percent > 1000 ? percent.toFixed(0) : percent.toFixed(1);
            return `<tr>
                        <td class="app-name">${pvc.name}</td><td>${pvc.namespace}</td><td>${pvc.capacity || K8sState.formatBytes(pvc.capacity_bytes)}</td>
                        <td><div class="usage-cell"><div class="pod-bar-container" title="${K8sState.formatBytes(pvc.used_bytes) || '?'} / ${pvc.capacity}"><div class="pod-bar ${barClass}" style="width: ${Math.min(percent, 100)}%"></div></div><span class="usage-text">${displayPercent}%</span></div></td>
                        <td><span class="status-badge ${pvc.status.toLowerCase()}">${pvc.status}</span></td><td class="storage-class">${pvc.storage_class || '-'}</td>
                    </tr>`;
        }).join('')}</tbody>
            </table>
            <div class="pagination-controls">
                <button ${K8sState.storagePage === 1 ? 'disabled' : ''} onclick="K8sStorage.changeStoragePage(-1)" class="page-btn">◀</button>
                <span class="page-info">Page ${K8sState.storagePage} of ${totalPages}</span>
                <button ${K8sState.storagePage === totalPages ? 'disabled' : ''} onclick="K8sStorage.changeStoragePage(1)" class="page-btn">▶</button>
            </div>
        `;
    },

    sortStorage(field) {
        if (K8sState.storageSortField === field) K8sState.storageSortDir = K8sState.storageSortDir === 'asc' ? 'desc' : 'asc';
        else { K8sState.storageSortField = field; K8sState.storageSortDir = 'desc'; }
        this.renderStorageTable();
    },

    getSortArrow(field) {
        if (K8sState.storageSortField !== field) return '';
        return K8sState.storageSortDir === 'asc' ? '▲' : '▼';
    },

    changeStoragePage(delta) {
        K8sState.storagePage += delta;
        this.renderStorageTable();
    },

    // === BACKUPS ===
    async fetchBackupsStatus() {
        // Only fetch if we're on the Backups tab (or unknown tab for initial load)
        const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
        if (activeTab && activeTab !== 'backups') {
            return;
        }

        try {
            const data = await api.get('/api/backups');

            if (data._warning) {
                console.warn('Backups API warning:', data._warning);
            }

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
                        <button onclick="K8sStorage.fetchBackupsStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
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
                    <button onclick="K8sStorage.fetchBackupsStatus()" class="cyber-btn" style="margin-top: 1rem;">Refresh</button>
                </div>
            `;
            return;
        }

        container.innerHTML = `
            <table class="issues-table">
                <thead><tr>
                    <th>CronJob</th><th>Namespace</th><th>Schedule</th><th>Last Run</th>
                    <th>Status</th><th>Recent Jobs</th><th>Actions</th>
                </tr></thead>
                <tbody>${cronjobs.map(cj => {
            const statusClass = cj.suspend ? 'suspended' : (cj.active_jobs > 0 ? 'running' : 'healthy');
            const statusText = cj.suspend ? 'Suspended' : (cj.active_jobs > 0 ? 'Running' : 'Idle');

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
                        <td>${recentJobsHtml || '<span style="opacity: 0.5;">No runs</span>'}</td>
                        <td>
                            <button class="cyber-btn small" onclick="K8sStorage.triggerCronJob('${cj.namespace}', '${cj.name}')">
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
            const result = await api.post('/api/backups/trigger', { namespace, cronjob_name: name });
            if (window.showNotification) window.showNotification(result.message, 'success');
            setTimeout(() => this.fetchBackupsStatus(), 1000);
        } catch (error) {
            console.error('Trigger error:', error);
            if (window.showNotification) window.showNotification(`Error: ${error.message}`, 'error');
        }
    }
};

window.K8sStorage = K8sStorage;
