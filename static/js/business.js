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
        if (!document.getElementById('user-world-map')) return;

        const countryData = [
            { id: 'FR', name: 'France', users: 4500, investorsCount: 120, totalInvested: 2450000, performance: '+12.5%', geo: 'Europe' },
            { id: 'US', name: 'USA', users: 3200, investorsCount: 85, totalInvested: 5800000, performance: '+15.2%', geo: 'NA' },
            { id: 'GB', name: 'UK', users: 1800, investorsCount: 45, totalInvested: 1200000, performance: '+8.7%', geo: 'Europe' },
            { id: 'DE', name: 'Germany', users: 1500, investorsCount: 38, totalInvested: 950000, performance: '+10.1%', geo: 'Europe' },
            { id: 'JP', name: 'Japan', users: 1200, investorsCount: 25, totalInvested: 1100000, performance: '+6.4%', geo: 'Asia' },
            { id: 'CN', name: 'China', users: 950, investorsCount: 15, totalInvested: 750000, performance: '+18.9%', geo: 'Asia' },
            { id: 'IN', name: 'India', users: 800, investorsCount: 12, totalInvested: 450000, performance: '+22.5%', geo: 'Asia' },
            { id: 'CA', name: 'Canada', users: 650, investorsCount: 18, totalInvested: 550000, performance: '+11.2%', geo: 'NA' },
            { id: 'BR', name: 'Brazil', users: 500, investorsCount: 8, totalInvested: 150000, performance: '+5.3%', geo: 'SA' }
        ];

        const svgStart = `<svg viewBox="0 0 1000 500" xmlns="http://www.w3.org/2000/svg" style="width: 100%; height: 100%;">`;
        const svgEnd = `</svg>`;

        const regions = [
            { id: 'NA', path: 'M200,100 L350,100 L350,250 L200,250 Z', color: 'rgba(0, 255, 249, 0.4)', label: 'North America' },
            { id: 'SA', path: 'M300,280 L400,280 L350,450 L250,450 Z', color: 'rgba(0, 255, 249, 0.2)', label: 'South America' },
            { id: 'EUR', path: 'M450,100 L550,100 L550,220 L450,220 Z', color: 'rgba(0, 255, 249, 0.8)', label: 'Europe' },
            { id: 'AFR', path: 'M450,240 L550,240 L500,420 L400,420 Z', color: 'rgba(0, 255, 249, 0.1)', label: 'Africa' },
            { id: 'ASIA', path: 'M600,80 L850,80 L800,320 L600,320 Z', color: 'rgba(0, 255, 249, 0.5)', label: 'Asia' },
            { id: 'OCE', path: 'M750,350 L850,350 L820,450 L720,450 Z', color: 'rgba(0, 255, 249, 0.1)', label: 'Oceania' }
        ];

        const generateMapHtml = (isFinance) => {
            let paths = '';
            regions.forEach(c => {
                const color = isFinance ? c.color.replace('0, 255, 249', '236, 201, 75') : c.color;
                paths += `<path d="${c.path}" class="country" fill="${color}" title="${c.label}">
                    <title>${c.label}</title>
                </path>`;
            });
            return svgStart + paths + svgEnd;
        };

        const userMapEl = document.getElementById('user-world-map');
        const financeMapEl = document.getElementById('finance-world-map');

        if (userMapEl) userMapEl.innerHTML = generateMapHtml(false);
        if (financeMapEl) financeMapEl.innerHTML = generateMapHtml(true);

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
