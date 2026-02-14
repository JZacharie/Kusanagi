const K8sServices = {
    init() {
        console.log('🌐 K8s Network Module Initialized');
    },

    // === SERVICES ===
    async fetchServices() {
        const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
        const container = document.getElementById('services-content');

        // Skip if we're definitely on a different tab (but allow if activeTab is undefined/unknown)
        if (activeTab && activeTab !== 'services') {
            console.log('⏭️ Skipping services fetch - not on services tab (current: ' + activeTab + ')');
            return;
        }

        // Check TTL unless container shows loading or error
        const now = Date.now();
        const isStale = !container || container.innerHTML.includes('Loading') || container.innerHTML.includes('Error');

        if (K8sState.lastServicesFetch !== 0 && !isStale && (now - K8sState.lastServicesFetch < K8sState.SERVICES_INGRESS_TTL)) {
            console.log('⏭️ Services fetch skipped - TTL not expired');
            return;
        }

        // Try to load from cache first for instant display
        const cached = K8sState.loadFromCache('kusanagi_services_cache', K8sState.SERVICES_INGRESS_TTL);
        if (cached && cached.length > 0) {
            console.log('📋 Displaying cached services:', cached.length);
            this.renderServicesData(cached);
        } else if (container && !container.innerHTML.includes('Loading')) {
            container.innerHTML = '<div class="loading">Loading services...</div>';
        }

        try {
            console.log('🌐 Fetching services from API...');
            const data = await api.get('/api/services');
            console.log('✅ Services received:', data.length);

            K8sState.saveToCache('kusanagi_services_cache', data);
            K8sState.lastServicesFetch = Date.now();

            this.renderServicesData(data);
        } catch (error) {
            console.error('❌ Error fetching services:', error);
            if (container && !cached) {
                container.innerHTML = `
                    <div class="error-state" style="padding: 2rem; text-align: center;">
                        <span style="font-size: 2rem;">⚠️</span>
                        <p style="color: #ff4444;">Failed to load services</p>
                        <p style="color: var(--text-secondary); font-size: 0.9rem;">${error.message}</p>
                        <button onclick="K8sServices.fetchServices()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                    </div>
                `;
            }
        }
    },

    renderServicesData(services) {
        console.log('🔍 renderServicesData called with', services?.length, 'services');
        
        if (!services || !Array.isArray(services)) {
            console.error('❌ Invalid services data:', services);
            return;
        }
        
        const countEl = document.getElementById('services-count');
        if (countEl) countEl.textContent = services.length;

        try {
            if (window.TableManager) {
                console.log('🔍 Using TableManager for rendering');
                TableManager.init('services', services, (svc) => this.renderServicesRows(svc), [
                    { key: 'name', label: 'Name' }, { key: 'namespace', label: 'Namespace' }, { key: 'type_', label: 'Type' },
                    { key: 'cluster_ip', label: 'Cluster IP' }, { key: 'external_ip', label: 'External IP' }, { key: 'ports', label: 'Ports' }, { key: 'age', label: 'Age' }
                ]);
                this.renderServicesStructure();
            } else {
                console.log('🔍 Using renderServicesSimple (TableManager not available)');
                this.renderServicesSimple(services);
            }
        } catch (e) {
            console.error('❌ Error in renderServicesData:', e);
            const container = document.getElementById('services-content');
            if (container) {
                container.innerHTML = `<div class="error-state">Render error: ${e.message}</div>`;
            }
        }
    },

    renderServicesSimple(services) {
        console.log('🔍 renderServicesSimple called, services:', services?.length);
        const container = document.getElementById('services-content');
        if (!container) {
            console.error('❌ services-content container not found!');
            return;
        }

        if (!services || services.length === 0) {
            console.log('🔍 No services to render');
            container.innerHTML = '<div class="no-issues">No services found</div>';
            return;
        }
        console.log('🔍 Rendering', services.length, 'services to table');

        try {
            const html = `
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Name</th><th>Namespace</th><th>Type</th>
                            <th>Cluster IP</th><th>External IP</th><th>Ports</th><th>Age</th>
                        </tr>
                    </thead>
                    <tbody>
                        ${services.map(svc => `
                            <tr>
                                <td class="app-name">${(svc.name || '-').toString().replace(/</g, '&lt;')}</td>
                                <td>${(svc.namespace || '-').toString().replace(/</g, '&lt;')}</td>
                                <td>${(svc.type_ || '-').toString().replace(/</g, '&lt;')}</td>
                                <td>${(svc.cluster_ip || '-').toString().replace(/</g, '&lt;')}</td>
                                <td>${(svc.external_ip || '-').toString().replace(/</g, '&lt;')}</td>
                                <td>${(svc.ports || '-').toString().replace(/</g, '&lt;')}</td>
                                <td>${(svc.age || '-').toString().replace(/</g, '&lt;')}</td>
                            </tr>
                        `).join('')}
                    </tbody>
                </table>
            `;
            container.innerHTML = html;
            console.log('✅ Table rendered successfully');
        } catch (e) {
            console.error('❌ Error rendering table:', e);
            container.innerHTML = `<div class="error-state">Error rendering table: ${e.message}</div>`;
        }
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
        if (!tbody) return;

        if (!services || services.length === 0) {
            tbody.innerHTML = '<tr><td colspan="7" style="text-align: center; padding: 2rem;">No services found</td></tr>';
            return;
        }

        tbody.innerHTML = services.map(svc => `
            <tr>
                <td class="app-name">${svc.name || '-'}</td>
                <td>${svc.namespace || '-'}</td>
                <td>${svc.type_ || '-'}</td>
                <td>${svc.cluster_ip || '-'}</td>
                <td>${svc.external_ip || '-'}</td>
                <td>${svc.ports || '-'}</td>
                <td>${svc.age || '-'}</td>
            </tr>
        `).join('');
    },

    // === INGRESS ===
    async fetchIngress() {
        const activeTab = window.KusanagiDashboard ? window.KusanagiDashboard.activeTab : null;
        const container = document.getElementById('ingress-content');

        // Skip if we're definitely on a different tab (but allow if activeTab is undefined/unknown)
        if (activeTab && activeTab !== 'ingress') {
            console.log('⏭️ Skipping ingress fetch - not on ingress tab (current: ' + activeTab + ')');
            return;
        }

        // Check TTL unless container shows loading or error
        const now = Date.now();
        const isStale = !container || container.innerHTML.includes('Loading') || container.innerHTML.includes('Error');

        if (K8sState.lastIngressFetch !== 0 && !isStale && (now - K8sState.lastIngressFetch < K8sState.SERVICES_INGRESS_TTL)) {
            console.log('⏭️ Ingress fetch skipped - TTL not expired');
            return;
        }

        // Try to load from cache first for instant display
        const cached = K8sState.loadFromCache('kusanagi_ingress_cache', K8sState.SERVICES_INGRESS_TTL);
        if (cached && cached.length > 0) {
            console.log('🌐 Displaying cached ingress:', cached.length);
            const countEl = document.getElementById('ingress-count');
            if (countEl) countEl.textContent = cached.length;
            this.renderIngressTable(cached);
        } else if (container && !container.innerHTML.includes('Loading')) {
            container.innerHTML = '<div class="loading">Loading ingress...</div>';
        }

        try {
            console.log('🌐 Fetching ingress from API...');
            const data = await api.get('/api/ingress');
            console.log('✅ Ingress received:', data.length);

            K8sState.saveToCache('kusanagi_ingress_cache', data);
            K8sState.lastIngressFetch = Date.now();

            const countEl = document.getElementById('ingress-count');
            if (countEl) countEl.textContent = data.length;
            this.renderIngressTable(data);
        } catch (error) {
            console.error('❌ Error fetching ingress:', error);
            if (container && !cached) {
                container.innerHTML = `
                    <div class="error-state" style="padding: 2rem; text-align: center;">
                        <span style="font-size: 2rem;">⚠️</span>
                        <p style="color: #ff4444;">Failed to load ingress</p>
                        <p style="color: var(--text-secondary); font-size: 0.9rem;">${error.message}</p>
                        <button onclick="K8sServices.fetchIngress()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                    </div>
                `;
            }
        }
    },

    renderIngressTable(ingresses) {
        const container = document.getElementById('ingress-content');
        if (!container) return;

        if (!ingresses || ingresses.length === 0) {
            container.innerHTML = '<div class="no-issues" style="padding: 2rem; text-align: center;">No ingress rules found</div>';
            return;
        }

        container.innerHTML = `
            <table class="data-table">
                <thead>
                    <tr>
                        <th>Name</th><th>Namespace</th><th>Rules</th><th>Age</th>
                    </tr>
                </thead>
                <tbody>
                    ${ingresses.map(ing => `
                        <tr>
                            <td class="app-name">${ing.name || '-'}</td>
                            <td>${ing.namespace || '-'}</td>
                            <td>${Array.isArray(ing.rules) ? ing.rules.join(', ') : (ing.rules || '-')}</td>
                            <td>${ing.age || '-'}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;
    }
};

window.K8sServices = K8sServices;
