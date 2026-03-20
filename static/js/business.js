/**
 * Kusanagi Business Dashboard
 * Handles Business KPIs, World Maps and Financial Data
 */

const BusinessDashboard = {
    initialized: false,
    updateInterval: null,
    charts: {},

    init() {
        if (this.initialized) return;
        console.log('📈 Business Dashboard initializing...');

        // Generic Zone Collapsibility
        document.querySelectorAll('.zone-title').forEach(header => {
            header.addEventListener('click', () => {
                header.classList.toggle('collapsed');
            });
        });

        this.initialized = true;
        this.refreshData();

        // Refresh every 5 minutes
        this.updateInterval = setInterval(() => this.refreshData(), 300000);
    },

    async refreshData() {
        if (!document.getElementById('biz-total-users')) return; // Not on page

        this.updateTimestamp();
        
        try {
            const response = await fetch('/api/business/cloudflare');
            if (response.ok) {
                const data = await response.json();
                this.renderCloudflareKPIs(data);
            } else {
                console.error('Failed to fetch Cloudflare data');
                this.renderBusinessKPIs(); // Fallback to mock
            }
        } catch (e) {
            console.error('Error fetching Cloudflare data:', e);
            this.renderBusinessKPIs(); // Fallback to mock
        }
        
        this.renderBusinessBI();
    },

    renderCloudflareKPIs(data) {
        const cf = data.cloudflare || {};
        
        // Update DOM with real Cloudflare data
        const setText = (id, val) => {
            const el = document.getElementById(id);
            if (el) el.textContent = val;
        };

        setText('biz-total-requests', cf.requests.toLocaleString());
        setText('biz-bandwidth', this.formatBytes(cf.bandwidth));
        setText('biz-threats', cf.threats.toLocaleString());
        setText('biz-page-views', cf.page_views.toLocaleString());
        
        // Since we don't have conversion/churn from CF easily, we can keep them mock or hide
    },

    formatBytes(bytes, decimals = 2) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const dm = decimals < 0 ? 0 : decimals;
        const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
    },

    updateTimestamp() {
        const span = document.getElementById('biz-last-update');
        if (!span) return;
        const now = new Date();
        const options = { month: 'short', day: 'numeric', year: 'numeric', hour: 'numeric', minute: '2-digit', hour12: true };
        span.textContent = now.toLocaleString('en-US', options);
    },

    renderBusinessKPIs() {
        // Mock data generation
        const totalUsers = 12450 + Math.floor(Math.random() * 50);
        const uniqueUsers = 3240 + Math.floor(Math.random() * 100);
        const dailyTransactions = 850 + Math.floor(Math.random() * 30);
        const conversionRate = 65.4 + (Math.random() * 2);
        const churnRate = 1.2 + (Math.random() * 0.5);
        const loyaltyPoints = 458200 + Math.floor(Math.random() * 1000);

        // Update DOM
        const setText = (id, val) => {
            const el = document.getElementById(id);
            if (el) el.textContent = val;
        };

        setText('biz-total-users', totalUsers.toLocaleString());
        setText('biz-unique-users', uniqueUsers.toLocaleString());
        setText('biz-daily-transactions', dailyTransactions.toLocaleString());
        setText('biz-conversion-rate', `${conversionRate.toFixed(1)}%`);
        setText('biz-churn-rate', `${churnRate.toFixed(2)}%`);
        setText('biz-loyalty-points', loyaltyPoints.toLocaleString());

        // Render Activity Chart
        if (typeof Chart === 'undefined') return;

        const ctx = document.getElementById('biz-users-chart');
        if (!ctx) return;

        if (this.charts['biz-users-chart']) {
            this.charts['biz-users-chart'].destroy();
        }

        const labels = Array.from({ length: 24 }, (_, i) => `${i}h`);
        const data = labels.map(() => 200 + Math.floor(Math.random() * 400));

        this.charts['biz-users-chart'] = new Chart(ctx, {
            type: 'line',
            data: {
                labels: labels,
                datasets: [{
                    label: 'Users per Hour',
                    data: data,
                    borderColor: '#ecc94b',
                    backgroundColor: 'rgba(236, 201, 75, 0.1)',
                    fill: true,
                    tension: 0.4,
                    pointRadius: 0
                }]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                plugins: { legend: { display: false } },
                scales: {
                    x: { display: false },
                    y: {
                        beginAtZero: true,
                        grid: { color: '#2a3548' },
                        ticks: { color: '#a0aec0', font: { size: 10 } }
                    }
                }
            }
        });
    },

    renderBusinessBI() {
        const mapEl = document.getElementById('business-global-map');
        if (!mapEl) return;

        const countryData = [
            // Europe
            { id: 'FRA', name: 'France', users: 4850, investorsCount: 142, totalInvested: 2850000, performance: '+12.5%', geo: 'Europe', latlng: [46.2276, 2.2137] },
            { id: 'DEU', name: 'Germany', users: 3800, investorsCount: 95, totalInvested: 1950000, performance: '+10.1%', geo: 'Europe', latlng: [51.1657, 10.4515] },
            { id: 'GBR', name: 'UK', users: 3100, investorsCount: 78, totalInvested: 2200000, performance: '+8.7%', geo: 'Europe', latlng: [55.3781, -3.4360] },
            { id: 'ITA', name: 'Italy', users: 2400, investorsCount: 56, totalInvested: 1100000, performance: '+7.2%', geo: 'Europe', latlng: [41.8719, 12.5674] },
            { id: 'ESP', name: 'Spain', users: 2100, investorsCount: 48, totalInvested: 950000, performance: '+9.4%', geo: 'Europe', latlng: [40.4637, -3.7492] },
            { id: 'NLD', name: 'Netherlands', users: 1850, investorsCount: 64, totalInvested: 1550000, performance: '+11.8%', geo: 'Europe', latlng: [52.1326, 5.2913] },
            { id: 'CHE', name: 'Switzerland', users: 1200, investorsCount: 82, totalInvested: 3200000, performance: '+6.5%', geo: 'Europe', latlng: [46.8182, 8.2275] },
            { id: 'BEL', name: 'Belgium', users: 1150, investorsCount: 34, totalInvested: 750000, performance: '+8.2%', geo: 'Europe', latlng: [50.5039, 4.4699] },
            { id: 'SWE', name: 'Sweden', users: 1400, investorsCount: 41, totalInvested: 1100000, performance: '+10.4%', geo: 'Europe', latlng: [60.1282, 18.6435] },
            { id: 'NOR', name: 'Norway', users: 950, investorsCount: 38, totalInvested: 1450000, performance: '+5.8%', geo: 'Europe', latlng: [60.4720, 8.4689] },
            { id: 'DNK', name: 'Denmark', users: 850, investorsCount: 29, totalInvested: 880000, performance: '+9.1%', geo: 'Europe', latlng: [56.2639, 9.5018] },
            { id: 'FIN', name: 'Finland', users: 780, investorsCount: 22, totalInvested: 650000, performance: '+7.7%', geo: 'Europe', latlng: [61.9241, 25.7481] },
            { id: 'POL', name: 'Poland', users: 1600, investorsCount: 28, totalInvested: 550000, performance: '+14.2%', geo: 'Europe', latlng: [51.9194, 19.1451] },
            { id: 'AUT', name: 'Austria', users: 920, investorsCount: 31, totalInvested: 780000, performance: '+8.9%', geo: 'Europe', latlng: [47.5162, 14.5501] },
            { id: 'PRT', name: 'Portugal', users: 1100, investorsCount: 19, totalInvested: 420000, performance: '+6.1%', geo: 'Europe', latlng: [39.3999, -8.2245] },
            { id: 'IRL', name: 'Ireland', users: 850, investorsCount: 45, totalInvested: 1850000, performance: '+13.4%', geo: 'Europe', latlng: [53.4129, -8.2439] },
            { id: 'GRC', name: 'Greece', users: 750, investorsCount: 12, totalInvested: 280000, performance: '+4.5%', geo: 'Europe', latlng: [39.0742, 21.8243] },
            { id: 'CZE', name: 'Czechia', users: 980, investorsCount: 18, totalInvested: 480000, performance: '+9.8%', geo: 'Europe', latlng: [49.8175, 15.4730] },
            { id: 'HUN', name: 'Hungary', users: 680, investorsCount: 11, totalInvested: 240000, performance: '+11.2%', geo: 'Europe', latlng: [47.1625, 19.5033] },
            { id: 'ROU', name: 'Romania', users: 1250, investorsCount: 14, totalInvested: 320000, performance: '+15.5%', geo: 'Europe', latlng: [45.9432, 24.9668] },

            // Rest of the World
            { id: 'USA', name: 'USA', users: 12500, investorsCount: 450, totalInvested: 15800000, performance: '+15.2%', geo: 'NA', latlng: [37.0902, -95.7129] },
            { id: 'JPN', name: 'Japan', users: 4200, investorsCount: 95, totalInvested: 3100000, performance: '+6.4%', geo: 'Asia', latlng: [36.2048, 138.2529] },
            { id: 'CHN', name: 'China', users: 18000, investorsCount: 120, totalInvested: 4500000, performance: '+18.9%', geo: 'Asia', latlng: [35.8617, 104.1954] },
            { id: 'IND', name: 'India', users: 22000, investorsCount: 85, totalInvested: 2250000, performance: '+22.5%', geo: 'Asia', latlng: [20.5937, 78.9629] },
            { id: 'CAN', name: 'Canada', users: 2800, investorsCount: 64, totalInvested: 1850000, performance: '+11.2%', geo: 'NA', latlng: [56.1304, -106.3468] },
            { id: 'BRA', name: 'Brazil', users: 4500, investorsCount: 42, totalInvested: 950000, performance: '+5.3%', geo: 'SA', latlng: [-14.2350, -51.9253] },
            { id: 'AUS', name: 'Australia', users: 1800, investorsCount: 52, totalInvested: 2150000, performance: '+9.7%', geo: 'Oceania', latlng: [-25.2744, 133.7751] }
        ];

        // Initialize Leaflet Map if not already done
        if (!this.map) {
            this.map = L.map('business-global-map', {
                center: [20, 0],
                zoom: 2,
                zoomControl: false,
                attributionControl: true
            });

            // Dark Matter tile layer (CartoDB)
            L.tileLayer('https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png', {
                attribution: '&copy; OpenStreetMap contributors &copy; CARTO',
                subdomains: 'abcd',
                maxZoom: 20
            }).addTo(this.map);

            // Add Zoom Control at bottom right
            L.control.zoom({ position: 'bottomright' }).addTo(this.map);

            // Force resize after init
            setTimeout(() => this.map.invalidateSize(), 100);
        }

        // Clear existing layers if any
        if (this.mapLayers) {
            this.mapLayers.forEach(layer => this.map.removeLayer(layer));
        }
        this.mapLayers = [];

        // Helper to get color based on user density (Yellow scale)
        const getUserColor = d => {
            return d > 4000 ? '#f6e05e' :
                d > 2000 ? '#faf089' :
                    d > 1000 ? '#fefcbf' :
                        '#fff9c4';
        };

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

        const tableBody = document.getElementById('bi-investment-table-body');
        if (tableBody) {
            tableBody.innerHTML = countryData.map(c => `
                <tr>
                    <td class="country-name">${c.name}</td>
                    <td>${c.investorsCount} <span class="mdi mdi-account-star" style="color: var(--neon-cyan);"></span></td>
                    <td class="finance-value">${c.totalInvested.toLocaleString()} €</td>
                    <td style="color: ${c.performance.includes('+') ? '#48bb78' : '#f56565'}; font-weight: bold;">
                        ${c.performance}
                    </td>
                    <td><span class="bi-badge">${c.geo}</span></td>
                </tr>
            `).join('');
        }
    }
};

window.BusinessDashboard = BusinessDashboard;
