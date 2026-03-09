/**
 * K8sStorage - Storage and Backups management
 * Note: Polling is handled by K8sManager (tab-aware)
 */
const K8sStorage = {
    init() {
        console.log('💾 K8s Storage Module Initialized (no internal polling)');
    },

    // === PVCs ===
    async fetchStorageStatus() {
        try {
            console.log('🔍 Fetching storage status...');
            const data = await api.get('/api/storage/analysis');
            console.log('📡 Storage analysis data received:', data);

            K8sState.storageData = data.pvcs || (data.proxmox_volumes && data.proxmox_volumes.length > 0 ? data.pvcs : []); // API backwards compat

            const storageData = await api.get('/api/storage');
            K8sState.storageData = storageData.pvcs || [];

            const stats = {
                'pvc-total-count': storageData.pvc_count ?? K8sState.storageData.length,
                'pvc-bound-count': K8sState.storageData.filter(p => p.status === 'Bound').length,
                'pvc-total-storage': storageData.pvc_total_capacity || '-',
                'pvc-unattached-count': data.unattached_pvcs ? data.unattached_pvcs.length : 0,
                'proxmox-orphaned-count': data.orphaned_proxmox_volumes ? data.orphaned_proxmox_volumes.length : 0
            };

            for (const [id, value] of Object.entries(stats)) {
                const el = document.getElementById(id);
                if (el) el.textContent = value;
            }

            const countEl = document.getElementById('pvc-table-count');
            if (countEl) countEl.textContent = K8sState.storageData.length;

            this.renderStorageTable(storageData);
            this.renderUnattachedPvcs(data.unattached_pvcs || []);
            this.renderOrphanedPv(data.orphaned_proxmox_volumes || []);

        } catch (error) {
            console.error('Failed to fetch storage status:', error);
            this.renderStorageError('Failed to fetch storage data from server');
        }
    },

    renderUnattachedPvcs(unattached) {
        const container = document.getElementById('unattached-pvc-container');
        const content = document.getElementById('unattached-pvc-content');
        const countSpan = document.getElementById('unattached-pvc-table-count');

        if (!container || !content) return;

        if (unattached.length === 0) {
            container.style.display = 'none';
            return;
        }

        container.style.display = 'block';
        if (countSpan) countSpan.textContent = unattached.length;

        content.innerHTML = `
            <table class="issues-table">
                <thead><tr>
                    <th>PVC Name</th>
                    <th>Namespace</th>
                    <th>PV Volume Name</th>
                    <th>Reason</th>
                    <th>Actions</th>
                </tr></thead>
                <tbody>${unattached.map(pvc => `
                    <tr>
                        <td class="app-name">${pvc.name}</td>
                        <td>${pvc.namespace}</td>
                        <td>${pvc.volume_name}</td>
                        <td><span style="color: var(--neon-red);">${pvc.reason}</span></td>
                        <td>
                            <button class="cyber-btn small danger" onclick="K8sStorage.deletePvc('${pvc.namespace}', '${pvc.name}')">Delete PVC</button>
                        </td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    },

    async deletePvc(namespace, name) {
        if (!confirm(`Are you sure you want to delete PVC ${name} in ${namespace}?\nThis action cannot be undone.`)) return;
        // Requires delete API implementation on backend. I'll mock it or use kubectl for now.
        alert('Please run: ./scripts/force-detach-k8s-volume.sh or kubectl delete pvc');
    },

    renderOrphanedPv(orphans) {
        const container = document.getElementById('orphaned-proxmox-container');
        const content = document.getElementById('orphaned-proxmox-content');
        const countSpan = document.getElementById('orphaned-proxmox-table-count');

        if (!container || !content) return;

        if (orphans.length === 0) {
            container.style.display = 'none';
            return;
        }

        container.style.display = 'block';
        if (countSpan) countSpan.textContent = orphans.length;

        content.innerHTML = `
            <table class="issues-table">
                <thead><tr>
                    <th>Volume ID / Disk</th>
                    <th>Proxmox Node</th>
                    <th>Storage Pool</th>
                    <th>Size</th>
                    <th>Format</th>
                    <th>Actions</th>
                </tr></thead>
                <tbody>${orphans.map(vol => {
            const sizeBytes = vol.size || 0;
            const sizeFormatted = sizeBytes > 0 ? (sizeBytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB' : '-';
            return `
                    <tr>
                        <td class="app-name">${vol.volid}</td>
                        <td>${vol.proxmox_node}</td>
                        <td>${vol.proxmox_storage}</td>
                        <td>${sizeFormatted}</td>
                        <td>${vol.format || '-'}</td>
                        <td>
                            <button class="cyber-btn small danger" onclick="K8sStorage.deleteProxmoxVolume('${vol.proxmox_url}', '${vol.proxmox_node}', '${vol.proxmox_storage}', '${vol.volid}')">Delete Volume</button>
                        </td>
                    </tr>
                `}).join('')}</tbody>
            </table>
        `;
    },

    async deleteProxmoxVolume(serverUrl, node, storage, volume) {
        if (!confirm(`DANGER: Are you absolutely sure you want to delete ${volume} from Proxmox (${node}/${storage})?\nTHIS WILL PERMANENTLY DESTROY DATA!`)) return;

        try {
            // Strip the http/https for the server parameter if needed, or pass it url encoded
            const encodedServer = encodeURIComponent(serverUrl);
            const encodedVolume = encodeURIComponent(volume);

            const result = await api.delete(`/api/proxmox/volume/${encodedServer}/${node}/${storage}/${encodedVolume}`);
            if (window.showNotification) window.showNotification(result.message || 'Volume deleted', 'success');
            setTimeout(() => this.fetchStorageStatus(), 1500);
        } catch (error) {
            console.error('Delete error:', error);
            if (window.showNotification) window.showNotification(`Error: ${error.message}`, 'error');
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
                // Check for 429 rate limit error
                const isRateLimit = error.message?.includes('429') ||
                    error.message?.includes('Too many requests');
                const errorTitle = isRateLimit ? '⏳ Rate Limited' : '⚠️ API Error';
                const errorColor = isRateLimit ? 'var(--neon-yellow)' : 'var(--neon-orange)';
                const errorMsg = isRateLimit
                    ? 'Kubernetes API rate limit reached. Please wait a moment...'
                    : error.message;

                container.innerHTML = `
                    <div class="error-state" style="padding: 2rem; text-align: center;">
                        <span style="font-size: 2rem;">${isRateLimit ? '⏳' : '⚠️'}</span>
                        <p style="color: ${errorColor};">${errorTitle}</p>
                        <p style="color: var(--text-secondary); font-size: 0.9rem;">${errorMsg}</p>
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
