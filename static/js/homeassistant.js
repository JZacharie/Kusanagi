// Home Assistant Dashboard Module
const HomeAssistantDashboard = {
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

        // Group sensors by domain
        const grouped = {};
        sensors.forEach(sensor => {
            const domain = sensor.entity_id.split('.')[0];
            if (!grouped[domain]) grouped[domain] = [];
            grouped[domain].push(sensor);
        });

        let html = '<div class="sensors-grid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1rem;">';

        Object.keys(grouped).slice(0, 20).forEach(domain => {
            grouped[domain].slice(0, 5).forEach(sensor => {
                const icon = sensor.attributes.icon || '📊';
                const unit = sensor.attributes.unit_of_measurement || '';
                const name = sensor.attributes.friendly_name || sensor.entity_id;

                html += `
                    <div class="sensor-card" style="padding: 1rem; background: rgba(0, 255, 136, 0.05); border: 1px solid var(--neon-green); border-radius: 4px;">
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <span style="font-size: 1.5rem;">${icon}</span>
                            <span style="font-size: 1.2rem; font-weight: bold; color: var(--neon-green);">${sensor.state} ${unit}</span>
                        </div>
                        <div style="margin-top: 0.5rem; font-size: 0.9rem; opacity: 0.8;">${name}</div>
                        <div style="margin-top: 0.25rem; font-size: 0.75rem; opacity: 0.6;">Updated: ${this.formatTime(sensor.last_updated)}</div>
                    </div>
                `;
            });
        });

        html += '</div>';
        container.innerHTML = html;
    },

    renderAutomations(automations) {
        const container = document.getElementById('ha-automations-content');
        document.getElementById('ha-automations-count').textContent = automations.length;

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
                        <th>Mode</th>
                    </tr>
                </thead>
                <tbody>
                    ${automations.map(auto => `
                        <tr>
                            <td><strong>${auto.attributes.friendly_name || auto.entity_id}</strong></td>
                            <td><span class="status-badge ${auto.state === 'on' ? 'healthy' : 'info'}">${auto.state}</span></td>
                            <td>${auto.last_triggered ? this.formatTime(auto.last_triggered) : 'Never'}</td>
                            <td>${auto.attributes.mode || 'single'}</td>
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
