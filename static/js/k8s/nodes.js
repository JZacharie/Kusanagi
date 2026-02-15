const K8sNodes = {
    init() {
        console.log('🖥️ K8s Nodes Module Initialized');
    },

    async fetchNodesStatus() {
        try {
            const data = await api.get('/api/k8s/nodes');

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
                <div class="no-issues" style="padding: 2rem; text-align: center; grid-column: 1 / -1;">
                    <span style="font-size: 2rem;">🖥️</span>
                    <p>No nodes found</p>
                    ${warningMsg ? `<p style="color: var(--neon-orange); margin-top: 1rem; font-size: 0.9rem;">⚠️ ${warningMsg}</p>` : ''}
                    <button onclick="K8sNodes.fetchNodesStatus()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                </div>
            `;
            return;
        }

        nodes.sort((a, b) => a.name.localeCompare(b.name));

        // Mapping des conditions vers des icônes
        const conditionIcons = {
            'Ready': '✓',
            'DiskPressure': '💾',
            'MemoryPressure': '🧠',
            'NetworkUnavailable': '🌐',
            'PIDPressure': '⚡'
        };

        const archIcons = {
            'amd64': '<i class="mdi mdi-cpu-64-bit" title="x86_64 / amd64"></i>',
            'arm64': '<i class="mdi mdi-chip" title="ARM64"></i>'
        };


        container.innerHTML = nodes.map((node) => {
            const isReady = node.status === 'Ready';
            const cpuPercent = parseFloat(node.cpu_usage_percent) || 0;
            const memPercent = parseFloat(node.memory_usage_percent) || 0;
            const podCount = parseInt(node.pod_count) || 0;
            const podCapacity = parseInt(node.pod_capacity) || 0;
            const podPercent = podCapacity ? Math.round((podCount / podCapacity) * 100) : 0;

            // Déterminer la couleur pour chaque métrique
            const getColorClass = (p) => p > 90 ? 'high' : p > 75 ? 'medium' : 'low';
            const getColorValue = (p) => p > 90 ? '#ef4444' : p > 75 ? '#f59e0b' : '#22c55e';

            const cpuColorClass = getColorClass(cpuPercent);
            const memColorClass = getColorClass(memPercent);
            const podColorClass = getColorClass(podPercent);

            // Rendu des conditions avec icônes
            const renderConditions = () => {
                if (!node.conditions) return '';

                return Object.entries(node.conditions).map(([condition, status]) => {
                    const icon = conditionIcons[condition] || '●';
                    const isTrue = status === 'True';
                    // Pour Ready: true = bon (vert), false = mauvais (gris)
                    // Pour les autres (Pressure): false = bon (vert), true = mauvais (rouge)
                    const isPositive = condition === 'Ready' ? isTrue : !isTrue;
                    const statusClass = isTrue ? 'true' : 'false';

                    return `
                        <div class="condition-icon ${condition} ${statusClass}" 
                             data-condition="${condition}" 
                             data-status="${status}"
                             title="${condition}: ${status}">
                            ${icon}
                        </div>
                    `;
                }).join('');
            };

            // Format compact des infos système
            const arch = node.architecture || 'N/A';
            const os = node.os ? node.os.split(' ')[0] : 'N/A';
            const kubelet = node.kubelet_version ? node.kubelet_version.replace('v', '').split('+')[0] : 'N/A';
            const cpuCapacity = node.cpu_capacity || '-';

            return `
                <div class="node-card ${isReady ? 'ready' : 'not-ready'}">
                    <!-- Header -->
                    <div class="node-header">
                        <div class="node-name">${node.name}</div>
                        <div class="node-status ${isReady ? 'ready' : 'not-ready'}">${node.status}</div>
                    </div>

                    <!-- Info compacte -->
                    <div class="node-info-compact">
                        <span><span class="label">Arch:</span> <span class="value">${archIcons[arch.toLowerCase()] || ''}${arch}</span></span>
                        <span><span class="label">OS:</span> <span class="value">${os}</span></span>
                        <span><span class="label">Kubelet:</span> <span class="value">${kubelet}</span></span>
                    </div>

                    <!-- Jauges compactes -->
                    <div class="node-gauges">
                        <div class="gauge-mini">
                            <div class="gauge-mini-label">CPU</div>
                            <div class="gauge-mini-value" style="color: ${getColorValue(cpuPercent)}">${cpuPercent.toFixed(0)}%</div>
                            <div class="gauge-mini-bar">
                                <div class="gauge-mini-fill ${cpuColorClass}" style="width: ${Math.min(cpuPercent, 100)}%"></div>
                            </div>
                        </div>
                        <div class="gauge-mini">
                            <div class="gauge-mini-label">Mem</div>
                            <div class="gauge-mini-value" style="color: ${getColorValue(memPercent)}">${memPercent.toFixed(0)}%</div>
                            <div class="gauge-mini-bar">
                                <div class="gauge-mini-fill ${memColorClass}" style="width: ${Math.min(memPercent, 100)}%"></div>
                            </div>
                        </div>
                        <div class="gauge-mini">
                            <div class="gauge-mini-label">Pods</div>
                            <div class="gauge-mini-value" style="color: ${getColorValue(podPercent)}">${podCount}/${podCapacity}</div>
                            <div class="gauge-mini-bar">
                                <div class="gauge-mini-fill ${podColorClass}" style="width: ${Math.min(podPercent, 100)}%"></div>
                            </div>
                        </div>
                    </div>

                    <!-- Age et capacité -->
                    <div class="node-meta-row">
                        <div class="node-age">
                            Age: <span class="node-age-value">${node.age || 'N/A'}</span>
                        </div>
                        <div class="node-pods">
                            ${cpuCapacity} • ${node.memory_allocatable || '-'}
                        </div>
                    </div>

                    <!-- Conditions avec icônes -->
                    <div class="node-conditions">
                        ${renderConditions()}
                    </div>
                </div>
            `;
        }).join('');
    },

    renderNodesError(message) {
        const container = document.getElementById('nodes-container');
        if (!container) return;
        container.innerHTML = `
            <div class="error-state" style="padding: 2rem; text-align: center; grid-column: 1 / -1;">
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
