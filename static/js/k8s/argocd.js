const K8sArgo = {
    init() {
        console.log('🚀 K8s ArgoCD Module Initialized');
    },

    async fetchArgoStatus() {
        // Only fetch if we're on the ArgoCD tab (or unknown tab for initial load)
        const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
        if (activeTab && activeTab !== 'argocd') {
            console.log('Skipping ArgoCD fetch (not active tab:', activeTab + ')');
            return;
        }

        try {
            const data = await api.get('/api/argocd/status');
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
        if (data.message) console.info('ArgoCD:', data.message);
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
                        <button class="sync-btn" onclick="K8sArgo.syncApp(event, '${app.name}')" title="Sync ${app.name}">
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
                <thead><tr>
                    <th>Application</th><th>Namespace</th><th>Health</th><th>Sync</th>
                    <th>Revision</th><th>Duration</th><th>Message</th>
                </tr></thead>
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
                <thead><tr>
                    <th>Application</th><th>Namespace</th><th>Health</th><th>Sync</th>
                    <th>Revision</th><th>Duration</th><th>Action</th>
                </tr></thead>
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
            const data = await api.post('/api/argocd/sync', { app_name: appName });
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
    }
};

window.K8sArgo = K8sArgo;
