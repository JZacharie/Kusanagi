// Home Assistant Dashboard Module
const HomeAssistantDashboard = {
    refreshInterval: null,

    init() {
        this.fetchAndRender();
        if (this.refreshInterval) clearInterval(this.refreshInterval);
        this.refreshInterval = setInterval(() => this.fetchAndRender(), 30000);
        console.log('✅ Home Assistant Dashboard initialized');
    },

    async fetchAndRender() {
        try {
            // Fetch all data in parallel
            const [sensors, automations, devices] = await Promise.all([
                fetch('/api/ha/sensors').then(r => r.json()),
                fetch('/api/ha/automations').then(r => r.json()),
                fetch('/api/ha/devices').then(r => r.json())
            ]);

            this.renderStats(sensors, automations, devices);
            this.renderSensors(sensors);
            this.renderAutomations(automations);
        } catch (error) {
            console.error('Failed to fetch Home Assistant data:', error);
            document.getElementById('ha-sensors-content').innerHTML =
                `<div class="error">Failed to load Home Assistant data: ${error.message}</div>`;
        }
    },

    renderStats(sensors, automations, devices) {
        document.getElementById('ha-sensors').textContent = sensors.length || '0';
        document.getElementById('ha-automations').textContent = automations.length || '0';
        document.getElementById('ha-devices').textContent = devices.length || '0';
    },

    renderSensors(sensors) {
        const container = document.getElementById('ha-sensors-content');
        document.getElementById('ha-sensors-count').textContent = sensors.length;

        if (!sensors || sensors.length === 0) {
            container.innerHTML = '<div class="no-issues">No sensors found</div>';
            return;
        }

        // Group sensors by device_class or domain
        const categories = {
            'temperature': [],
            'humidity': [],
            'battery': [],
            'energy': [],
            'connectivity': [],
            'other': []
        };

        const domainGroups = {};

        sensors.forEach(sensor => {
            const deviceClass = sensor.attributes.device_class;
            const domain = sensor.entity_id.split('.')[0];

            if (categories[deviceClass]) {
                categories[deviceClass].push(sensor);
            } else if (domain === 'binary_sensor' && (deviceClass === 'connectivity' || deviceClass === 'problem' || deviceClass === 'update')) {
                categories['connectivity'].push(sensor);
            } else {
                if (!domainGroups[domain]) domainGroups[domain] = [];
                domainGroups[domain].push(sensor);
            }
        });

        let html = '<div class="ha-dashboard-container" style="display: flex; flex-direction: column; gap: 2rem;">';

        // Render prioritized categories
        const categoryOrder = ['temperature', 'humidity', 'battery', 'energy', 'connectivity'];
        categoryOrder.forEach(cat => {
            if (categories[cat].length > 0) {
                html += `
                    <div class="ha-category-section">
                        <h3 style="color: var(--neon-cyan); border-bottom: 1px solid rgba(0, 255, 249, 0.3); padding-bottom: 0.5rem; margin-bottom: 1rem; text-transform: uppercase; font-size: 0.9rem;">
                            ${cat} (${categories[cat].length})
                        </h3>
                        <div class="sensors-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1rem;">
                            ${categories[cat].map(s => this.renderSensorCard(s)).join('')}
                        </div>
                    </div>
                `;
            }
        });

        // Render other domains
        Object.keys(domainGroups).forEach(domain => {
            if (domainGroups[domain].length > 0) {
                html += `
                    <div class="ha-category-section">
                        <h3 style="color: var(--neon-magenta); border-bottom: 1px solid rgba(255, 0, 255, 0.2); padding-bottom: 0.5rem; margin-bottom: 1rem; text-transform: uppercase; font-size: 0.8rem;">
                            ${domain} (${domainGroups[domain].length})
                        </h3>
                        <div class="sensors-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1rem;">
                            ${domainGroups[domain].slice(0, 10).map(s => this.renderSensorCard(s)).join('')}
                        </div>
                    </div>
                `;
            }
        });

        html += '</div>';
        container.innerHTML = html;
    },

    renderSensorCard(sensor) {
        const icon = this.getSensorIcon(sensor);
        const unit = sensor.attributes.unit_of_measurement || '';
        const name = sensor.attributes.friendly_name || sensor.entity_id;
        const stateColor = this.getStateColor(sensor);

        return `
            <div class="sensor-card" style="padding: 1rem; background: rgba(0, 0, 0, 0.3); border-left: 3px solid ${stateColor}; border-radius: 2px; transition: transform 0.2s ease;">
                <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                    <div style="font-size: 1.2rem; opacity: 0.9;">${icon}</div>
                    <div style="text-align: right;">
                        <span style="font-size: 1.1rem; font-weight: bold; color: ${stateColor};">${sensor.state}</span>
                        <span style="font-size: 0.8rem; opacity: 0.6; margin-left: 2px;">${unit}</span>
                    </div>
                </div>
                <div style="margin-top: 0.75rem; font-size: 0.85rem; font-weight: 500; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;" title="${name}">${name}</div>
                <div style="margin-top: 0.25rem; font-size: 0.7rem; opacity: 0.4;">${this.formatTime(sensor.last_updated)}</div>
            </div>
        `;
    },

    getSensorIcon(sensor) {
        if (sensor.attributes.icon) return sensor.attributes.icon;

        const deviceClass = sensor.attributes.device_class;
        const icons = {
            'temperature': '🌡️',
            'humidity': '💧',
            'battery': '🔋',
            'energy': '⚡',
            'connectivity': '🌐',
            'signal_strength': '📶',
            'timestamp': '🕒'
        };

        return icons[deviceClass] || '📊';
    },

    getStateColor(sensor) {
        const state = sensor.state.toLowerCase();
        if (state === 'on' || state === 'home' || state === 'locked') return 'var(--neon-green)';
        if (state === 'off' || state === 'not_home' || state === 'unlocked') return 'var(--neon-magenta)';
        if (state === 'unavailable' || state === 'unknown') return '#666';

        // Dynamic colors for numeric values
        if (!isNaN(parseFloat(state))) {
            return 'var(--neon-cyan)';
        }

        return 'var(--neon-cyan)';
    },

    renderAutomations(automations) {
        const container = document.getElementById('ha-automations-content');
        document.getElementById('ha-automations-count').textContent = automations.length;

        if (!automations || automations.length === 0) {
            container.innerHTML = '<div class="no-issues">No automations found</div>';
            return;
        }

        const table = `
            <table class="data-table" style="width: 100%; border-collapse: separate; border-spacing: 0 4px;">
                <thead>
                    <tr style="text-align: left; opacity: 0.6; font-size: 0.8rem;">
                        <th style="padding: 0.5rem;">Name</th>
                        <th style="padding: 0.5rem;">State</th>
                        <th style="padding: 0.5rem;">Last Triggered</th>
                        <th style="padding: 0.5rem;">Mode</th>
                    </tr>
                </thead>
                <tbody>
                    ${automations.map(auto => `
                        <tr style="background: rgba(0, 255, 249, 0.03);">
                            <td style="padding: 0.75rem; border-left: 2px solid var(--neon-cyan);"><strong>${auto.attributes.friendly_name || auto.entity_id}</strong></td>
                            <td style="padding: 0.75rem;"><span class="status-badge ${auto.state === 'on' ? 'healthy' : 'info'}">${auto.state}</span></td>
                            <td style="padding: 0.75rem; font-size: 0.8rem; opacity: 0.8;">${auto.last_triggered ? this.formatTime(auto.last_triggered) : 'Never'}</td>
                            <td style="padding: 0.75rem; font-size: 0.8rem; opacity: 0.6;">${auto.attributes.mode || 'single'}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;

        container.innerHTML = table;
    },

    formatTime(timestamp) {
        if (!timestamp) return 'N/A';
        try {
            const date = new Date(timestamp);
            const now = new Date();
            const diff = Math.floor((now - date) / 1000);

            if (diff < 60) return `${diff}s ago`;
            if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
            if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
            return `${Math.floor(diff / 86400)}d ago`;
        } catch (e) {
            return timestamp;
        }
    }
};

// Auto-load when tab is switched
window.HomeAssistantDashboard = HomeAssistantDashboard;
