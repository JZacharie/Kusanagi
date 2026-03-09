/**
 * Kusanagi Overview Dashboard
 * Handles the "Weather Dashboard" view for cluster metrics and mock cost data
 */

const OverviewDashboard = {
    initialized: false,
    updateInterval: null,
    charts: {},
    namespaceCostWindow: '5m',

    init() {
        if (this.initialized) return;

        console.log('🌦️ Overview Dashboard initializing...');

        // Setup Event Listeners
        const clusterDropdown = document.getElementById('overview-cluster-dropdown');
        if (clusterDropdown) {
            clusterDropdown.addEventListener('change', () => this.refreshData());
        }

        // Timeframe toggle for Namespace Cost
        const timeframeContainer = document.getElementById('namespace-cost-timeframe');
        if (timeframeContainer) {
            timeframeContainer.addEventListener('click', (e) => {
                const btn = e.target.closest('.toggle-btn');
                if (!btn) return;

                // Update active state
                timeframeContainer.querySelectorAll('.toggle-btn').forEach(b => b.classList.remove('active'));
                btn.classList.add('active');

                // Update timeframe and refresh only cost data if possible, or full refresh
                this.namespaceCostWindow = btn.dataset.window;
                this.refreshNamespaceCost();
            });
        }

        // Generic Zone Collapsibility
        document.querySelectorAll('.zone-title').forEach(header => {
            header.addEventListener('click', () => {
                header.classList.toggle('collapsed');

                // Special handling for the Alert Card wrapper if needed
                const parentCard = header.closest('.overview-alert-card');
                if (parentCard) {
                    parentCard.classList.toggle('collapsed');
                }
            });
        });

        this.initialized = true;
        this.initBusinessMap();
        this.refreshData();
        this.initPromotionHandler();

        // Refresh every 5 minutes (300000 ms) to match the mockup text
        this.updateInterval = setInterval(() => this.refreshData(), 300000);
    },

    initBusinessMap() {
        const mapEl = document.getElementById('overview-business-map');
        if (!mapEl) return;

        console.log('🌍 Initializing Business Map on Overview...');

        if (!this.map) {
            this.map = L.map('overview-business-map', {
                center: [20, 0],
                zoom: 2,
                zoomControl: false,
                attributionControl: true
            });

            L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png', {
                attribution: '&copy; OpenStreetMap contributors &copy; CARTO',
                subdomains: 'abcd',
                maxZoom: 20
            }).addTo(this.map);

            L.control.zoom({ position: 'bottomright' }).addTo(this.map);

            // Force resize after init
            setTimeout(() => this.map.invalidateSize(), 100);
        }

        // Add special handling for collapsibility to resize map
        const bizHeader = mapEl.closest('.zone-content').previousElementSibling;
        if (bizHeader && bizHeader.classList.contains('zone-title')) {
            bizHeader.addEventListener('click', () => {
                if (!bizHeader.classList.contains('collapsed')) {
                    setTimeout(() => this.map.invalidateSize(), 200);
                }
            });
        }
    },

    initPromotionHandler() {
        const btn = document.getElementById('btn-promote-prod');
        const statusEl = document.getElementById('promote-status');
        if (!btn) return;

        btn.addEventListener('click', async () => {
            if (!confirm("Êtes-vous sûr de vouloir promouvoir la version Dev vers la Production ?")) return;

            btn.disabled = true;
            btn.innerHTML = '<span class="mdi mdi-loading mdi-spin"></span> PROMOTION EN COURS...';
            statusEl.innerHTML = '';
            statusEl.className = 'release-status';

            try {
                const response = await fetch('/api/github/promote', { method: 'POST' });
                const result = await response.json();

                if (response.ok && result.success) {
                    statusEl.innerHTML = `<span class="mdi mdi-check-circle"></span> ${result.message}`;
                    statusEl.className = 'release-status status-success';
                    btn.innerHTML = '<span class="mdi mdi-check"></span> PROMU AVEC SUCCÈS';
                } else {
                    throw new Error(result.message || 'Erreur lors de la promotion');
                }
            } catch (error) {
                console.error('Promotion error:', error);
                statusEl.innerHTML = `<span class="mdi mdi-alert-circle"></span> Erreur: ${error.message}`;
                statusEl.className = 'release-status status-error';
                btn.disabled = false;
                btn.innerHTML = '<span class="mdi mdi-cloud-upload"></span> RÉESSAYER LA PROMOTION';
            }
        });
    },

    async refreshData() {
        if (!document.getElementById('overview-health-icon')) return; // Not on page

        this.updateTimestamp();

        try {
            const now = Math.floor(Date.now() / 1000);
            const start = now - 3600;

            // Fetch real metrics where possible
            const [clusterResp, nodesResp, metricsResp, systemResp, backupsResp, nsMetricsResp, podsResp, cpuHisResp, memHisResp, argocdResp] = await Promise.allSettled([
                api.get('/api/k8s/cluster'),
                api.get('/api/k8s/nodes'),
                api.get('/api/dashboard/metrics'),
                api.get('/api/system/status'),
                api.get('/api/backups'),
                api.get(`/api/k8s/namespaces/metrics?window=${this.namespaceCostWindow}`),
                api.get('/api/k8s/pods'),
                api.get(`/api/prometheus/range?query=${encodeURIComponent('avg(1 - rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100')}&start=${start}&end=${now}&step=300`),
                api.get(`/api/prometheus/range?query=${encodeURIComponent('avg(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100')}&start=${start}&end=${now}&step=300`),
                api.get('/api/argocd/status')
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
            const argocdData = argocdResp.status === 'fulfilled' ? argocdResp.value : null;

            const pipelineResp = await fetch('/api/github/pipelines').then(r => r.json()).catch(() => ({ success: false }));
            const pipelineData = pipelineResp.success ? pipelineResp.data : [];

            this.renderWeather(clusterData, nodesData, metricsData, systemData, backupsData, cpuHistory, memHistory);
            this.renderCostData(nodesData, metricsData, nsMetricsData);
            this.renderAppHealth(podsData, argocdData);
            this.renderSecurityScore(metricsData);
            this.renderPipelines(pipelineData);
            this.renderBusinessMap();

        } catch (error) {
            console.error('Error refreshing overview data:', error);
        }
    },

    renderBusinessMap() {
        const mapEl = document.getElementById('overview-business-map');
        if (!mapEl) return;

        const countryData = [
            { id: 'FRA', name: 'France', users: 4500, investorsCount: 120, totalInvested: 2450000, performance: '+12.5%', geo: 'Europe', latlng: [46.2276, 2.2137] },
            { id: 'USA', name: 'USA', users: 3200, investorsCount: 85, totalInvested: 5800000, performance: '+15.2%', geo: 'NA', latlng: [37.0902, -95.7129] },
            { id: 'GBR', name: 'UK', users: 1800, investorsCount: 45, totalInvested: 1200000, performance: '+8.7%', geo: 'Europe', latlng: [55.3781, -3.4360] },
            { id: 'DEU', name: 'Germany', users: 1500, investorsCount: 38, totalInvested: 950000, performance: '+10.1%', geo: 'Europe', latlng: [51.1657, 10.4515] },
            { id: 'JPN', name: 'Japan', users: 1200, investorsCount: 25, totalInvested: 1100000, performance: '+6.4%', geo: 'Asia', latlng: [36.2048, 138.2529] },
            { id: 'CHN', name: 'China', users: 950, investorsCount: 15, totalInvested: 750000, performance: '+18.9%', geo: 'Asia', latlng: [35.8617, 104.1954] },
            { id: 'IND', name: 'India', users: 800, investorsCount: 12, totalInvested: 450000, performance: '+22.5%', geo: 'Asia', latlng: [20.5937, 78.9629] },
            { id: 'CAN', name: 'Canada', users: 650, investorsCount: 18, totalInvested: 550000, performance: '+11.2%', geo: 'NA', latlng: [56.1304, -106.3468] },
            { id: 'BRA', name: 'Brazil', users: 500, investorsCount: 8, totalInvested: 150000, performance: '+5.3%', geo: 'SA', latlng: [-14.2350, -51.9253] }
        ];

        // Ensure map is initialized
        if (!this.map) return;

        // Clear existing layers if any
        if (this.mapLayers) {
            this.mapLayers.forEach(layer => this.map.removeLayer(layer));
        }
        this.mapLayers = [];

        // Render Investment Markers (Orange Circles)
        countryData.forEach(c => {
            const radius = Math.sqrt(c.totalInvested) / 100; // Scale radius
            const marker = L.circleMarker(c.latlng, {
                radius: radius,
                fillColor: '#ed8936',
                color: '#fff',
                weight: 1,
                opacity: 1,
                fillOpacity: 0.6
            }).addTo(this.map);

            marker.bindPopup(`
                <div style="font-family: 'Rajdhani', sans-serif;">
                    <strong style="color: #ed8936; font-size: 1.1rem;">${c.name}</strong><br/>
                    <span style="color: #f6e05e;">${c.users.toLocaleString()} Utilisateurs</span><br/>
                    <span>${c.investorsCount} Investisseurs</span><br/>
                    <span style="font-weight: bold;">${c.totalInvested.toLocaleString()} €</span>
                </div>
            `);

            this.mapLayers.push(marker);
        });
    },

    async refreshNamespaceCost() {
        try {
            const resp = await api.get(`/api/k8s/namespaces/metrics?window=${this.namespaceCostWindow}`);
            // We need nodesData and metricsData for full context of cost distribution
            // but for now we can just use cached version if we store it, 
            // or just trigger re-render of cost data specifically if we have nodesData.
            // For simplicity, let's just trigger full refresh or pass current cached data.
            this.refreshData();
        } catch (e) {
            console.error('Failed to refresh namespace cost:', e);
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
            const uptimeEl = document.getElementById('overview-uptime-text');
            if (uptimeEl) uptimeEl.textContent = uptimeText;
        }

        // Mock Users (Keep as 42 or random)
        const usersEl = document.getElementById('overview-users-text');
        if (usersEl) usersEl.textContent = "42";

        // Backup handling
        if (backupsData && backupsData.cronjobs) {
            let latestBackup = null;
            backupsData.cronjobs.forEach(cj => {
                if (cj.last_schedule_age && !latestBackup) latestBackup = cj.last_schedule_age;
            });
            const backupEl = document.getElementById('overview-backup-text');
            if (backupEl) backupEl.textContent = latestBackup || 'Never';
        }

        // Health logic: Sunny only if all nodes ready AND >90% pods running AND no failed jobs
        const failedJobsCount = metricsData?.failed_jobs_count || 0;
        const isHealthy = (nodesTotal > 0 && nodesReady === nodesTotal) &&
            (podsTotal > 0 && podsRunning / podsTotal > 0.9) &&
            (failedJobsCount === 0);

        const healthText = isHealthy ? 'SUNNY' : (failedJobsCount > 5 ? 'STORMY' : 'CLOUDY');
        const healthIcon = isHealthy ? '☀️' : (failedJobsCount > 5 ? '⛈️' : '⛅');
        const weatherImg = isHealthy ? '/static/images/weather/sunny.svg' :
            (failedJobsCount > 5 ? '/static/images/weather/stormy.svg' : '/static/images/weather/cloudy.svg');

        // Update DOM
        const healthIconEl = document.getElementById('overview-health-icon');
        if (healthIconEl) healthIconEl.textContent = healthIcon;

        const mainHealthText = document.getElementById('overview-health-text');
        if (mainHealthText) {
            mainHealthText.textContent = healthText;
            mainHealthText.className = isHealthy ? 'status-sunny' : 'status-warning';
        }

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
        if (weatherTitle) {
            weatherTitle.textContent = healthText;
            weatherTitle.className = `weather-title ${isHealthy ? 'status-sunny' : 'status-warning'}`;
        }

        const nodesStatEl = document.getElementById('overview-nodes-stat');
        if (nodesStatEl) nodesStatEl.textContent = `${nodesReady}/${nodesTotal} Healthy`;

        const podsStatEl = document.getElementById('overview-pods-stat');
        if (podsStatEl) podsStatEl.textContent = `${podsRunning}/${podsTotal} Running`;

        // Detailed Cluster Resource Metrics
        const resources = metricsData?.cluster_resources;
        if (resources) {
            const cpu = resources.cpu || {};
            const mem = resources.memory || {};

            // Helper to set text and ensure no null errors
            const setRes = (id, val) => {
                const el = document.getElementById(id);
                if (el) el.textContent = val;
            };

            // CPU
            const cpuUsage = cpu.usage || 0;
            const cpuReq = cpu.requests || 0;
            const cpuLimit = cpu.limits || 0;
            const cpuAlloc = cpu.allocatable || 0;
            const cpuCap = cpu.capacity || 0;

            setRes('overview-cpu-usage', cpuUsage.toFixed(2));
            setRes('overview-cpu-req', cpuReq.toFixed(2));
            setRes('overview-cpu-limit', cpuLimit.toFixed(2));
            setRes('overview-cpu-alloc', cpuAlloc.toFixed(2));
            setRes('overview-cpu-cap', cpuCap.toFixed(2));

            this.renderGaugeChart('overview-cpu-gauge', cpuUsage, cpuReq, cpuLimit, cpuCap);

            // Memory (GiB)
            const memUsage = (mem.usage || 0) / (1024 * 1024 * 1024);
            const memReq = (mem.requests || 0) / (1024 * 1024 * 1024);
            const memLimit = (mem.limits || 0) / (1024 * 1024 * 1024);
            const memAlloc = (mem.allocatable || 0) / (1024 * 1024 * 1024);
            const memCap = (mem.capacity || 0) / (1024 * 1024 * 1024);

            setRes('overview-mem-usage', memUsage.toFixed(1) + 'GiB');
            setRes('overview-mem-req', memReq.toFixed(1) + 'GiB');
            setRes('overview-mem-limit', memLimit.toFixed(1) + 'GiB');
            setRes('overview-mem-alloc', memAlloc.toFixed(1) + 'GiB');
            setRes('overview-mem-cap', memCap.toFixed(1) + 'GiB');

            this.renderGaugeChart('overview-mem-gauge', memUsage, memReq, memLimit, memCap);
        }

        // Mock API latency
        const latency = Math.floor(Math.random() * 20) + 35;
        const latencyEl = document.getElementById('overview-latency-stat');
        if (latencyEl) latencyEl.textContent = `${latency}ms`;

        const cpuStatEl = document.getElementById('overview-cpu-stat');
        if (cpuStatEl) cpuStatEl.textContent = `${cpuPercent.toFixed(0)}%`;

        const memStatEl = document.getElementById('overview-mem-stat');
        if (memStatEl) memStatEl.textContent = `${memPercent.toFixed(0)}%`;

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

    renderGaugeChart(canvasId, usage, requests, limits, capacity) {
        if (typeof Chart === 'undefined') {
            console.warn('Chart.js not loaded. Skipping gauge chart.');
            return;
        }

        const ctx = document.getElementById(canvasId);
        if (!ctx) return;

        if (this.charts[canvasId]) {
            this.charts[canvasId].destroy();
        }

        const safeCapacity = capacity > 0 ? capacity : 1; // Prevent division by zero

        // Calculate remaining capacities for each ring
        const remainingUsage = Math.max(0, safeCapacity - usage);
        const remainingRequests = Math.max(0, safeCapacity - requests);
        const remainingLimits = Math.max(0, safeCapacity - limits);

        this.charts[canvasId] = new Chart(ctx, {
            type: 'doughnut',
            data: {
                labels: ['Usage', 'Requests', 'Limits', 'Available'],
                datasets: [
                    {
                        // Inner ring: Limits
                        data: [0, 0, limits, remainingLimits],
                        backgroundColor: ['transparent', 'transparent', '#319795', '#2d3748'],
                        borderWidth: 0,
                        circumference: 360,
                        weight: 1
                    },
                    {
                        // Middle ring: Requests
                        data: [0, requests, 0, remainingRequests],
                        backgroundColor: ['transparent', '#48bb78', 'transparent', '#2d3748'],
                        borderWidth: 0,
                        circumference: 360,
                        weight: 1
                    },
                    {
                        // Outer ring: Usage
                        data: [usage, 0, 0, remainingUsage],
                        backgroundColor: ['#d53f8c', 'transparent', 'transparent', '#2d3748'],
                        borderWidth: 0,
                        circumference: 360,
                        weight: 1
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                cutout: '50%', // Size of the inner hole
                plugins: {
                    legend: {
                        display: false // We use custom HTML legend
                    },
                    tooltip: {
                        callbacks: {
                            label: function (context) {
                                // Only show tooltip for the active (colored) section, not the gray background
                                const dataIndex = context.dataIndex;
                                const datasetIndex = context.datasetIndex; // 0: Limits, 1: Requests, 2: Usage

                                if (datasetIndex === 0 && dataIndex === 2) return `Limits: ${context.raw.toFixed(2)}`;
                                if (datasetIndex === 1 && dataIndex === 1) return `Requests: ${context.raw.toFixed(2)}`;
                                if (datasetIndex === 2 && dataIndex === 0) return `Usage: ${context.raw.toFixed(2)}`;

                                return null; // Hide tooltips for the gray background parts
                            }
                        },
                        filter: function (tooltipItem) {
                            // Filter out tooltips returning null label
                            const dataIndex = tooltipItem.dataIndex;
                            const datasetIndex = tooltipItem.datasetIndex;
                            return (datasetIndex === 0 && dataIndex === 2) ||
                                (datasetIndex === 1 && dataIndex === 1) ||
                                (datasetIndex === 2 && dataIndex === 0);
                        }
                    }
                }
            }
        });
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

            const top20 = processed.slice(0, 20);
            const others = processed.slice(20);

            const totalWeight = processed.reduce((sum, item) => sum + item.weight, 0) || 1;

            top20.forEach(item => {
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

    renderAppHealth(podsData, argocdData) {
        // 1. Update ArgoCD Status counts
        if (argocdData) {
            const updateCount = (selector, val) => {
                const el = document.querySelector(`.argocd-health-item.${selector} .argocd-val`);
                if (el) el.textContent = val || 0;
            };

            updateCount('progressing', argocdData.progressing);
            updateCount('suspended', argocdData.suspended);
            updateCount('healthy', argocdData.healthy);
            updateCount('degraded', argocdData.degraded);
            updateCount('missing', argocdData.missing);
            updateCount('unknown', argocdData.unknown);
        }

        // 2. Update Degraded Apps List
        const listContainer = document.getElementById('overview-degraded-apps-list');
        if (!listContainer) return;

        let degradedApps = [];

        // Prefer data from ArgoCD if available for specific degraded apps
        if (argocdData && argocdData.apps_with_issues) {
            degradedApps = argocdData.apps_with_issues
                .filter(app => app.health_status !== 'Healthy')
                .map(app => ({
                    name: app.name,
                    namespace: app.namespace,
                    status: app.health_status
                }));
        } else if (podsData && Array.isArray(podsData)) {
            // Fallback to pod status heuristic
            const problematic = podsData.filter(pod => {
                const status = pod.status;
                return status !== 'Running' && status !== 'Succeeded' && status !== 'Completed';
            });

            const seen = new Set();
            problematic.forEach(pod => {
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
                        status: pod.status
                    });
                }
            });
        }

        if (degradedApps.length === 0) {
            listContainer.style.display = 'none';
            return;
        }

        listContainer.style.display = 'flex';

        let html = '';
        degradedApps.forEach(app => {
            const argoUrl = `https://argocd.p.zacharie.org/applications/${app.name}`;
            html += `
                <a href="${argoUrl}" target="_blank" class="degraded-app-entry" title="Status: ${app.status}">
                    <span class="degraded-app-name" style="color: var(--status-critical); font-weight: bold;">${app.name}</span>
                    <span class="degraded-app-status" style="font-size: 0.7rem; opacity: 0.7;">[${app.status}]</span>
                </a>
            `;
        });

        listContainer.innerHTML = html;
    },

    renderSecurityScore(metricsData) {
        if (!metricsData) return;

        const score = metricsData.security_score || 0;
        const scoreEl = document.getElementById('overview-security-score');
        if (scoreEl) {
            scoreEl.textContent = `${score.toFixed(0)}%`;

            // Color coding
            scoreEl.className = 'security-score-value';
            if (score > 80) scoreEl.classList.add('status-good');
            else if (score > 50) scoreEl.classList.add('status-warning');
            else scoreEl.classList.add('status-critical');
        }

        const details = metricsData.security_details;
        const failedJobsCount = metricsData.failed_jobs_count || 0;
        const failedJobsList = metricsData.failed_jobs_list || [];

        if (details) {
            document.getElementById('overview-trivy-score').textContent = `${(details.trivy_score || 0).toFixed(0)}%`;
            document.getElementById('overview-compliance-score').textContent = `${(details.steampipe_score || 0).toFixed(0)}%`;
        }

        const failedJobsEl = document.getElementById('overview-failed-jobs-count');
        if (failedJobsEl) {
            failedJobsEl.textContent = failedJobsCount;
            failedJobsEl.className = failedJobsCount > 0 ? 'status-critical' : 'status-good';
        }

        // 1. Manage Global Alert Card for Failed Jobs
        const alertCard = document.getElementById('overview-failed-jobs-alert');
        if (alertCard) {
            if (failedJobsCount > 0) {
                alertCard.style.display = 'block';
                const alertTitle = document.getElementById('overview-alert-title');
                const alertSummary = document.getElementById('overview-alert-summary');
                const alertDetails = document.getElementById('overview-alert-details');

                if (alertTitle) {
                    alertTitle.innerHTML = `⚠️ ATTENTION: ${failedJobsCount} jobs en échec détectés (Pénalité appliquée)`;
                }

                if (alertSummary) {
                    let summaryHtml = '';
                    if (details && details.steampipe_stats) {
                        const stats = details.steampipe_stats;
                        summaryHtml += `
                            <div style="font-size: 0.95rem; color: #a0aec0; margin-bottom: 5px;">
                                <span style="color: #48bb78;">${stats.passed || 0} tests réussis</span> / 
                                <span style="color: #f56565;">${stats.failed || 0} échecs conformité</span>
                            </div>
                            <div style="font-size: 0.95rem; color: #718096; margin-bottom: 15px;">
                                ${metricsData.trivy_critical_count || 0} Vulnerabilités Critiques (Filtrées)
                            </div>
                        `;
                    }
                    alertSummary.innerHTML = summaryHtml;
                }

                if (alertDetails) {
                    let html = '<div style="font-size: 0.85rem; font-weight: bold; color: #f56565; margin-bottom: 0.5rem; text-transform: uppercase; border-bottom: 1px dashed rgba(245,101,101,0.3); padding-bottom: 5px;">❌ Recent Failed Jobs:</div>';
                    html += '<div class="failed-jobs-full-list">';
                    failedJobsList.forEach(job => {
                        html += `
                            <div class="failed-job-full-item">
                                <span class="job-name">${job.job_name}</span>
                                <span class="job-ns">${job.namespace}</span>
                            </div>
                        `;
                    });
                    html += '</div>';
                    alertDetails.innerHTML = html;
                }
            } else {
                alertCard.style.display = 'none';
            }
        }

        // 2. Summary text inside Security Card (as fallback or secondary info)
        const summaryEl = document.getElementById('overview-security-summary');
        if (summaryEl) {
            if (failedJobsCount > 0) {
                summaryEl.innerHTML = `
                    <div style="color: #f56565; font-weight: bold; margin-bottom: 5px; margin-top: 1rem; border-top: 1px dashed #2a3548; padding-top: 1rem;">
                        ⚠️ ${failedJobsCount} jobs en échec
                    </div>
                `;
            } else {
                summaryEl.innerHTML = `
                    <div style="color: #48bb78; font-weight: bold; margin-bottom: 5px; margin-top: 1rem; border-top: 1px dashed #2a3548; padding-top: 1rem;">
                        ✅ Tous les contrôles de santé sont au vert
                    </div>
                `;
            }

            if (details && details.steampipe_stats) {
                const stats = details.steampipe_stats;
                summaryEl.innerHTML += `
                    <div style="font-size: 0.75rem; color: #a0aec0;">
                        ${stats.passed || 0} tests réussis / ${stats.failed || 0} échecs conformité<br>
                        ${metricsData.trivy_critical_count || 0} Vulnerabilités Critiques (Filtrées)
                    </div>
                `;
            }
        }

        // Clear the old list inside the security card to avoid duplication
        const listContainer = document.getElementById('overview-failed-jobs-list');
        if (listContainer) listContainer.innerHTML = '';
    },

    renderPipelines(data) {
        console.log('📦 Rendering pipelines with data:', data);
        const listEl = document.getElementById('overview-pipelines-list');
        if (!listEl) return;

        // Handle both old array format and new object format
        const pipelines = (data && data.pipelines) || (Array.isArray(data) ? data : []);
        const repoStats = (data && data.repo_stats) || {};

        if (pipelines.length === 0 && Object.keys(repoStats).length === 0) {
            listEl.innerHTML = '<div class="no-data" style="color:#a0aec0;font-size:0.85rem;">No recent pipelines found.</div>';
            return;
        }

        const renderRepoLine = (repoName, searchStr) => {
            const filtered = Array.isArray(pipelines) ? pipelines : [];
            const repoPipelines = filtered.filter(p => (p.repo || '').toLowerCase().includes(searchStr.toLowerCase())).slice(0, 5);
            const stats = repoStats[repoName] || { open_prs: 0, prs_url: `https://github.com/JZacharie/${repoName}/pulls` };

            let iconsHtml = '';
            if (repoPipelines.length === 0) {
                iconsHtml = '<span style="color:#718096;font-size:0.75rem;font-style:italic;">No recent runs</span>';
            } else {
                repoPipelines.forEach(run => {
                    const statusClass = run.status === 'completed'
                        ? (run.conclusion === 'success' ? 'health-good' : 'status-critical')
                        : 'status-warning';
                    const icon = run.status === 'completed'
                        ? (run.conclusion === 'success' ? '✅' : '❌')
                        : '⏳';

                    const tooltip = `${run.name || 'Workflow'} - ${run.status}${run.conclusion ? ' (' + run.conclusion + ')' : ''}\n${new Date(run.created_at).toLocaleString()}`;

                    iconsHtml += `
                        <a href="${run.url}" target="_blank" class="pipeline-mini-icon ${statusClass}" title="${tooltip}">
                            ${icon}
                        </a>`;
                });
            }

            return `
                <div class="pipeline-repo-line" style="display: flex; flex-direction: column; background: #151b28; padding: 0.8rem; border-radius: 6px; border: 1px solid #2a3548; margin-bottom: 0.8rem;">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px;">
                        <div class="pipeline-repo-name" style="font-weight: bold; font-size: 0.9rem; display: flex; align-items: center; gap: 0.5rem;">
                            <span>📂 ${repoName}</span>
                            <a href="${stats.prs_url}" target="_blank" style="font-size: 0.75rem; font-weight: normal; color: #4299e1; text-decoration: none; background: rgba(66, 153, 225, 0.1); padding: 2px 6px; border-radius: 10px; border: 1px solid rgba(66, 153, 225, 0.2);">
                                🔄 ${stats.open_prs} Open PRs
                            </a>
                        </div>
                        <div class="pipeline-status-icons" style="display: flex; gap: 6px;">${iconsHtml}</div>
                    </div>
                </div>
            `;
        };

        listEl.innerHTML = renderRepoLine('helmscharts', 'helmscharts') + renderRepoLine('Kusanagi', 'kusanagi');
    }
};

window.OverviewDashboard = OverviewDashboard;
