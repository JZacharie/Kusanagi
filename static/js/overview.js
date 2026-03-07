/**
 * Kusanagi Overview Dashboard
 * Handles the "Weather Dashboard" view for cluster metrics and mock cost data
 */

const OverviewDashboard = {
    initialized: false,
    updateInterval: null,
    charts: {},

    init() {
        if (this.initialized) return;

        console.log('🌦️ Overview Dashboard initializing...');

        // Setup Event Listeners
        const clusterDropdown = document.getElementById('overview-cluster-dropdown');
        if (clusterDropdown) {
            clusterDropdown.addEventListener('change', () => this.refreshData());
        }

        this.initialized = true;
        this.refreshData();

        // Refresh every 5 minutes (300000 ms) to match the mockup text
        this.updateInterval = setInterval(() => this.refreshData(), 300000);
    },

    async refreshData() {
        if (!document.getElementById('overview-health-icon')) return; // Not on page

        this.updateTimestamp();

        try {
            const now = Math.floor(Date.now() / 1000);
            const start = now - 3600;

            // Fetch real metrics where possible
            const [clusterResp, nodesResp, metricsResp, systemResp, backupsResp, nsMetricsResp, podsResp, cpuHisResp, memHisResp] = await Promise.allSettled([
                api.get('/api/k8s/cluster'),
                api.get('/api/k8s/nodes'),
                api.get('/api/dashboard/metrics'),
                api.get('/api/system/status'),
                api.get('/api/backups'),
                api.get('/api/k8s/namespaces/metrics'),
                api.get('/api/k8s/pods'),
                api.get(`/api/prometheus/range?query=${encodeURIComponent('avg(1 - rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100')}&start=${start}&end=${now}&step=300`),
                api.get(`/api/prometheus/range?query=${encodeURIComponent('avg(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100')}&start=${start}&end=${now}&step=300`)
            ]);

            const clusterData = clusterResp.status === 'fulfilled' ? clusterResp.value : null;
            const nodesData = nodesResp.status === 'fulfilled' ? nodesResp.value : null;
            const metricsData = metricsResp.status === 'fulfilled' ? metricsResp.value : null;
            const systemData = systemResp.status === 'fulfilled' ? systemResp.value : null;
            const backupsData = backupsResp.status === 'fulfilled' ? backupsResp.value : null;
            const nsMetricsData = nsMetricsResp.status === 'fulfilled' ? nsMetricsResp.value : null;
            const podsData = podsResp.status === 'fulfilled' ? podsResp.value : null;
            const cpuHistory = cpuHisResp.status === 'fulfilled' ? cpuHisResp.value : null;
            const memHistory = memHisResp.status === 'fulfilled' ? memHisResp.value : null;

            const pipelineData = await fetch('/api/github/pipelines').then(r => r.json()).catch(() => []);

            this.renderWeather(clusterData, nodesData, metricsData, systemData, backupsData, cpuHistory, memHistory);
            this.renderCostData(nodesData, metricsData, nsMetricsData);
            this.renderAppHealth(podsData);
            this.renderSecurityScore(metricsData);
            this.renderPipelines(pipelineData);

        } catch (error) {
            console.error('Error refreshing overview data:', error);
        }
    },

    updateTimestamp() {
        const span = document.getElementById('overview-last-update');
        if (!span) return;

        const now = new Date();
        const options = { month: 'short', day: 'numeric', year: 'numeric', hour: 'numeric', minute: '2-digit', hour12: true };
        span.textContent = now.toLocaleString('en-US', options);
    },

    renderWeather(clusterData, nodesData, metricsData, systemData, backupsData, cpuHistory, memHistory) {
        // Safe extractions
        const nodesTotal = nodesData?.total_nodes || 0;
        const nodesReady = nodesData?.ready_nodes || 0;
        const podsTotal = clusterData?.pods || 0;
        const podsRunning = clusterData?.pods_running || 0;

        const cpuPercent = metricsData?.cpu_usage_percent || 0;
        const memPercent = metricsData?.memory_usage_percent || 0;

        // Uptime handling
        if (systemData && systemData.uptime_secs) {
            const uptimeText = this.formatUptime(systemData.uptime_secs);
            document.getElementById('overview-uptime-text').textContent = uptimeText;
        }

        // Mock Users (Keep as 42 or random)
        document.getElementById('overview-users-text').textContent = "42";

        // Backup handling
        if (backupsData && backupsData.cronjobs) {
            let latestBackup = null;
            backupsData.cronjobs.forEach(cj => {
                if (cj.last_schedule_age && !latestBackup) latestBackup = cj.last_schedule_age;
            });
            document.getElementById('overview-backup-text').textContent = latestBackup || 'Never';
        }

        // Health logic: Sunny only if all nodes ready AND >90% pods running
        const isHealthy = (nodesTotal > 0 && nodesReady === nodesTotal) && (podsTotal > 0 && podsRunning / podsTotal > 0.9);
        const healthText = isHealthy ? 'SUNNY' : 'CLOUDY';
        const healthIcon = isHealthy ? '☀️' : '⛅';
        const weatherImg = isHealthy ? '/static/images/weather/sunny.svg' : '/static/images/weather/cloudy.svg';

        // Update DOM
        document.getElementById('overview-health-icon').textContent = healthIcon;
        const mainHealthText = document.getElementById('overview-health-text');
        mainHealthText.textContent = healthText;
        mainHealthText.className = isHealthy ? 'status-sunny' : 'status-warning';

        const weatherImgEl = document.getElementById('overview-weather-img');
        if (weatherImgEl) {
            weatherImgEl.onerror = () => {
                weatherImgEl.style.display = 'none';
                const parent = weatherImgEl.parentElement;
                let iconEl = parent.querySelector('.overview-weather-icon');
                if (!iconEl) {
                    iconEl = document.createElement('span');
                    iconEl.className = 'overview-weather-icon';
                    parent.insertBefore(iconEl, weatherImgEl);
                }
                iconEl.textContent = healthIcon;
            };
            weatherImgEl.src = weatherImg;
        }

        const weatherTitle = document.getElementById('overview-weather-title');
        weatherTitle.textContent = healthText;
        weatherTitle.className = `weather-title ${isHealthy ? 'status-sunny' : 'status-warning'}`;

        document.getElementById('overview-nodes-stat').textContent = `${nodesReady}/${nodesTotal} Healthy`;
        document.getElementById('overview-pods-stat').textContent = `${podsRunning}/${podsTotal} Running`;

        // Mock API latency
        const latency = Math.floor(Math.random() * 20) + 35;
        document.getElementById('overview-latency-stat').textContent = `${latency}ms`;

        document.getElementById('overview-cpu-stat').textContent = `${cpuPercent.toFixed(0)}%`;
        document.getElementById('overview-mem-stat').textContent = `${memPercent.toFixed(0)}%`;

        // Extract values from Prometheus history if available
        const extractHistory = (history) => {
            if (history?.data?.result?.[0]?.values) {
                return history.data.result[0].values.map(v => parseFloat(v[1]));
            }
            return null;
        };

        const cpuValues = extractHistory(cpuHistory);
        const memValues = extractHistory(memHistory);

        this.drawMiniChart('overview-cpu-chart', cpuPercent, '#f6ad55', cpuValues);
        this.drawMiniChart('overview-mem-chart', memPercent, '#f6ad55', memValues);
    },

    drawMiniChart(canvasId, currentVal, color, historyValues) {
        const canvas = document.getElementById(canvasId);
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        const width = canvas.width;
        const height = canvas.height;

        ctx.clearRect(0, 0, width, height);

        ctx.beginPath();

        if (historyValues && Array.isArray(historyValues) && historyValues.length > 1) {
            // Draw real historical sparkline
            const maxVal = Math.max(...historyValues, currentVal, 100);
            const segments = historyValues.length - 1;

            historyValues.forEach((val, i) => {
                const x = (i / segments) * width;
                const y = height - (val / 100) * height; // Use 100 as base for percentage
                if (i === 0) ctx.moveTo(x, y);
                else ctx.lineTo(x, y);
            });
        } else {
            // Fallback to simulated sparkline if no history
            ctx.moveTo(0, height - (Math.random() * height * 0.5));
            const segments = 10;
            for (let i = 1; i <= segments; i++) {
                const x = (i / segments) * width;
                let y = height - (Math.random() * height * 0.8);
                if (i === segments) {
                    y = height - (currentVal / 100) * height;
                }
                ctx.lineTo(x, y);
            }
        }

        ctx.strokeStyle = color || '#00fff9';
        ctx.lineWidth = 2;
        ctx.stroke();

        // Fill under
        ctx.lineTo(width, height);
        ctx.lineTo(0, height);
        ctx.fillStyle = (color || '#00fff9') + '33';
        ctx.fill();
    },

    renderCostData(nodesData, metricsData, nsMetricsData) {
        // --- On-Premise Cost Analysis Adjustment ---
        // Total infrastructure budget for the month: 200€
        const totalBudget = 200;

        // Ratios for distribution (can be adjusted based on actual usage)
        const ratios = {
            compute: 0.40,   // 80€
            storage: 0.30,   // 60€
            network: 0.20,   // 40€
            llm: 0.10        // 20€
        };

        const computeCost = totalBudget * ratios.compute;
        const storageCost = totalBudget * ratios.storage;
        const networkCost = totalBudget * ratios.network;
        const llmTokensCost = totalBudget * ratios.llm;

        const totalCost = computeCost + storageCost + networkCost + llmTokensCost;
        const forecastCost = totalBudget; // Fixed budget for on-premise

        document.getElementById('overview-current-cost').textContent = `${totalCost.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}€`;
        document.getElementById('overview-forecast-cost').textContent = `${forecastCost.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}€`;

        this.renderPieChart('overview-cost-pie-chart', {
            labels: [
                `Compute: ${computeCost.toLocaleString()}€ (${(ratios.compute * 100).toFixed(0)}%)`,
                `Storage: ${storageCost.toLocaleString()}€ (${(ratios.storage * 100).toFixed(0)}%)`,
                `Tokens LLM: ${llmTokensCost.toLocaleString()}€ (${(ratios.llm * 100).toFixed(0)}%)`,
                `Network: ${networkCost.toLocaleString()}€ (${(ratios.network * 100).toFixed(0)}%)`
            ],
            data: [computeCost, storageCost, llmTokensCost, networkCost],
            backgroundColor: ['#4299e1', '#ed8936', '#b794f4', '#f56565']
        });

        // --- Resource Usage by Namespace Calculation ---
        let labels = [];
        let data = [];

        if (nsMetricsData && Array.isArray(nsMetricsData)) {
            // Calculate a score per namespace (0.5 * normalized CPU + 0.5 * normalized Memory)
            // For simplicity, we'll just sum them up or use a simplified weight
            const processed = nsMetricsData.map(ns => {
                // Normalize CPU (approx max 8 cores) and Mem (approx max 32Gb)
                const cpuVal = (ns.cpu_usage || 0);
                const memVal = (ns.memory_usage_bytes || 0) / (1024 * 1024 * 1024); // GB
                return {
                    name: ns.namespace,
                    weight: cpuVal + (memVal / 4) // simple weight
                };
            }).sort((a, b) => b.weight - a.weight);

            const top5 = processed.slice(0, 5);
            const others = processed.slice(5);

            const totalWeight = processed.reduce((sum, item) => sum + item.weight, 0) || 1;

            top5.forEach(item => {
                labels.push(item.name);
                data.push(Math.round((item.weight / totalWeight) * totalBudget * 100) / 100);
            });

            if (others.length > 0) {
                const otherWeight = others.reduce((sum, item) => sum + item.weight, 0);
                labels.push('other');
                data.push(Math.round((otherWeight / totalWeight) * totalBudget * 100) / 100);
            }
        } else {
            // Fallback to placeholders if API fails
            labels = ['pg-prd', 'redis', 'openobserve', 'awx', 'other'];
            data = [
                Math.round(totalBudget * 0.35 * 100) / 100,
                Math.round(totalBudget * 0.15 * 100) / 100,
                Math.round(totalBudget * 0.25 * 100) / 100,
                Math.round(totalBudget * 0.20 * 100) / 100,
                Math.round(totalBudget * 0.05 * 100) / 100
            ];
        }

        this.renderBarChart('overview-namespace-bar-chart', {
            labels: labels,
            data: data,
            backgroundColor: ['#63b3ed', '#f6ad55', '#b794f4', '#f56565', '#48bb78', '#a0aec0']
        });
    },

    formatUptime(seconds) {
        if (!seconds && seconds !== 0) return 'N/A';
        const days = Math.floor(seconds / 86400);
        const hours = Math.floor((seconds % 86400) / 3600);
        const minutes = Math.floor((seconds % 3600) / 60);
        const secs = seconds % 60;

        if (days > 0) {
            return `${days}j ${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
        }
        return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
    },

    renderPieChart(canvasId, config) {
        if (typeof Chart === 'undefined') {
            console.warn('Chart.js not loaded. Skipping pie chart.');
            return;
        }

        const ctx = document.getElementById(canvasId);
        if (!ctx) return;

        if (this.charts[canvasId]) {
            this.charts[canvasId].destroy();
        }

        this.charts[canvasId] = new Chart(ctx, {
            type: 'pie',
            data: {
                labels: config.labels,
                datasets: [{
                    data: config.data,
                    backgroundColor: config.backgroundColor,
                    borderWidth: 0
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: {
                        position: 'right',
                        labels: { color: '#e2e8f0', font: { size: 11 } }
                    }
                }
            }
        });
    },

    renderBarChart(canvasId, config) {
        if (typeof Chart === 'undefined') {
            console.warn('Chart.js not loaded. Skipping bar chart.');
            return;
        }

        const ctx = document.getElementById(canvasId);
        if (!ctx) return;

        if (this.charts[canvasId]) {
            this.charts[canvasId].destroy();
        }

        this.charts[canvasId] = new Chart(ctx, {
            type: 'bar',
            data: {
                labels: config.labels,
                datasets: [{
                    data: config.data,
                    backgroundColor: config.backgroundColor,
                    borderRadius: 4
                }]
            },
            options: {
                indexAxis: 'y', // horizontal bar chart
                responsive: true,
                maintainAspectRatio: false,
                plugins: {
                    legend: { display: false },
                    tooltip: {
                        callbacks: {
                            label: function (context) {
                                return ` ${context.raw}€`;
                            }
                        }
                    }
                },
                scales: {
                    x: {
                        grid: { color: '#2a3548' },
                        ticks: { color: '#a0aec0' }
                    },
                    y: {
                        grid: { display: false },
                        ticks: { color: '#e2e8f0' }
                    }
                }
            }
        });
    },

    renderAppHealth(podsData) {
        const grid = document.getElementById('overview-apps-grid');
        if (!grid) return;

        let degradedApps = [];

        if (podsData && Array.isArray(podsData)) {
            // Filter for pods that are not in "Running" or "Succeeded"
            // and group them roughly by name (simple deduplication for common prefixes)
            const problematic = podsData.filter(pod => {
                const status = pod.status;
                return status !== 'Running' && status !== 'Succeeded';
            });

            const seen = new Set();
            problematic.forEach(pod => {
                // Try to find a meaningful name (e.g. part before the last dash if it looks like a hash)
                let name = pod.name;
                const parts = name.split('-');
                if (parts.length > 2 && /^[a-z0-9]{8,10}$/.test(parts[parts.length - 1])) {
                    name = parts.slice(0, -1).join('-');
                } else if (parts.length > 1 && /^[a-z0-9]{5,}$/.test(parts[parts.length - 1])) {
                    name = parts.slice(0, -1).join('-');
                }

                if (!seen.has(name)) {
                    seen.add(name);
                    degradedApps.push({
                        name: name,
                        namespace: pod.namespace,
                        status: 'red',
                        type: 'rainy',
                        note: `${pod.status}\nin ${pod.namespace}`
                    });
                }
            });
        }

        if (degradedApps.length === 0) {
            grid.innerHTML = '<div style="color: #48bb78; grid-column: 1/-1; text-align: center; padding: 20px;">All systems operational. No degraded applications found.</div>';
            return;
        }

        let html = '';
        degradedApps.forEach(app => {
            const icons = '⛅ 🌧️';
            let noteHtml = app.note ? `<div style="text-align: right; white-space: pre-line; font-size: 10px; color: #a0aec0;">${app.note}</div>` : '';

            html += `
                <div class="app-card">
                    <div class="app-card-left">
                        <div class="app-status-dot dot-${app.status}"></div>
                        <div class="app-name" title="${app.namespace}">${app.name}</div>
                    </div>
                    <div class="app-card-right">
                        <div class="app-weather-icons">${icons}</div>
                        ${noteHtml}
                    </div>
                </div>
            `;
        });

        grid.innerHTML = html;
    },

    renderSecurityScore(metricsData) {
        if (!metricsData || !metricsData.security_score) {
            document.getElementById('overview-security-score').textContent = '100%';
            return;
        }

        const score = metricsData.security_score;
        const details = metricsData.security_details;

        const scoreEl = document.getElementById('overview-security-score');
        scoreEl.textContent = `${score.toFixed(1)}%`;

        // Color coding for score
        if (score >= 90) scoreEl.className = 'security-score-value health-good';
        else if (score >= 70) scoreEl.className = 'security-score-value status-warning';
        else scoreEl.className = 'security-score-value status-critical';

        if (details) {
            document.getElementById('overview-trivy-score').textContent = `${details.trivy_score.toFixed(1)}%`;
            document.getElementById('overview-compliance-score').textContent = `${details.steampipe_score.toFixed(1)}%`;

            const summaryEl = document.getElementById('overview-security-summary');
            const stats = details.steampipe_stats;
            if (stats && stats.total_checks) {
                summaryEl.innerHTML = `
                    <div style="font-size: 0.75rem; color: #a0aec0; margin-top: 0.5rem;">
                        ${stats.passed} Passed / ${stats.failed} Failed checks<br>
                        ${metricsData.trivy_critical_count || 0} Critical vulnerabilities
                    </div>
                `;
            }
        }
    },

    renderPipelines(pipelines) {
        const listEl = document.getElementById('overview-pipelines-list');
        if (!pipelines || pipelines.length === 0) {
            listEl.innerHTML = '<div class="no-data">No recent pipelines found.</div>';
            return;
        }

        let html = '';
        pipelines.forEach(run => {
            const statusClass = run.status === 'completed'
                ? (run.conclusion === 'success' ? 'health-good' : 'status-critical')
                : 'status-warning';

            const icon = run.status === 'completed'
                ? (run.conclusion === 'success' ? '✅' : '❌')
                : '⏳';

            const date = new Date(run.created_at).toLocaleString();

            html += `
                <div class="pipeline-item">
                    <div class="pipeline-status-icon ${statusClass}">${icon}</div>
                    <div class="pipeline-info">
                        <div class="pipeline-repo">${run.repo}</div>
                        <div class="pipeline-name">${run.name || 'Workflow'}</div>
                        <div class="pipeline-meta">${date}</div>
                    </div>
                    <div class="pipeline-actions">
                        <a href="${run.url}" target="_blank" class="btn btn-xs">View</a>
                    </div>
                </div>
            `;
        });
        listEl.innerHTML = html;
    }
};

window.OverviewDashboard = OverviewDashboard;
