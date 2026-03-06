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
            const [clusterResp, nodesResp, metricsResp] = await Promise.allSettled([
                api.get('/api/k8s/cluster'),
                api.get('/api/k8s/nodes'),
                api.get('/api/dashboard/metrics')
            ]);

            const clusterData = clusterResp.status === 'fulfilled' ? clusterResp.value : null;
            const nodesData = nodesResp.status === 'fulfilled' ? nodesResp.value : null;
            const metricsData = metricsResp.status === 'fulfilled' ? metricsResp.value : null;

            this.renderWeather(clusterData, nodesData, metricsData);
            this.renderCostData(); // Mock data for now
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

    renderWeather(clusterData, nodesData, metricsData) {
        // Safe extractions
        const nodesTotal = nodesData?.total_nodes || 15;
        const nodesReady = nodesData?.ready_nodes || 15;
        const podsTotal = clusterData?.pods || 423;
        const podsRunning = clusterData?.pods_running || 420;

        const cpuPercent = metricsData?.cpu_usage_percent || 72;
        const memPercent = metricsData?.memory_usage_percent || 68;

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

    renderCostData() {
        // --- Mock Data ---
        document.getElementById('overview-current-cost').textContent = '$3,450.20';
        document.getElementById('overview-forecast-cost').textContent = '$4,200.00';

        this.renderPieChart('overview-cost-pie-chart', {
            labels: ['Compute: $2,100 (61%)', 'Storage: $950 (28%)', 'Network: $400 (11%)'],
            data: [2100, 950, 400],
            backgroundColor: ['#4299e1', '#ed8936', '#f56565']
        });

        this.renderBarChart('overview-namespace-bar-chart', {
            labels: ['frontend', 'backend', 'database', 'monitoring', 'default'],
            data: [1250, 980, 650, 320, 250],
            backgroundColor: ['#63b3ed', '#f6ad55', '#b794f4', '#f56565', '#68d391']
        });
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
                                return ` $${context.raw}`;
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

        // Mock Applications corresponding to the design
        const apps = [
            { name: 'frontend-app', status: 'green', type: 'sunny' },
            { name: 'backend-svc', status: 'yellow', type: 'cloudy' },
            { name: 'redis-db', status: 'red', type: 'rainy', note: 'Redis\n24 mins ago' },
            { name: 'replara-app', status: 'green', type: 'none', note: 'CPU\n24 mins ago' },
            { name: 'network-dv', status: 'red', type: 'none', note: 'Memory\n24 mins ago' },
            { name: 'local-ro', status: 'green', type: 'none', note: 'Redis\n24 mins ago' }
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
