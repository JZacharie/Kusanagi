/**
 * Home Assistant Dashboard Module
 * Note: Polling is handled by TabManager (tab-aware)
 */
const HomeAssistantDashboard = {
    init() {
        console.log('✅ Home Assistant Dashboard initialized (no internal polling)');

        // Listen for Esc key to close modal
        window.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') this.closeDetail();
        });
    },

    // Alias pour TabManager
    loadData() {
        return this.fetchAndRender();
    },

    async fetchAndRender() {
        if (document.hidden) return;
        try {
            // Use apiFetch to get unwrapped data from the standard envelope
            const [sensorsData, automationsData, devicesData] = await Promise.all([
                api.get('/api/ha/sensors').catch(() => ({ sensors: [], count: 0 })),
                api.get('/api/ha/automations').catch(() => ({ automations: [], count: 0 })),
                api.get('/api/ha/devices').catch(() => ({ devices: [], count: 0 }))
            ]);

            // Extract arrays from response objects (API returns {sensors: [...], count: N})
            const sensors = sensorsData.sensors || [];
            const automations = automationsData.automations || [];
            const devices = devicesData.devices || [];

            this.sensors = sensors; // Store for detail view
            this.renderStats(sensors, automations, devices);
            this.renderSensors(sensors);
            this.renderAutomations(automations);
        } catch (error) {
            console.error('Failed to fetch Home Assistant data:', error);
            const content = document.getElementById('ha-sensors-content');
            if (content) {
                content.innerHTML = `<div class="error">Failed to load Home Assistant data: ${error.message}</div>`;
            }
        }
    },

    renderStats(sensors, automations, devices) {
        const sensorsEl = document.getElementById('ha-sensors');
        const automationsEl = document.getElementById('ha-automations');
        const devicesEl = document.getElementById('ha-devices');

        if (sensorsEl) sensorsEl.textContent = sensors.length || '0';
        if (automationsEl) automationsEl.textContent = automations.length || '0';
        if (devicesEl) devicesEl.textContent = devices.length || '0';
    },

    getCategory(sensor) {
        const deviceClass = sensor.attributes.device_class;
        const entityId = sensor.entity_id;
        const domain = entityId.split('.')[0];

        if (deviceClass === 'temperature' || deviceClass === 'humidity' || deviceClass === 'pressure') return 'Climate';
        if (deviceClass === 'power' || deviceClass === 'energy' || deviceClass === 'current' || deviceClass === 'voltage') return 'Energy';
        if (deviceClass === 'battery') return 'Battery';
        if (deviceClass === 'connectivity' || deviceClass === 'signal_strength' || domain === 'binary_sensor') return 'Security & Status';
        if (deviceClass === 'timestamp' || domain === 'update') return 'System';

        return 'Other Sensors';
    },

    getMDIIcon(sensor) {
        const deviceClass = sensor.attributes.device_class;
        const domain = sensor.entity_id.split('.')[0];
        const state = String(sensor.state).toLowerCase();

        const iconMap = {
            'temperature': 'mdi-thermometer',
            'humidity': 'mdi-water-percent',
            'battery': state === 'on' ? 'mdi-battery-check' : 'mdi-battery',
            'power': 'mdi-flash',
            'energy': 'mdi-lightning-bolt',
            'connectivity': state === 'on' ? 'mdi-lan-check' : 'mdi-lan-disconnect',
            'signal_strength': 'mdi-wifi',
            'timestamp': 'mdi-clock-outline',
            'pressure': 'mdi-gauge',
            'voltage': 'mdi-sine-wave',
            'current': 'mdi-current-ac',
            'update': 'mdi-package-up'
        };

        if (iconMap[deviceClass]) return iconMap[deviceClass];

        if (domain === 'binary_sensor') {
            if (deviceClass === 'motion') return state === 'on' ? 'mdi-motion-sensor' : 'mdi-motion-sensor-off';
            if (deviceClass === 'door') return state === 'on' ? 'mdi-door-open' : 'mdi-door-closed';
            if (deviceClass === 'window') return state === 'on' ? 'mdi-window-open' : 'mdi-window-closed';
            return state === 'on' ? 'mdi-check-circle' : 'mdi-close-circle';
        }

        return 'mdi-eye-outline';
    },

    renderSensors(sensors) {
        const container = document.getElementById('ha-sensors-content');
        const countEl = document.getElementById('ha-sensors-count');

        if (!container) return;

        // Ensure sensors is an array
        const sensorsArray = Array.isArray(sensors) ? sensors : [];
        if (countEl) countEl.textContent = sensorsArray.length;

        if (sensorsArray.length === 0) {
            container.innerHTML = '<div class="no-issues">No sensors found</div>';
            return;
        }

        const grouped = {};
        sensorsArray.filter(s => s.state !== 'unknown').forEach(s => {
            const cat = this.getCategory(s);
            if (!grouped[cat]) grouped[cat] = [];
            grouped[cat].push(s);
        });

        const categories = Object.keys(grouped).sort((a, b) => {
            const priority = { 'Climate': 1, 'Energy': 2, 'Security & Status': 3, 'Battery': 4, 'System': 5, 'Other Sensors': 6 };
            return (priority[a] || 99) - (priority[b] || 99);
        });

        let html = '<div class="ha-dashboard-container">';
        categories.forEach(cat => {
            html += `
                <div class="ha-category-section">
                    <div class="ha-category-title">
                        <span class="mdi ${this.getCategoryIcon(cat)}"></span>
                        ${cat}
                    </div>
                    <div class="sensors-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 1rem;">
                        ${grouped[cat].map(s => this.renderSensorCard(s)).join('')}
                    </div>
                </div>
            `;
        });
        html += '</div>';
        container.innerHTML = html;
    },

    getCategoryIcon(cat) {
        const icons = {
            'Climate': 'mdi-thermostat',
            'Energy': 'mdi-lightning-bolt-circle',
            'Battery': 'mdi-battery-charging',
            'Security & Status': 'mdi-shield-check',
            'System': 'mdi-cog',
            'Other Sensors': 'mdi-dots-horizontal-circle'
        };
        return icons[cat] || 'mdi-view-dashboard';
    },

    renderSensorCard(sensor) {
        const icon = this.getMDIIcon(sensor);
        const unit = sensor.attributes.unit_of_measurement || '';
        const name = sensor.attributes.friendly_name || sensor.entity_id;
        const color = this.getStateColor(sensor);
        const link = `https://vha.zacharie.org/config/entities?search=${sensor.entity_id}`;

        return `
            <a href="${link}" target="_blank" rel="noopener" class="sensor-icon-link" title="${name}: ${sensor.state} ${unit}">
                <div class="sensor-card-mini" style="border-color: ${color}44;">
                    <span class="mdi ${icon}" style="font-size: 1.5rem; color: ${color}; margin-right: 0.5rem;"></span>
                    <div class="sensor-mini-info">
                        <div class="sensor-mini-name">${name}</div>
                        <div class="sensor-mini-value" style="color: ${color};">${sensor.state} ${unit}</div>
                    </div>
                    <div class="sensor-mini-state" style="background: ${color};"></div>
                </div>
            </a>
        `;
    },

    showDetail(entityId) {
        const sensor = this.sensors.find(s => s.entity_id === entityId);
        if (!sensor) return;

        const modal = document.getElementById('ha-detail-modal');
        const body = document.getElementById('ha-modal-body');

        const attributes = Object.entries(sensor.attributes)
            .map(([k, v]) => `
                <tr>
                    <td class="ha-attr-label">${k}</td>
                    <td class="ha-attr-value">${typeof v === 'object' ? JSON.stringify(v) : v}</td>
                </tr>
            `).join('');

        body.innerHTML = `
            <h2 style="color: var(--neon-cyan); margin-bottom: 0.5rem;">${sensor.attributes.friendly_name || sensor.entity_id}</h2>
            <div style="font-size: 0.8rem; opacity: 0.5; margin-bottom: 1.5rem;">${sensor.entity_id}</div>
            
            <div style="font-size: 2.5rem; font-weight: bold; color: ${this.getStateColor(sensor)}; margin-bottom: 2rem;">
                ${sensor.state} <span style="font-size: 1rem; opacity: 0.6;">${sensor.attributes.unit_of_measurement || ''}</span>
            </div>

            <h3 style="font-size: 0.9rem; text-transform: uppercase; letter-spacing: 1px; color: var(--text-secondary); border-bottom: 1px solid rgba(255,255,255,0.1); padding-bottom: 0.5rem;">Attributes</h3>
            <table class="ha-attr-table">
                ${attributes}
                <tr>
                    <td class="ha-attr-label">Last Updated</td>
                    <td class="ha-attr-value">${new Date(sensor.last_updated).toLocaleString()}</td>
                </tr>
            </table>
        `;

        modal.style.display = 'flex';
        document.body.style.overflow = 'hidden';
    },

    closeDetail() {
        const modal = document.getElementById('ha-detail-modal');
        if (modal) modal.style.display = 'none';
        document.body.style.overflow = 'auto';
    },

    getStateColor(sensor) {
        const state = String(sensor.state).toLowerCase();
        if (['on', 'home', 'locked', 'open'].includes(state)) return 'var(--neon-green)';
        if (['off', 'not_home', 'unlocked', 'closed'].includes(state)) return 'var(--neon-magenta)';
        if (['unavailable', 'unknown'].includes(state)) return '#666';
        return 'var(--neon-cyan)';
    },

    renderAutomations(automations) {
        const container = document.getElementById('ha-automations-content');
        if (!container) return;

        if (!automations || automations.length === 0) {
            container.innerHTML = '<div class="no-issues">No automations found</div>';
            return;
        }

        const table = `
            <table class="issues-table">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>State</th>
                        <th>Last Triggered</th>
                    </tr>
                </thead>
                <tbody>
                    ${automations.map(auto => `
                        <tr>
                            <td><strong>${auto.attributes.friendly_name || auto.entity_id}</strong></td>
                            <td><span class="status-badge ${auto.state === 'on' ? 'healthy' : 'info'}">${auto.state}</span></td>
                            <td>${auto.last_triggered ? this.formatTime(auto.last_triggered) : 'Never'}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;
        container.innerHTML = table;
    },

    formatTime(timestamp) {
        try {
            const date = new Date(timestamp);
            const now = new Date();
            const diff = Math.floor((now - date) / 1000);
            if (diff < 60) return `${diff}s ago`;
            if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
            if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
            return `${Math.floor(diff / 86400)}d ago`;
        } catch (e) { return timestamp; }
    }
};

// Auto-load when tab is switched
window.HomeAssistantDashboard = HomeAssistantDashboard;
