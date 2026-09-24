/**
 * GpuDashboard - Management and monitoring for GPU Nodes (vm168 & vm169)
 * Displays live DCGM telemetry, on-demand workload scaling, and pod actions
 */

const GpuDashboard = {
    data: null,
    searchQuery: '',
    nodeFilter: 'all',

    init() {
        console.log('⚡ GpuDashboard Initialized');
        this.loadData();
    },

    async loadData(forceRefresh = false) {
        const refreshBtn = document.getElementById('btn-gpu-refresh');
        if (refreshBtn) {
            refreshBtn.disabled = true;
            refreshBtn.innerHTML = '<i class="mdi mdi-loading mdi-spin"></i> Loading...';
        }

        try {
            const url = forceRefresh ? '/api/k8s/gpu?refresh=true' : '/api/k8s/gpu';
            const data = await api.get(url);
            this.data = data;
            this.render();
        } catch (error) {
            console.error('Failed to load GPU data:', error);
            const container = document.getElementById('gpu-nodes-container');
            if (container) {
                container.innerHTML = `
                    <div style="grid-column: 1 / -1; padding: 2rem; text-align: center; color: #ff4444; background: rgba(255, 68, 68, 0.1); border: 1px solid #ff4444; border-radius: 8px;">
                        <p>⚠️ Failed to fetch GPU cluster status</p>
                        <p style="font-size: 0.8rem; color: #888;">${error.message}</p>
                        <button onclick="GpuDashboard.loadData(true)" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                    </div>
                `;
            }
        } finally {
            if (refreshBtn) {
                refreshBtn.disabled = false;
                refreshBtn.innerHTML = '<i class="mdi mdi-refresh"></i> Refresh';
            }
        }
    },

    render() {
        if (!this.data) return;

        this.renderNodeCards(this.data.nodes || []);
        this.renderWorkloads(this.data.workloads || []);
        this.renderPods(this.data.pods || []);
    },

    renderNodeCards(nodes) {
        const container = document.getElementById('gpu-nodes-container');
        if (!container) return;

        if (!nodes || nodes.length === 0) {
            container.innerHTML = '<div class="no-issues" style="grid-column: 1 / -1;">No GPU nodes detected</div>';
            return;
        }

        const getColor = (pct) => pct > 90 ? '#ef4444' : pct > 75 ? '#f59e0b' : '#00fff9';
        const getTempColor = (t) => t > 80 ? '#ef4444' : t > 65 ? '#f59e0b' : '#10b981';

        container.innerHTML = nodes.map(node => {
            const util = parseFloat(node.utilization) || 0;
            const temp = parseFloat(node.temperature) || 0;
            const power = parseFloat(node.power_usage) || 0;
            const memUsed = parseFloat(node.memory_used_mb) || 0;
            const memTotal = parseFloat(node.memory_total_mb) || 24576; // Default RTX 3090 / 4090 24GB
            const memPct = memTotal > 0 ? Math.round((memUsed / memTotal) * 100) : 0;
            const memFree = Math.max(0, memTotal - memUsed);
            const podsCount = node.pods_count || 0;

            const isOnline = node.status === 'Ready' || util > 0 || temp > 0;

            return `
                <div class="node-card ${isOnline ? 'ready' : 'not-ready'}" style="background: rgba(10, 20, 30, 0.6); border: 1px solid var(--neon-cyan); border-radius: 8px; padding: 1.25rem; display: flex; flex-direction: column; gap: 1rem; box-shadow: 0 0 15px rgba(0, 255, 249, 0.1);">
                    <!-- Header -->
                    <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid rgba(0, 255, 249, 0.2); padding-bottom: 0.5rem;">
                        <div style="display: flex; align-items: center; gap: 0.5rem;">
                            <i class="mdi mdi-expansion-card" style="font-size: 1.5rem; color: var(--neon-cyan);"></i>
                            <div>
                                <div style="font-weight: bold; font-size: 1.1rem; color: #fff;">${node.name}</div>
                                <div style="font-size: 0.75rem; color: rgba(255,255,255,0.6);">NVIDIA GeForce / RTX Series</div>
                            </div>
                        </div>
                        <span class="status-badge ${isOnline ? 'healthy' : 'unhealthy'}" style="text-transform: uppercase;">
                            ${isOnline ? 'Active' : 'Offline'}
                        </span>
                    </div>

                    <!-- Gauges Mini Row -->
                    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 0.75rem; text-align: center;">
                        <!-- GPU Core Load -->
                        <div style="background: rgba(0,0,0,0.4); padding: 0.5rem; border-radius: 6px; border: 1px solid rgba(255,255,255,0.05);">
                            <div style="font-size: 0.7rem; color: rgba(255,255,255,0.6); margin-bottom: 0.25rem;">GPU LOAD</div>
                            <div style="font-size: 1.25rem; font-weight: bold; color: ${getColor(util)}; font-family: 'JetBrains Mono', monospace;">
                                ${util.toFixed(0)}%
                            </div>
                            <div style="width: 100%; height: 4px; background: rgba(255,255,255,0.1); border-radius: 2px; margin-top: 0.35rem; overflow: hidden;">
                                <div style="width: ${Math.min(util, 100)}%; height: 100%; background: ${getColor(util)};"></div>
                            </div>
                        </div>

                        <!-- Temp -->
                        <div style="background: rgba(0,0,0,0.4); padding: 0.5rem; border-radius: 6px; border: 1px solid rgba(255,255,255,0.05);">
                            <div style="font-size: 0.7rem; color: rgba(255,255,255,0.6); margin-bottom: 0.25rem;">TEMP</div>
                            <div style="font-size: 1.25rem; font-weight: bold; color: ${getTempColor(temp)}; font-family: 'JetBrains Mono', monospace;">
                                ${temp > 0 ? temp.toFixed(0) + '°C' : '--'}
                            </div>
                            <div style="font-size: 0.7rem; color: rgba(255,255,255,0.5); margin-top: 0.35rem;">
                                ${temp > 75 ? '🔥 Warm' : '❄️ Cool'}
                            </div>
                        </div>

                        <!-- Power -->
                        <div style="background: rgba(0,0,0,0.4); padding: 0.5rem; border-radius: 6px; border: 1px solid rgba(255,255,255,0.05);">
                            <div style="font-size: 0.7rem; color: rgba(255,255,255,0.6); margin-bottom: 0.25rem;">POWER</div>
                            <div style="font-size: 1.25rem; font-weight: bold; color: #ffb86c; font-family: 'JetBrains Mono', monospace;">
                                ${power > 0 ? power.toFixed(0) + ' W' : '--'}
                            </div>
                            <div style="font-size: 0.7rem; color: rgba(255,255,255,0.5); margin-top: 0.35rem;">
                                Draw
                            </div>
                        </div>
                    </div>

                    <!-- VRAM Progress Bar -->
                    <div style="background: rgba(0,0,0,0.3); padding: 0.75rem; border-radius: 6px; border: 1px solid rgba(0, 255, 249, 0.1);">
                        <div style="display: flex; justify-content: space-between; font-size: 0.8rem; margin-bottom: 0.35rem;">
                            <span style="color: var(--neon-cyan);"><i class="mdi mdi-memory"></i> VRAM Usage</span>
                            <span style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem;">
                                <strong>${(memUsed / 1024).toFixed(2)} GB</strong> / ${(memTotal / 1024).toFixed(1)} GB (${memPct}%)
                            </span>
                        </div>
                        <div style="width: 100%; height: 8px; background: rgba(255,255,255,0.1); border-radius: 4px; overflow: hidden;">
                            <div style="width: ${Math.min(memPct, 100)}%; height: 100%; background: linear-gradient(90deg, #00fff9, #bd93f9); transition: width 0.3s ease;"></div>
                        </div>
                        <div style="display: flex; justify-content: space-between; font-size: 0.7rem; color: rgba(255,255,255,0.6); margin-top: 0.3rem;">
                            <span>Free: ${(memFree / 1024).toFixed(2)} GB</span>
                            <span>Scheduled Pods: ${podsCount}</span>
                        </div>
                    </div>
                </div>
            `;
        }).join('');
    },

    renderWorkloads(workloads) {
        const container = document.getElementById('gpu-workloads-list');
        if (!container) return;

        if (!workloads || workloads.length === 0) {
            container.innerHTML = '<div style="color: rgba(255,255,255,0.6); font-size: 0.85rem;">No managed GPU workloads detected.</div>';
            return;
        }

        container.innerHTML = workloads.map(w => {
            const isRunning = w.running;
            return `
                <div style="background: rgba(0,0,0,0.5); border: 1px solid ${isRunning ? 'var(--neon-green)' : 'rgba(255,255,255,0.15)'}; border-radius: 6px; padding: 1rem; display: flex; flex-direction: column; justify-content: space-between; gap: 0.75rem;">
                    <div>
                        <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                            <span style="font-weight: bold; color: ${isRunning ? 'var(--neon-green)' : '#fff'}; font-size: 1rem;">
                                ${w.name}
                            </span>
                            <span class="status-badge ${isRunning ? 'healthy' : 'unhealthy'}" style="font-size: 0.7rem;">
                                ${w.status} (${w.replicas} rep)
                            </span>
                        </div>
                        <div style="font-size: 0.75rem; color: rgba(255,255,255,0.6); margin-top: 0.25rem;">
                            ${w.description}
                        </div>
                        <div style="font-size: 0.7rem; color: var(--neon-cyan); margin-top: 0.25rem;">
                            Namespace: ${w.namespace}
                        </div>
                    </div>

                    <div style="display: flex; gap: 0.5rem; margin-top: 0.5rem;">
                        ${isRunning ? `
                            <button class="cyber-btn sm" onclick="GpuDashboard.scaleWorkload('${w.namespace}', '${w.name}', 0)" style="border-color: #ff5555; color: #ff5555; width: 100%;">
                                <i class="mdi mdi-stop"></i> Stop (Free VRAM)
                            </button>
                        ` : `
                            <button class="cyber-btn sm" onclick="GpuDashboard.scaleWorkload('${w.namespace}', '${w.name}', 1)" style="border-color: var(--neon-green); color: var(--neon-green); width: 100%;">
                                <i class="mdi mdi-play"></i> Start (1 Replica)
                            </button>
                        `}
                    </div>
                </div>
            `;
        }).join('');
    },

    renderPods(pods) {
        const tbody = document.getElementById('gpu-pods-tbody');
        const countBadge = document.getElementById('gpu-pods-count');
        if (!tbody) return;

        // Apply filters
        let filtered = pods || [];

        if (this.nodeFilter !== 'all') {
            filtered = filtered.filter(p => (p.node || '').includes(this.nodeFilter));
        }

        if (this.searchQuery) {
            const q = this.searchQuery.toLowerCase();
            filtered = filtered.filter(p => 
                (p.name || '').toLowerCase().includes(q) ||
                (p.namespace || '').toLowerCase().includes(q) ||
                (p.images || []).some(img => img.toLowerCase().includes(q))
            );
        }

        if (countBadge) countBadge.textContent = filtered.length;

        if (filtered.length === 0) {
            tbody.innerHTML = `
                <tr>
                    <td colspan="8" style="text-align: center; padding: 2rem; color: rgba(255,255,255,0.6);">
                        No pods match filter on GPU nodes
                    </td>
                </tr>
            `;
            return;
        }

        tbody.innerHTML = filtered.map(pod => {
            const isRunning = pod.status === 'Running' || pod.status === 'Succeeded';
            const statusClass = isRunning ? 'healthy' : (pod.status === 'Pending' ? 'progressing' : 'unhealthy');

            const imagesHtml = (pod.images || []).map(img => {
                const shortImg = img.split('/').pop();
                return `<div style="font-family: monospace; font-size: 0.75rem; color: #a0aec0;" title="${img}">📦 ${shortImg}</div>`;
            }).join('');

            return `
                <tr>
                    <td style="font-weight: bold; color: var(--neon-cyan); font-family: monospace;">
                        ${pod.name}
                    </td>
                    <td><span class="tag" style="background: rgba(0, 255, 249, 0.1); border: 1px solid rgba(0, 255, 249, 0.3); border-radius: 4px; padding: 2px 6px; font-size: 0.75rem;">${pod.namespace}</span></td>
                    <td style="font-weight: bold; color: #ffb86c;">${pod.node || '-'}</td>
                    <td>
                        <span class="status-badge ${statusClass}">${pod.status}</span>
                        ${pod.reason ? `<div style="color: #ff5555; font-size: 0.75rem;">${pod.reason}</div>` : ''}
                    </td>
                    <td style="text-align: center; font-weight: bold; ${pod.restart_count > 0 ? 'color: #ff5555;' : ''}">${pod.restart_count}</td>
                    <td style="font-size: 0.8rem; color: rgba(255,255,255,0.7);">${pod.age}</td>
                    <td>
                        ${imagesHtml}
                        ${pod.has_gpu_env ? `<span style="font-size: 0.7rem; color: #50fa7b; border: 1px solid #50fa7b; border-radius: 3px; padding: 1px 4px;">CUDA Env</span>` : ''}
                    </td>
                    <td style="text-align: right;">
                        <div style="display: flex; gap: 5px; justify-content: flex-end;">
                            <button class="cyber-btn sm" onclick="GpuDashboard.viewPodLogs('${pod.namespace}', '${pod.name}')" title="Logs in OpenObserve">
                                <i class="mdi mdi-text-box-search-outline"></i> Logs
                            </button>
                            <button class="cyber-btn sm" onclick="GpuDashboard.forceDeletePod('${pod.namespace}', '${pod.name}')" title="Delete / Restart Pod" style="border-color: #ff5555; color: #ff5555;">
                                <i class="mdi mdi-delete-outline"></i>
                            </button>
                        </div>
                    </td>
                </tr>
            `;
        }).join('');
    },

    handleFilter(query) {
        this.searchQuery = query;
        this.renderPods(this.data?.pods || []);
    },

    setNodeFilter(btn, node) {
        this.nodeFilter = node;
        document.querySelectorAll('.filter-btn[data-node]').forEach(b => b.classList.remove('active'));
        if (btn) btn.classList.add('active');
        this.renderPods(this.data?.pods || []);
    },

    async scaleWorkload(namespace, deployment, replicas) {
        const action = replicas === 0 ? 'stop' : 'start';
        if (!confirm(`Are you sure you want to ${action} ${namespace}/${deployment} (replicas: ${replicas})?`)) return;

        try {
            if (window.showNotification) showNotification({ title: 'Scaling Workload', message: `Setting ${deployment} replicas to ${replicas}...`, severity: 'info' });

            const res = await api.post('/api/k8s/gpu/scale', {
                namespace,
                deployment,
                replicas
            });

            if (res.success) {
                if (window.showNotification) showNotification({ title: 'Success', message: res.message, severity: 'success' });
                setTimeout(() => this.loadData(true), 1500);
            } else {
                if (window.showNotification) showNotification({ title: 'Error', message: res.error || 'Failed to scale', severity: 'error' });
            }
        } catch (error) {
            console.error('Scale error:', error);
            if (window.showNotification) showNotification({ title: 'Error', message: error.message, severity: 'error' });
        }
    },

    viewPodLogs(namespace, podName) {
        console.log(`📄 Opening OpenObserve logs for ${namespace}/${podName}`);
        const sqlQuery = `SELECT * FROM "default" WHERE k8s_pod_name = '${podName}'`;
        const encodedQuery = btoa(sqlQuery);
        const openObserveUrl = `https://o2-openobserve.p.zacharie.org/web/logs?stream_type=logs&stream=default&period=1d&refresh=0&sql_mode=true&query=${encodedQuery}&fn_editor=false&defined_schemas=user_defined_schema&org_identifier=default&quick_mode=false&show_histogram=true&logs_visualize_toggle=logs`;
        window.open(openObserveUrl, '_blank');
    },

    async forceDeletePod(namespace, podName) {
        if (!confirm(`⚠️ Force Delete Pod: ${namespace}/${podName}?`)) return;
        try {
            const data = await api.post('/api/pods/force-delete', { namespace, pod_name: podName });
            if (data.success) {
                if (window.showNotification) showNotification({ title: 'Pod Deleted', message: `Deleted ${podName}`, severity: 'success' });
                setTimeout(() => this.loadData(true), 1000);
            } else {
                if (window.showNotification) showNotification({ title: 'Delete Failed', message: data.message, severity: 'error' });
            }
        } catch (error) {
            console.error('Failed to delete pod:', error);
        }
    }
};

window.GpuDashboard = GpuDashboard;
