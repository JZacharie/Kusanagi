/**
 * Kusanagi Network Visualization Module
 * D3.js-based network flow visualization for Cilium/Hubble data
 */

const KusanagiNetwork = {
    config: {
        namespacesEndpoint: '/api/cilium/namespaces',
        flowsEndpoint: '/api/cilium/flows',
        matrixEndpoint: '/api/cilium/matrix',
        metricsEndpoint: '/api/cilium/metrics',
        anomaliesEndpoint: '/api/cilium/anomalies',
        exportEndpoint: '/api/cilium/export',
        refreshInterval: 30000,
        width: 800,
        height: 600,
        defaultNamespace: null
    },

    state: {
        flows: null,
        matrix: null,
        metrics: null,
        namespaces: [],
        selectedNamespace: 'default',
        intervalId: null,
        performanceHistory: []  // Track performance over time
    },

    // Performance tracking for APM
    perf: {
        lastFetchDuration: 0,
        lastParseDuration: 0,
        lastRenderDuration: 0,
        avgFetchDuration: 0,
        requestCount: 0
    },

    /**
     * Initialize network visualization
     * Note: Polling is handled by TabManager (tab-aware)
     */
    async init(containerId = 'network-visualization') {
        this.container = document.getElementById(containerId);
        if (!this.container) {
            console.warn('Network visualization container not found');
            return;
        }

        this.setupSVG();

        // Pre-fetch namespaces first for better performance
        await this.fetchNamespaces();
        this.populateNamespaceFilter();

        // Start with all namespaces selected
        this.state.selectedNamespace = this.config.defaultNamespace;

        console.log('✅ Network visualization initialized (no internal polling)');
    },

    // Alias pour TabManager
    fetchNetworkData() {
        return this.fetchAndRender();
    },

    /**
     * Setup SVG canvas for D3.js
     */
    setupSVG() {
        this.svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
        this.svg.setAttribute('width', '100%');
        this.svg.setAttribute('height', this.config.height);
        this.svg.setAttribute('class', 'network-graph');
        this.container.innerHTML = '';
        this.container.appendChild(this.svg);
    },

    /**
     * Fetch namespaces from API (pre-filter for performance)
     */
    async fetchNamespaces() {
        try {
            const namespaces = await api.get(this.config.namespacesEndpoint);
            this.state.namespaces = namespaces;
            console.log(`Loaded ${namespaces.length} namespaces from K8s`);
            return namespaces;
        } catch (error) {
            console.error('Failed to fetch namespaces:', error);
            // Fallback to default list
            this.state.namespaces = ['default', 'kube-system'];
            return this.state.namespaces;
        }
    },

    /**
     * Fetch flows data from API with detailed APM tracking
     */
    async fetchFlows(namespace = null) {
        const startTime = performance.now();
        const markName = `network_flows_fetch_${Date.now()}`;
        performance.mark(`${markName}_start`);

        try {
            const url = namespace
                ? `${this.config.flowsEndpoint}?namespace=${encodeURIComponent(namespace)}`
                : this.config.flowsEndpoint;

            console.log(`🌐 Fetching flows from: ${url}`);

            // Fetch phase
            const fetchStart = performance.now();
            const data = await api.get(url);
            const fetchDuration = performance.now() - fetchStart;
            performance.mark(`${markName}_fetch_end`);

            // Parse phase (data already unwrapped by api.get)
            const parseStart = performance.now();
            const parseDuration = performance.now() - parseStart;
            performance.mark(`${markName}_parse_end`);

            // Debug logging
            if (window.KusanagiDebug) {
                KusanagiDebug.logApiResponse('/api/cilium/flows', data, fetchDuration + parseDuration);
                KusanagiDebug.validateNetworkData(data);
            }

            this.state.flows = data;

            // Update performance metrics
            const totalDuration = performance.now() - startTime;
            this.perf.lastFetchDuration = fetchDuration;
            this.perf.lastParseDuration = parseDuration;
            this.perf.requestCount++;
            this.perf.avgFetchDuration = (
                (this.perf.avgFetchDuration * (this.perf.requestCount - 1)) + totalDuration
            ) / this.perf.requestCount;

            // Store in history (keep last 20)
            this.state.performanceHistory.push({
                timestamp: new Date().toISOString(),
                fetchMs: fetchDuration,
                parseMs: parseDuration,
                totalMs: totalDuration,
                flowsCount: data.flows?.length || 0
            });
            if (this.state.performanceHistory.length > 20) {
                this.state.performanceHistory.shift();
            }

            // Track detailed metrics for RUM/APM
            if (window.KusanagiRUM) {
                window.KusanagiRUM.track('network_flows_performance', {
                    fetch_duration_ms: Math.round(fetchDuration),
                    parse_duration_ms: Math.round(parseDuration),
                    total_duration_ms: Math.round(totalDuration),
                    flows_count: data.flows?.length || 0,
                    matrix_count: data.matrix?.length || 0,
                    namespace: namespace || 'all',
                    avg_duration_ms: Math.round(this.perf.avgFetchDuration)
                });

                // Also track as standard API call
                window.KusanagiRUM.trackApiCall(this.config.flowsEndpoint, totalDuration, true, 200);
            }

            console.log(`⏱️ Network flows: fetch=${fetchDuration.toFixed(0)}ms, parse=${parseDuration.toFixed(0)}ms, total=${totalDuration.toFixed(0)}ms, flows=${data.flows?.length || 0}`);

            return data;
        } catch (error) {
            const errorDuration = performance.now() - startTime;
            console.error('Failed to fetch network flows:', error);

            if (window.KusanagiRUM) {
                window.KusanagiRUM.track('network_flows_error', {
                    error: error.message,
                    duration_ms: Math.round(errorDuration),
                    namespace: namespace || 'all'
                });
                window.KusanagiRUM.trackApiCall(this.config.flowsEndpoint, errorDuration, false);
            }
            throw error;
        }
    },

    /**
     * Fetch flow matrix
     */
    async fetchMatrix(namespace = null) {
        try {
            const url = namespace
                ? `${this.config.matrixEndpoint}?namespace=${encodeURIComponent(namespace)}`
                : this.config.matrixEndpoint;

            const data = await api.get(url);
            this.state.matrix = data;
            return data;
        } catch (error) {
            console.error('Failed to fetch flow matrix:', error);
            throw error;
        }
    },

    /**
     * Fetch and render all data with performance tracking
     */
    async fetchAndRender() {
        const totalStart = performance.now();

        // Auto-initialize if init() was never called (e.g. direct URL hash navigation)
        if (!this.container || !this.svg) {
            await this.init();
        }

        try {
            const namespace = this.state.selectedNamespace;

            // Parallel fetch phase
            const fetchStart = performance.now();
            await Promise.all([
                this.fetchFlows(namespace),
                this.fetchMatrix(namespace)
            ]);
            const fetchDuration = performance.now() - fetchStart;

            // Render phase
            const renderStart = performance.now();
            this.populateNamespaceFilter();
            this.renderGraph();
            this.renderMatrix();
            this.renderStats();
            this.renderPerformanceStats();  // New: Show performance metrics
            const renderDuration = performance.now() - renderStart;

            this.perf.lastRenderDuration = renderDuration;

            const totalDuration = performance.now() - totalStart;
            console.log(`🌐 Network render complete: fetch=${fetchDuration.toFixed(0)}ms, render=${renderDuration.toFixed(0)}ms, total=${totalDuration.toFixed(0)}ms`);

            // Track full cycle performance
            if (window.KusanagiRUM) {
                window.KusanagiRUM.track('network_render_cycle', {
                    fetch_duration_ms: Math.round(fetchDuration),
                    render_duration_ms: Math.round(renderDuration),
                    total_duration_ms: Math.round(totalDuration),
                    namespace: namespace || 'all'
                });
            }
        } catch (error) {
            this.renderError(error.message);
        }
    },

    /**
     * Render performance statistics panel
     */
    renderPerformanceStats() {
        const container = document.getElementById('network-perf-stats');
        if (!container) return;

        const history = this.state.performanceHistory;
        const avgFetch = history.length > 0
            ? history.reduce((sum, h) => sum + h.fetchMs, 0) / history.length
            : 0;
        const avgParse = history.length > 0
            ? history.reduce((sum, h) => sum + h.parseMs, 0) / history.length
            : 0;
        const avgTotal = history.length > 0
            ? history.reduce((sum, h) => sum + h.totalMs, 0) / history.length
            : 0;

        // Determine health status
        const healthClass = avgTotal < 500 ? 'healthy' : avgTotal < 2000 ? 'warning' : 'error';
        const healthIcon = avgTotal < 500 ? '✅' : avgTotal < 2000 ? '⚠️' : '🔴';

        container.innerHTML = `
            <div class="perf-stats-panel">
                <div class="perf-header">
                    <span class="perf-icon">⏱️</span>
                    <span class="perf-title">APM - Network Flows</span>
                    <span class="perf-health ${healthClass}">${healthIcon} ${avgTotal.toFixed(0)}ms avg</span>
                </div>
                <div class="perf-metrics">
                    <div class="perf-metric">
                        <span class="metric-label">Fetch</span>
                        <span class="metric-value">${avgFetch.toFixed(0)}ms</span>
                    </div>
                    <div class="perf-metric">
                        <span class="metric-label">Parse</span>
                        <span class="metric-value">${avgParse.toFixed(0)}ms</span>
                    </div>
                    <div class="perf-metric">
                        <span class="metric-label">Render</span>
                        <span class="metric-value">${this.perf.lastRenderDuration.toFixed(0)}ms</span>
                    </div>
                    <div class="perf-metric">
                        <span class="metric-label">Requests</span>
                        <span class="metric-value">${this.perf.requestCount}</span>
                    </div>
                </div>
                ${this.renderSparkline(history)}
            </div>
        `;
    },

    /**
     * Render mini sparkline chart for performance history
     */
    renderSparkline(history) {
        if (history.length < 2) return '';

        const width = 200;
        const height = 30;
        const max = Math.max(...history.map(h => h.totalMs), 100);
        const points = history.map((h, i) => {
            const x = (i / (history.length - 1)) * width;
            const y = height - (h.totalMs / max) * height;
            return `${x},${y}`;
        }).join(' ');

        return `
            <div class="perf-sparkline">
                <svg width="${width}" height="${height}" class="sparkline-svg">
                    <polyline points="${points}" fill="none" stroke="#ff00ff" stroke-width="2"/>
                </svg>
            </div>
        `;
    },

    /**
     * Render network graph using D3.js-style SVG
     */
    renderGraph() {
        // Debug logging
        if (window.KusanagiDebug) {
            console.log('🌐 Rendering network graph, flows:', this.state.flows);
        }

        if (!this.state.flows || !this.state.flows.flows) {
            console.warn('No flows data available to render');
            this.renderError('No network flow data available');
            return;
        }

        const flows = this.state.flows.flows;

        if (!Array.isArray(flows) || flows.length === 0) {
            console.warn('Flows array is empty');
            this.renderError('No network flows to display');
            return;
        }
        const width = this.container.clientWidth || this.config.width;
        const height = this.config.height;

        // Clear SVG
        this.svg.innerHTML = '';

        // Build nodes and links from flows
        const nodesMap = new Map();
        const links = [];

        flows.forEach(flow => {
            const sourceId = `${flow.source_namespace}/${flow.source_pod}`;
            const targetId = `${flow.destination_namespace}/${flow.destination_pod}`;

            if (!nodesMap.has(sourceId)) {
                nodesMap.set(sourceId, {
                    id: sourceId,
                    namespace: flow.source_namespace,
                    pod: flow.source_pod,
                    type: 'source'
                });
            }

            if (!nodesMap.has(targetId)) {
                nodesMap.set(targetId, {
                    id: targetId,
                    namespace: flow.destination_namespace,
                    pod: flow.destination_pod,
                    type: 'destination'
                });
            }

            links.push({
                source: sourceId,
                target: targetId,
                protocol: flow.protocol,
                port: flow.destination_port,
                bytes: flow.bytes_sent,
                verdict: flow.verdict
            });
        });

        const nodes = Array.from(nodesMap.values());

        // Simple force layout simulation (manual positioning)
        const centerX = width / 2;
        const centerY = height / 2;
        const radius = Math.min(width, height) / 3;

        nodes.forEach((node, i) => {
            const angle = (2 * Math.PI * i) / nodes.length;
            node.x = centerX + radius * Math.cos(angle);
            node.y = centerY + radius * Math.sin(angle);
        });

        // Render links
        const linksGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        linksGroup.setAttribute('class', 'links');

        links.forEach(link => {
            const source = nodesMap.get(link.source);
            const target = nodesMap.get(link.target);
            if (!source || !target) return;

            const isDropped = link.verdict === 'DROPPED';
            const strokeColor = isDropped ? '#ff0080' : '#00fff9'; // neon-magenta vs neon-cyan

            const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
            line.setAttribute('x1', source.x);
            line.setAttribute('y1', source.y);
            line.setAttribute('x2', target.x);
            line.setAttribute('y2', target.y);
            line.setAttribute('stroke', strokeColor);
            line.setAttribute('stroke-opacity', '0.7');
            line.setAttribute('stroke-width', Math.max(1.5, Math.log(link.bytes / 100) || 1.5));
            line.setAttribute('stroke-dasharray', isDropped ? '6,3' : 'none');

            // Animated glow arrow
            const title = document.createElementNS('http://www.w3.org/2000/svg', 'title');
            title.textContent = `${link.source} → ${link.target} | ${link.protocol}:${link.port} (${this.formatBytes(link.bytes)})`;
            line.appendChild(title);

            linksGroup.appendChild(line);
        });
        this.svg.appendChild(linksGroup);

        // Render nodes
        const nodesGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        nodesGroup.setAttribute('class', 'nodes');

        nodes.forEach(node => {
            const group = document.createElementNS('http://www.w3.org/2000/svg', 'g');
            group.setAttribute('class', 'node');
            group.setAttribute('transform', `translate(${node.x}, ${node.y})`);

            const nsColor = this.getNamespaceColor(node.namespace);

            // Outer glow ring
            const glow = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            glow.setAttribute('r', 24);
            glow.setAttribute('fill', 'none');
            glow.setAttribute('stroke', nsColor);
            glow.setAttribute('stroke-width', '1');
            glow.setAttribute('stroke-opacity', '0.4');
            group.appendChild(glow);

            // Node circle
            const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            circle.setAttribute('r', 20);
            circle.setAttribute('fill', nsColor);
            circle.setAttribute('fill-opacity', '0.85');
            circle.setAttribute('stroke', nsColor);
            circle.setAttribute('stroke-width', '2');
            group.appendChild(circle);

            // Namespace label (above circle)
            const nsLabel = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            nsLabel.setAttribute('dy', -26);
            nsLabel.setAttribute('text-anchor', 'middle');
            nsLabel.setAttribute('fill', nsColor);
            nsLabel.setAttribute('font-size', '9');
            nsLabel.setAttribute('font-family', 'monospace');
            nsLabel.setAttribute('opacity', '0.8');
            nsLabel.textContent = node.namespace;
            group.appendChild(nsLabel);

            // Pod name label (below circle)
            const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            text.setAttribute('dy', 36);
            text.setAttribute('text-anchor', 'middle');
            text.setAttribute('fill', '#e0e0e0');
            text.setAttribute('font-size', '11');
            text.setAttribute('font-family', 'monospace');
            text.setAttribute('font-weight', 'bold');
            text.textContent = node.pod.length > 15 ? node.pod.substring(0, 12) + '…' : node.pod;
            group.appendChild(text);

            nodesGroup.appendChild(group);
        });
        this.svg.appendChild(nodesGroup);

        // Add legend
        this.renderLegend(Array.from(new Set(nodes.map(n => n.namespace))));
    },

    /**
     * Render color legend
     */
    renderLegend(namespaces) {
        const legendGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        legendGroup.setAttribute('class', 'legend');
        legendGroup.setAttribute('transform', 'translate(12, 16)');

        // Legend background
        const bg = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
        bg.setAttribute('x', -6);
        bg.setAttribute('y', -4);
        bg.setAttribute('width', 130);
        bg.setAttribute('height', namespaces.length * 20 + 8);
        bg.setAttribute('fill', 'rgba(0,0,0,0.55)');
        bg.setAttribute('rx', 4);
        legendGroup.appendChild(bg);

        namespaces.forEach((ns, i) => {
            const item = document.createElementNS('http://www.w3.org/2000/svg', 'g');
            item.setAttribute('transform', `translate(0, ${i * 20})`);

            const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
            rect.setAttribute('width', 10);
            rect.setAttribute('height', 10);
            rect.setAttribute('rx', 2);
            rect.setAttribute('fill', this.getNamespaceColor(ns));
            item.appendChild(rect);

            const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            text.setAttribute('x', 16);
            text.setAttribute('y', 9);
            text.setAttribute('fill', '#c0c0c0');
            text.setAttribute('font-size', '10');
            text.setAttribute('font-family', 'monospace');
            text.textContent = ns;
            item.appendChild(text);

            legendGroup.appendChild(item);
        });

        this.svg.appendChild(legendGroup);
    },

    /**
     * Render flow matrix table
     */
    renderMatrix() {
        const matrixContainer = document.getElementById('network-matrix');
        if (!matrixContainer) {
            console.warn('Network matrix container not found');
            return;
        }

        if (!this.state.matrix || !Array.isArray(this.state.matrix)) {
            matrixContainer.innerHTML = '<div class="loading">No matrix data available</div>';
            return;
        }

        const matrix = this.state.matrix;

        if (matrix.length === 0) {
            matrixContainer.innerHTML = '<div class="no-data">No flow matrix entries to display</div>';
            return;
        }

        let html = `
            <table class="data-table network-matrix-table">
                <thead>
                    <tr>
                        <th>Source</th>
                        <th>Destination</th>
                        <th>Protocol</th>
                        <th>Port</th>
                        <th>Flows</th>
                        <th>Bytes</th>
                        <th>Verdict</th>
                    </tr>
                </thead>
                <tbody>
        `;

        matrix.forEach(entry => {
            const verdictClass = entry.verdict === 'FORWARDED' ? 'status-healthy' : 'status-degraded';
            html += `
                <tr>
                    <td><code>${entry.source}</code></td>
                    <td><code>${entry.destination}</code></td>
                    <td>${entry.protocol}</td>
                    <td>${entry.port}</td>
                    <td>${entry.flow_count}</td>
                    <td>${this.formatBytes(entry.bytes_total)}</td>
                    <td><span class="status-badge ${verdictClass}">${entry.verdict}</span></td>
                </tr>
            `;
        });

        html += '</tbody></table>';
        matrixContainer.innerHTML = html;
    },

    /**
     * Render network stats
     */
    renderStats() {
        const statsContainer = document.getElementById('network-stats');
        if (!statsContainer || !this.state.flows) {
            console.warn('Cannot render stats: container or flows data missing');
            return;
        }

        const flows = this.state.flows;
        const flowsArray = Array.isArray(flows.flows) ? flows.flows : [];

        const totalBytes = flowsArray.reduce((sum, f) => sum + (f.bytes_sent || 0) + (f.bytes_received || 0), 0);
        const forwarded = flowsArray.filter(f => f.verdict === 'FORWARDED').length;
        const dropped = flowsArray.filter(f => f.verdict === 'DROPPED').length;

        statsContainer.innerHTML = `
            <div class="network-stats-grid">
                <div class="stat-card">
                    <span class="stat-value">${flows.total_flows}</span>
                    <span class="stat-label">Total Flows</span>
                </div>
                <div class="stat-card">
                    <span class="stat-value">${this.formatBytes(totalBytes)}</span>
                    <span class="stat-label">Total Traffic</span>
                </div>
                <div class="stat-card">
                    <span class="stat-value healthy">${forwarded}</span>
                    <span class="stat-label">Forwarded</span>
                </div>
                <div class="stat-card">
                    <span class="stat-value ${dropped > 0 ? 'error' : ''}">${dropped}</span>
                    <span class="stat-label">Dropped</span>
                </div>
                <div class="stat-card">
                    <span class="stat-value">${flows.namespaces.length}</span>
                    <span class="stat-label">Namespaces</span>
                </div>
            </div>
        `;
    },

    /**
     * Render error state
     */
    renderError(message) {
        if (this.container) {
            this.container.innerHTML = `
                <div class="error-state">
                    <span class="error-icon">⚠️</span>
                    <p>Failed to load network data</p>
                    <code>${message}</code>
                    <button onclick="KusanagiNetwork.fetchAndRender()" class="retry-btn">Retry</button>
                </div>
            `;
        }
    },

    /**
     * Filter by namespace
     */
    filterByNamespace(namespace) {
        this.state.selectedNamespace = namespace || null;
        this.fetchAndRender();
    },

    /**
     * Populate namespace filter dropdown with pre-fetched namespaces
     * Uses namespaces from K8s API for better performance
     */
    populateNamespaceFilter() {
        const select = document.getElementById('network-namespace-filter');
        if (!select) return;

        // Only populate the dropdown once (when empty)
        if (select.options.length <= 1 && this.state.namespaces.length > 0) {
            // Add namespace options from pre-fetched list
            this.state.namespaces.forEach(ns => {
                const option = document.createElement('option');
                option.value = ns;
                option.textContent = ns;
                select.appendChild(option);
            });
        }

        // Always update the selected value to match state
        if (this.state.selectedNamespace) {
            select.value = this.state.selectedNamespace;
        }
    },

    /**
     * Export data
     */
    async exportData(format = 'json') {
        const namespace = this.state.selectedNamespace;
        let url = `${this.config.exportEndpoint}?format=${format}`;
        if (namespace) url += `&namespace=${encodeURIComponent(namespace)}`;

        window.open(url, '_blank');
    },

    /**
     * Start auto-refresh
     */
    startAutoRefresh() {
        if (this.state.intervalId) clearInterval(this.state.intervalId);
        this.state.intervalId = setInterval(() => {
            this.fetchAndRender();
        }, this.config.refreshInterval);
    },

    /**
     * Stop auto-refresh
     */
    stopAutoRefresh() {
        if (this.state.intervalId) {
            clearInterval(this.state.intervalId);
            this.state.intervalId = null;
        }
    },

    /**
     * Get color for namespace
     */
    getNamespaceColor(namespace) {
        const colors = {
            'kube-system': '#ff6b6b',
            'argocd': '#4ecdc4',
            'monitoring': '#45b7d1',
            'kusanagi': '#ff00ff',
            'default': '#96ceb4',
            'minio': '#ffeaa7',
            'n8n': '#dfe6e9',
            'paperless': '#74b9ff'
        };
        return colors[namespace] || '#95a5a6';
    },

    /**
     * Format bytes to human readable
     */
    formatBytes(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }
};

// Export for global access
if (typeof window !== 'undefined') {
    window.KusanagiNetwork = KusanagiNetwork;
}
