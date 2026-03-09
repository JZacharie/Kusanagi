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
        this.renderBusinessKPIs();
        this.renderBusinessBI();
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
