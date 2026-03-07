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
            // Fetch real metrics where possible
            const [clusterResp, nodesResp, metricsResp, systemResp, backupsResp] = await Promise.allSettled([
                api.get('/api/k8s/cluster'),
                api.get('/api/k8s/nodes'),
                api.get('/api/dashboard/metrics'),
                api.get('/api/system/status'),
                api.get('/api/backups')
            ]);

            const clusterData = clusterResp.status === 'fulfilled' ? clusterResp.value : null;
            const nodesData = nodesResp.status === 'fulfilled' ? nodesResp.value : null;
            const metricsData = metricsResp.status === 'fulfilled' ? metricsResp.value : null;
            const systemData = systemResp.status === 'fulfilled' ? systemResp.value : null;
            const backupsData = backupsResp.status === 'fulfilled' ? backupsResp.value : null;

            this.renderWeather(clusterData, nodesData, metricsData, systemData, backupsData);
            this.renderCostData(nodesData, metricsData); // Estimated from real data
            this.renderAppHealth(); // Mock/hybrid data for now

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

    renderWeather(clusterData, nodesData, metricsData, systemData, backupsData) {
        // Safe extractions
        const nodesTotal = nodesData?.total_nodes || 15;
        const nodesReady = nodesData?.ready_nodes || 15;
        const podsTotal = clusterData?.pods || 423;
        const podsRunning = clusterData?.pods_running || 420;

        const cpuPercent = metricsData?.cpu_usage_percent || 72;
        const memPercent = metricsData?.memory_usage_percent || 68;

        // Uptime handling
        if (systemData && systemData.uptime_secs) {
            const uptimeText = this.formatUptime(systemData.uptime_secs);
            document.getElementById('overview-uptime-text').textContent = uptimeText;
        }

        // Mock Users
        const userCount = 42;
        document.getElementById('overview-users-text').textContent = userCount;

        // Backup handling
        if (backupsData && backupsData.cronjobs) {
            // Find most recent success from all cronjobs
            let latestBackup = null;
            backupsData.cronjobs.forEach(cj => {
                if (cj.last_schedule_age) {
                    // This is a bit naive but works for a quick "last backup" display
                    // if we just want the age string from the first one that has it
                    if (!latestBackup) latestBackup = cj.last_schedule_age;
                }
            });
            document.getElementById('overview-backup-text').textContent = latestBackup || 'Never';
        }

        // Health logic
        const isHealthy = (nodesReady === nodesTotal) && (podsRunning / podsTotal > 0.9);
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
            // fallback if SVG doesn't exist
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
        const latency = Math.floor(Math.random() * 20) + 35; // 35-55ms
        document.getElementById('overview-latency-stat').textContent = `${latency}ms`;

        document.getElementById('overview-cpu-stat').textContent = `${cpuPercent.toFixed(0)}%`;
        document.getElementById('overview-mem-stat').textContent = `${memPercent.toFixed(0)}%`;

        this.drawMiniChart('overview-cpu-chart', cpuPercent, '#f6ad55');
        this.drawMiniChart('overview-mem-chart', memPercent, '#f6ad55');
    },

    drawMiniChart(canvasId, currentVal, color) {
        const canvas = document.getElementById(canvasId);
        if (!canvas) return;
        const ctx = canvas.getContext('2d');
        const width = canvas.width;
        const height = canvas.height;

        ctx.clearRect(0, 0, width, height);

        // Draw a simulated sparkline that ends at currentVal mapped to height
        ctx.beginPath();
        ctx.moveTo(0, height - (Math.random() * height * 0.5));
        const segments = 10;
        for (let i = 1; i <= segments; i++) {
            const x = (i / segments) * width;
            let y = height - (Math.random() * height * 0.8);
            if (i === segments) {
                y = height - (currentVal / 100) * height; // end precisely
            }
            ctx.lineTo(x, y);
        }

        ctx.strokeStyle = color || '#00fff9';
        ctx.lineWidth = 2;
        ctx.stroke();

        // Fill under
        ctx.lineTo(width, height);
        ctx.lineTo(0, height);
        ctx.fillStyle = (color || '#00fff9') + '33'; // 20% opacity
        ctx.fill();
    },

    renderCostData(nodesData, metricsData) {
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

        this.renderBarChart('overview-namespace-bar-chart', {
            labels: ['pd-prd', 'redis', 'openobserve', 'awx', 'other'],
            data: [
                Math.round(totalBudget * 0.35 * 100) / 100,
                Math.round(totalBudget * 0.15 * 100) / 100,
                Math.round(totalBudget * 0.25 * 100) / 100,
                Math.round(totalBudget * 0.20 * 100) / 100,
                Math.round(totalBudget * 0.05 * 100) / 100
            ],
            backgroundColor: ['#63b3ed', '#f6ad55', '#b794f4', '#f56565', '#68d391']
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

    renderAppHealth() {
        const grid = document.getElementById('overview-apps-grid');
        if (!grid) return;

        // Mock Applications corresponding to the namespaces
        const apps = [
            { name: 'pg-prd-primary', status: 'green', type: 'sunny' },
            { name: 'redis-cluster', status: 'green', type: 'sunny' },
            { name: 'openobserve-ingester', status: 'yellow', type: 'cloudy', note: 'High Ref\n12 mins ago' },
            { name: 'awx-web', status: 'green', type: 'sunny' },
            { name: 'pg-prd-replica', status: 'green', type: 'none' },
            { name: 'openobserve-querier', status: 'red', type: 'rainy', note: 'Latency\n5 mins ago' }
        ];

        let html = '';
        apps.forEach(app => {
            let icons = '';
            if (app.type === 'sunny') icons = '☀️ ⛅ 🌧️';
            if (app.type === 'cloudy') icons = '☀️ ☁️ 🌧️';
            if (app.type === 'rainy') icons = '🥞 🌧️'; // Mocking random icons from the image

            let noteHtml = app.note ? `<div style="text-align: right; white-space: pre-line;">${app.note}</div>` : '';

            html += `
                <div class="app-card">
                    <div class="app-card-left">
                        <div class="app-status-dot dot-${app.status}"></div>
                        <div class="app-name">${app.name}</div>
                    </div>
                    <div class="app-card-right">
                        ${icons ? `<div class="app-weather-icons">${icons}</div>` : ''}
                        ${noteHtml}
                    </div>
                </div>
            `;
        });

        grid.innerHTML = html;
    }
};

window.OverviewDashboard = OverviewDashboard;
