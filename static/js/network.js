/**
 * Kusanagi Network Visualization Module
 * D3.js-based network flow visualization for Cilium/Hubble data
 */

const KusanagiNetwork = {
    config: {
        flowsEndpoint: '/api/cilium/flows',
        matrixEndpoint: '/api/cilium/matrix',
        metricsEndpoint: '/api/cilium/metrics',
        anomaliesEndpoint: '/api/cilium/anomalies',
        exportEndpoint: '/api/cilium/export',
        refreshInterval: 30000,
        width: 800,
        height: 600
    },

    state: {
        flows: null,
        matrix: null,
        metrics: null,
        selectedNamespace: null,
        intervalId: null
    },

    /**
     * Initialize network visualization
     */
    init(containerId = 'network-visualization') {
        this.container = document.getElementById(containerId);
        if (!this.container) {
            console.warn('Network visualization container not found');
            return;
        }

        this.setupSVG();
        this.fetchAndRender();
        this.startAutoRefresh();
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
     * Fetch flows data from API
     */
    async fetchFlows(namespace = null) {
        const startTime = performance.now();
        try {
            const url = namespace
                ? `${this.config.flowsEndpoint}?namespace=${encodeURIComponent(namespace)}`
                : this.config.flowsEndpoint;

            const response = await fetch(url);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);

            const data = await response.json();
            this.state.flows = data;

            // Track API call for RUM
            if (window.KusanagiRUM) {
                const duration = performance.now() - startTime;
                window.KusanagiRUM.trackApiCall(this.config.flowsEndpoint, duration, true);
            }

            return data;
        } catch (error) {
            console.error('Failed to fetch network flows:', error);
            if (window.KusanagiRUM) {
                window.KusanagiRUM.trackApiCall(this.config.flowsEndpoint, 0, false);
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

            const response = await fetch(url);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);

            const data = await response.json();
            this.state.matrix = data;
            return data;
        } catch (error) {
            console.error('Failed to fetch flow matrix:', error);
            throw error;
        }
    },

    /**
     * Fetch and render all data
     */
    async fetchAndRender() {
        try {
            const namespace = this.state.selectedNamespace;
            await Promise.all([
                this.fetchFlows(namespace),
                this.fetchMatrix(namespace)
            ]);

            this.populateNamespaceFilter();
            this.renderGraph();
            this.renderMatrix();
            this.renderStats();
        } catch (error) {
            this.renderError(error.message);
        }
    },

    /**
     * Render network graph using D3.js-style SVG
     */
    renderGraph() {
        if (!this.state.flows || !this.state.flows.flows) return;

        const flows = this.state.flows.flows;
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

            const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
            line.setAttribute('x1', source.x);
            line.setAttribute('y1', source.y);
            line.setAttribute('x2', target.x);
            line.setAttribute('y2', target.y);
            line.setAttribute('class', `flow-link verdict-${link.verdict.toLowerCase()}`);
            line.setAttribute('stroke-width', Math.max(1, Math.log(link.bytes / 100) || 1));

            // Add tooltip data
            line.dataset.tooltip = `${link.source} → ${link.target}\n${link.protocol}:${link.port} (${this.formatBytes(link.bytes)})`;

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

            // Node circle
            const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            circle.setAttribute('r', 20);
            circle.setAttribute('class', `node-circle ns-${node.namespace}`);
            circle.setAttribute('fill', this.getNamespaceColor(node.namespace));
            group.appendChild(circle);

            // Node label
            const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            text.setAttribute('dy', 35);
            text.setAttribute('text-anchor', 'middle');
            text.setAttribute('class', 'node-label');
            text.textContent = node.pod.length > 15 ? node.pod.substring(0, 12) + '...' : node.pod;
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
        legendGroup.setAttribute('transform', 'translate(10, 10)');

        namespaces.forEach((ns, i) => {
            const item = document.createElementNS('http://www.w3.org/2000/svg', 'g');
            item.setAttribute('transform', `translate(0, ${i * 20})`);

            const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
            rect.setAttribute('width', 12);
            rect.setAttribute('height', 12);
            rect.setAttribute('fill', this.getNamespaceColor(ns));
            item.appendChild(rect);

            const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            text.setAttribute('x', 18);
            text.setAttribute('y', 10);
            text.setAttribute('class', 'legend-text');
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
        if (!matrixContainer || !this.state.matrix) return;

        const matrix = this.state.matrix;

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
        if (!statsContainer || !this.state.flows) return;

        const flows = this.state.flows;
        const totalBytes = flows.flows.reduce((sum, f) => sum + f.bytes_sent + f.bytes_received, 0);
        const forwarded = flows.flows.filter(f => f.verdict === 'FORWARDED').length;
        const dropped = flows.flows.filter(f => f.verdict === 'DROPPED').length;

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
     * Populate namespace filter dropdown with available namespaces
     * Only populates on first load (when no namespace is selected) to preserve the full list
     */
    populateNamespaceFilter() {
        const select = document.getElementById('network-namespace-filter');
        if (!select || !this.state.flows || !this.state.flows.namespaces) return;

        // Only populate the dropdown on first load (when no namespace selected yet)
        // This preserves the full namespace list when filtering
        if (select.options.length <= 1) {
            const namespaces = this.state.flows.namespaces;

            // Add namespace options
            namespaces.sort().forEach(ns => {
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
