/**
 * K8sArgo - ArgoCD Management Dashboard Module
 * Dynamic Card Grid with Drag-and-Drop Layout persistence, Custom Icons, and Resource Usage monitoring
 */
const K8sArgo = {
    // Current state
    applications: [],
    currentFilter: 'all',
    searchQuery: '',

    init() {
        console.log('🚀 K8s ArgoCD Dashboard Module Initialized (Features added at the bottom)');
    },

    async fetchArgoStatus() {
        try {
            const data = await api.get('/api/argocd/status');
            
            // Store applications globally
            this.applications = data.applications || [];
            
            this.updateArgoStats(data);
            this.renderApplicationsGrid();
            
            // Keep existing tables populated
            this.updateArgoIssuesTable(data.apps_with_issues || []);
            this.updateArgoUpgradesTable(data.apps_with_upgrades || []);
        } catch (error) {
            console.warn('ArgoCD not available:', error.message);
            this.showArgoNotAvailable(error.message || 'ArgoCD not detected');
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
        console.timeEnd('ArgoCD Render');
    },

    // Maps application name or patterns to standard Material Design Icons (mdi-*)
    getAppIcon(appName) {
        const name = appName.toLowerCase();
        
        // Infrastructure / Monitoring
        if (name.includes('grafana')) return 'mdi-chart-line';
        if (name.includes('prometheus')) return 'mdi-fire';
        if (name.includes('alertmanager') || name.includes('alert')) return 'mdi-bell-ring';
        if (name.includes('loki') || name.includes('log')) return 'mdi-math-log';
        if (name.includes('vault') || name.includes('secret')) return 'mdi-safe';
        if (name.includes('external-secrets')) return 'mdi-key-chain';
        if (name.includes('cert-manager')) return 'mdi-certificate';
        
        // Databases
        if (name.includes('qdrant') || name.includes('vector')) return 'mdi-cube-scan';
        if (name.includes('postgres') || name.includes('pg')) return 'mdi-database-import';
        if (name.includes('redis')) return 'mdi-database-sync';
        if (name.includes('mysql') || name.includes('mariadb')) return 'mdi-database';
        
        // Network / Gateways
        if (name.includes('traefik')) return 'mdi-router-wireless';
        if (name.includes('ingress') || name.includes('nginx')) return 'mdi-web';
        if (name.includes('cilium')) return 'mdi-shield-half-full';
        
        // Integrations
        if (name.includes('home-assistant') || name.includes('homeassistant') || name.includes('ha-')) return 'mdi-home-assistant';
        if (name.includes('mqtt') || name.includes('mosquitto')) return 'mdi-radio-tower';
        if (name.includes('plex')) return 'mdi-plex';
        if (name.includes('jellyfin')) return 'mdi-youtube-tv';
        if (name.includes('transmission') || name.includes('torrent')) return 'mdi-download-network';
        
        // AI / ML
        if (name.includes('qwen') || name.includes('llm') || name.includes('ollama') || name.includes('ai-')) return 'mdi-robot-outline';

        // Default Kubernetes logo representation
        return 'mdi-kubernetes';
    },

    // Render applications into the grid layout
    renderApplicationsGrid() {
        const grid = document.getElementById('argocd-grid');
        if (!grid) return;

        if (this.applications.length === 0) {
            grid.innerHTML = '<div class="no-issues" style="grid-column: 1/-1;">No applications found in the cluster.</div>';
            return;
        }

        // Apply search and status filters
        const filteredApps = this.applications.filter(app => {
            const matchesSearch = 
                app.name.toLowerCase().includes(this.searchQuery) ||
                (app.namespace && app.namespace.toLowerCase().includes(this.searchQuery)) ||
                (app.repo_url && app.repo_url.toLowerCase().includes(this.searchQuery)) ||
                (app.description && app.description.toLowerCase().includes(this.searchQuery));

            let matchesStatus = true;
            if (this.currentFilter === 'Healthy') {
                matchesStatus = app.health_status === 'Healthy';
            } else if (this.currentFilter === 'OutOfSync') {
                matchesStatus = app.sync_status === 'OutOfSync';
            } else if (this.currentFilter === 'Degraded') {
                matchesStatus = app.health_status === 'Degraded' || app.health_status === 'Missing';
            }

            return matchesSearch && matchesStatus;
        });

        if (filteredApps.length === 0) {
            grid.innerHTML = '<div class="no-issues" style="grid-column: 1/-1;">No applications match the active filters.</div>';
            return;
        }

        // Sort applications based on saved layout positions
        const savedOrder = JSON.parse(localStorage.getItem('kusanagi-argocd-card-order') || '[]');
        if (savedOrder.length > 0) {
            filteredApps.sort((a, b) => {
                const posA = savedOrder.indexOf(a.name);
                const posB = savedOrder.indexOf(b.name);
                if (posA === -1 && posB === -1) return 0;
                if (posA === -1) return 1;
                if (posB === -1) return -1;
                return posA - posB;
            });
        }

        grid.innerHTML = filteredApps.map(app => {
            const icon = this.getAppIcon(app.name);
            const statusClass = app.health_status.toLowerCase();
            const syncClass = app.sync_status.toLowerCase().replace(' ', '');

            // Resource warning status thresholds
            const cpuWarnClass = app.cpu_percent > 80 ? 'critical' : (app.cpu_percent > 50 ? 'warning' : '');
            const memWarnClass = app.memory_percent > 80 ? 'critical' : (app.memory_percent > 50 ? 'warning' : '');

            return `
                <div class="argocd-card ${statusClass}" draggable="true" data-name="${app.name}">
                    <div class="argocd-card-header">
                        <div class="argocd-card-title-group">
                            <div class="argocd-card-icon">
                                <i class="mdi ${icon}"></i>
                            </div>
                            <div>
                                <h3 class="argocd-card-title">${app.name}</h3>
                                <div class="argocd-card-ns">ns: ${app.namespace}</div>
                            </div>
                        </div>
                        <div class="argocd-card-badges">
                            <span class="card-status-badge ${statusClass}">${app.health_status}</span>
                            <span class="card-status-badge ${syncClass}">${app.sync_status}</span>
                        </div>
                    </div>
                    
                    <div class="argocd-card-desc" title="${app.description}">
                        ${app.description}
                    </div>

                    <div class="argocd-resources">
                        <div class="resource-row">
                            <div class="resource-label">
                                <span>CPU Usage</span>
                                <span class="resource-value">${app.cpu_usage.toFixed(3)} Cores</span>
                            </div>
                            <div class="resource-bar-bg">
                                <div class="resource-bar-fill ${cpuWarnClass}" style="width: ${app.cpu_percent}%"></div>
                            </div>
                        </div>
                        <div class="resource-row" style="margin-top: 0.8rem;">
                            <div class="resource-label">
                                <span>Memory</span>
                                <span class="resource-value">${app.memory_usage_mb.toFixed(0)} MB</span>
                            </div>
                            <div class="resource-bar-bg">
                                <div class="resource-bar-fill ${memWarnClass}" style="width: ${app.memory_percent}%"></div>
                            </div>
                        </div>
                    </div>

                    <div class="argocd-card-actions">
                        <a href="${app.argocd_url}" target="_blank" class="card-action-btn">
                            <i class="mdi mdi-open-in-new"></i> ArgoCD
                        </a>
                        ${app.can_sync ? `
                            <button class="card-action-btn sync-action" onclick="K8sArgo.syncApp(event, '${app.name}')">
                                <i class="mdi mdi-sync"></i> Sync
                            </button>
                        ` : ''}
                    </div>
                </div>
            `;
        }).join('');

        this.initDragAndDrop();
    },

    // Search and Status Filters
    handleSearchFilter() {
        const searchInput = document.getElementById('argocd-search');
        if (searchInput) {
            this.searchQuery = searchInput.value.toLowerCase().trim();
            this.renderApplicationsGrid();
        }
    },

    setFilter(btnElement, status) {
        // Toggle active button class
        document.querySelectorAll('.filter-btn').forEach(btn => btn.classList.remove('active'));
        if (btnElement) btnElement.classList.add('active');
        
        this.currentFilter = status;
        this.renderApplicationsGrid();
    },

    // Native Drag and Drop Layout Positioning
    initDragAndDrop() {
        const cards = document.querySelectorAll('.argocd-card');
        const grid = document.getElementById('argocd-grid');
        
        cards.forEach(card => {
            card.addEventListener('dragstart', (e) => {
                card.classList.add('dragging');
                e.dataTransfer.effectAllowed = 'move';
                e.dataTransfer.setData('text/plain', card.dataset.name);
            });

            card.addEventListener('dragend', () => {
                card.classList.remove('dragging');
                this.saveCardPositions();
            });
        });

        if (grid) {
            grid.addEventListener('dragover', (e) => {
                e.preventDefault();
                const draggingCard = document.querySelector('.argocd-card.dragging');
                if (!draggingCard) return;

                // Find the closest sibling card to insert before
                const afterElement = this.getDragAfterElement(grid, e.clientX, e.clientY);
                if (afterElement == null) {
                    grid.appendChild(draggingCard);
                } else {
                    grid.insertBefore(draggingCard, afterElement);
                }
            });
        }
    },

    getDragAfterElement(container, x, y) {
        const draggableElements = [...container.querySelectorAll('.argocd-card:not(.dragging)')];

        return draggableElements.reduce((closest, child) => {
            const box = child.getBoundingClientRect();
            // Get center points of sibling cards
            const offset = x - box.left - box.width / 2;
            const offsetY = y - box.top - box.height / 2;
            
            // Check closeness on both 2D dimensions
            const distance = Math.sqrt(offset * offset + offsetY * offsetY);
            
            if (distance < closest.distance) {
                return { distance: distance, element: child };
            } else {
                return closest;
            }
        }, { distance: Number.POSITIVE_INFINITY }).element;
    },

    saveCardPositions() {
        const order = [];
        document.querySelectorAll('.argocd-card').forEach(card => {
            order.push(card.dataset.name);
        });
        localStorage.setItem('kusanagi-argocd-card-order', JSON.stringify(order));
    },

    resetPositions() {
        localStorage.removeItem('kusanagi-argocd-card-order');
        this.renderApplicationsGrid();
        
        if (typeof showNotification === 'function') {
            showNotification('Dashboard layout reset successfully!', 'success');
        }
    },

    // Legacy tables updates for full backward compatibility
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
        event.stopPropagation(); // Avoid dragging action
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
    },

    showArgoNotAvailable(reason) {
        this.updateArgoStats({
            total: 0, healthy: 0, unhealthy: 0,
            synced: 0, out_of_sync: 0, progressing: 0, upgrades_available: 0
        });
        
        const grid = document.getElementById('argocd-grid');
        if (grid) {
            grid.innerHTML = `
                <div class="no-issues" style="grid-column: 1/-1; color: var(--text-secondary);">
                    <p>🔌 ArgoCD Status Offline</p>
                    <p style="font-size: 0.85rem; margin-top: 0.5rem;">${reason}</p>
                </div>
            `;
        }
        
        const issuesContainer = document.getElementById('issues-content');
        if (issuesContainer) {
            issuesContainer.innerHTML = `
                <div class="no-issues" style="color: var(--text-secondary);">
                    <p>🔌 ArgoCD is not available</p>
                </div>
            `;
        }
        
        const upgradesContainer = document.getElementById('upgrades-content');
        if (upgradesContainer) {
            upgradesContainer.innerHTML = `
                <div class="no-issues" style="color: var(--text-secondary);">
                    <p>ArgoCD is not available</p>
                </div>
            `;
        }
    }
};

window.K8sArgo = K8sArgo;
