const K8sNodes = {
    init() {
        console.log('🖥️ K8s Nodes Module Initialized');
    },

    async fetchNodesStatus() {
        try {
            const data = await api.get('/api/nodes/status');

            if (data._warning) {
                console.warn('Nodes warning:', data._warning);
            }

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
                    <button onclick="K8sNodes.fetchNodesStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                </div>
            `;
            return;
        }

        nodes.sort((a, b) => a.name.localeCompare(b.name));

        container.innerHTML = nodes.map((node) => {
            const isReady = node.status === 'Ready';
            const cpuPercent = parseFloat(node.cpu_usage_percent) || 0;
            const memPercent = parseFloat(node.memory_usage_percent) || 0;
            const podCount = parseInt(node.pod_count) || 0;
            const podCapacity = parseInt(node.pod_capacity) || 0;
            const podPercent = podCapacity ? Math.round((podCount / podCapacity) * 100) : 0;

            const getColor = (p) => p > 90 ? '#ef4444' : p > 75 ? '#f59e0b' : '#22c55e';
            const getCpuColor = getColor(cpuPercent);
            const getMemColor = getColor(memPercent);
            const getPodColor = getColor(podPercent);

            const renderGauge = (percent, label, sublabel, color) => {
                // ... gauge SVG logic ...
                // Simplified for brevity, same as original
                const radius = 28;
                const stroke = 6;
                const normR = radius - stroke * 2;
                const circ = normR * 2 * Math.PI;
                const dash = circ - (percent / 100) * circ;

                return `
                    <div class="gauge-item" style="display: flex; flex-direction: column; align-items: center; gap: 4px;">
                        <div class="gauge-svg" style="position: relative; width: 60px; height: 60px;">
                            <svg height="60" width="60" style="transform: rotate(-90deg);">
                                <circle stroke="rgba(255,255,255,0.1)" stroke-width="${stroke}" fill="transparent" r="${normR}" cx="30" cy="30" />
                                <circle stroke="${color}" stroke-width="${stroke}" stroke-dasharray="${circ} ${circ}" style="stroke-dashoffset: ${dash}; transition: stroke-dashoffset 0.5s ease; filter: drop-shadow(0 0 4px ${color});" stroke-linecap="round" fill="transparent" r="${normR}" cx="30" cy="30" />
                            </svg>
                            <div class="gauge-value" style="position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); font-size: 0.85rem; font-weight: bold; color: ${color}; font-family: 'JetBrains Mono', monospace;">
                                ${percent.toFixed(0)}%
                            </div>
                        </div>
                        <div class="gauge-label" style="font-size: 0.7rem; text-transform: uppercase;">${label}</div>
                        <div class="gauge-sublabel" style="font-size: 0.65rem; color: var(--text-muted);">${sublabel}</div>
                    </div>
                `;
            };

            return `
                <div class="node-card ${isReady ? 'ready' : 'not-ready'}">
                    <div class="node-header">
                        <div class="node-name">${node.name}</div>
                        <div class="node-status ${isReady ? 'ready' : 'not-ready'}">${node.status}</div>
                    </div>
                    <div class="node-info-grid">
                        <div class="node-info-item"><div class="node-info-label">Arch</div><div class="node-info-value">${node.architecture || 'N/A'}</div></div>
                        <div class="node-info-item"><div class="node-info-label">OS</div><div class="node-info-value">${node.os || 'N/A'}</div></div>
                        <div class="node-info-item"><div class="node-info-label">Kernel</div><div class="node-info-value">${node.kernel_version || 'N/A'}</div></div>
                        <div class="node-info-item"><div class="node-info-label">Kubelet</div><div class="node-info-value">${node.kubelet_version || 'N/A'}</div></div>
                    </div>
                    <div class="node-resources-gauges" style="display: flex; justify-content: space-around; padding: 1.5rem 0; background: rgba(0,0,0,0.2); border-radius: 12px; margin: 1rem 0;">
                        ${renderGauge(cpuPercent, 'CPU', node.cpu_capacity || '-', getCpuColor)}
                        ${renderGauge(memPercent, 'Memory', node.memory_allocatable || '-', getMemColor)}
                        ${renderGauge(podPercent, 'Pods', `${podCount}/${podCapacity}`, getPodColor)}
                    </div>
                    <div class="node-age-bar" style="text-align: center; padding: 8px;">
                        <span style="font-size: 0.75rem; color: var(--text-secondary);">Node Age: </span>
                        <span style="font-size: 0.85rem; font-weight: bold; color: var(--neon-cyan); font-family: 'JetBrains Mono', monospace;">${node.age || 'N/A'}</span>
                    </div>
                    ${node.conditions ? `<div class="node-conditions" style="margin-top: 10px; display: flex; gap: 8px; flex-wrap: wrap;">${Object.entries(node.conditions).map(([k, v]) => `<span class="condition-item ${v === 'True' ? 'true' : 'false'}" style="font-size: 0.8em; padding: 2px 4px; background: rgba(255,255,255,0.1); border-radius: 4px;">${k}:${v}</span>`).join('')}</div>` : ''}
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
                <button onclick="K8sNodes.fetchNodesStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
            </div>
        `;
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
            const data = await api.get('/api/debug/nodes');
            let report = '🔍 DIAGNOSTIC REPORT\n====================\n\n';
            report += `[K8s Nodes] ${data.k8s_nodes_ok ? '✅ OK' : '❌ FAIL'}\n[K8s Pods]  ${data.k8s_pods_ok ? '✅ OK' : '❌ FAIL'}\n[Prometheus] ${data.prometheus_ok ? '✅ OK' : '❌ FAIL'}\n`;
            resultDiv.innerHTML = report;
        } catch (e) {
            resultDiv.innerHTML += `\n❌ Critical error: ${e.message}`;
        } finally {
            btn.disabled = false;
            btn.textContent = 'Run Deep Diagnostic';
        }
    }
};

window.K8sNodes = K8sNodes;
