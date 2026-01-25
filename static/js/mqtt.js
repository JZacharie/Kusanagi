const MqttManager = {
    devices: [],
    messages: [],
    filter: '',
    isInitialized: false,
    messageBuffer: [],

    init: async function () {
        if (this.isInitialized) return;

        console.log('Initializing MqttManager...');
        await this.fetchInitialData();
        this.isInitialized = true;

        // Listen for new messages via the global WebSocket handler if possible
        // Or we just rely on the periodic refresh if WS isn't unified yet
        // In Kusanagi, WS sends 'mqtt_message' types now.
    },

    fetchInitialData: async function () {
        try {
            const [devRes, msgRes] = await Promise.all([
                fetch('/api/mqtt/devices'),
                fetch('/api/mqtt/messages')
            ]);

            this.devices = await devRes.json();
            this.messageBuffer = await msgRes.json();

            this.render();
        } catch (error) {
            console.error('Failed to fetch MQTT data:', error);
        }
    },

    handleWsMessage: function (msg) {
        if (msg.type === 'mqtt_message') {
            console.log('📡 MQTT Message Received:', msg.topic, msg.payload);
            const newMsg = {
                topic: msg.topic,
                payload: msg.payload,
                timestamp: msg.timestamp
            };

            this.messageBuffer.unshift(newMsg);
            if (this.messageBuffer.length > 500) {
                this.messageBuffer.pop();
            }

            // Update device info locally to avoid full fetch
            this.updateDeviceFromMessage(newMsg);

            if (window.KusanagiDashboard && window.KusanagiDashboard.activeTab === 'mqtt') {
                this.render();
                // Also update log modal if it is open
                const modal = document.getElementById('mqtt-logs-modal');
                if (modal && modal.style.display === 'flex') {
                    this.renderLogsModalContent();
                }
            }
        }
    },

    updateDeviceFromMessage: function (msg) {
        const deviceId = msg.topic.split('/')[0];
        let device = this.devices.find(d => d.id === deviceId);

        if (device) {
            device.last_seen = msg.timestamp;
            device.last_topic = msg.topic;
            device.message_count++;
        } else {
            this.devices.push({
                id: deviceId,
                name: deviceId,
                last_seen: msg.timestamp,
                last_topic: msg.topic,
                message_count: 1
            });
        }
    },

    applyFilter: function () {
        this.filter = document.getElementById('mqtt-filter-input').value.toLowerCase();
        this.render();
    },

    clearFlux: function () {
        this.messageBuffer = [];
        this.render();
    },

    render: function () {
        this.renderStats();
        this.renderDevices();
        this.renderFlux();
    },

    renderStats: function () {
        document.getElementById('mqtt-device-count').textContent = this.devices.length;
        document.getElementById('mqtt-device-table-count').textContent = this.devices.length;
        document.getElementById('mqtt-total-msg').textContent = this.messageBuffer.length;

        // Simple rate calculation
        const now = new Date();
        const minAgo = new Date(now - 60000);
        const rate = this.messageBuffer.filter(m => new Date(m.timestamp) > minAgo).length;
        document.getElementById('mqtt-msg-rate').textContent = rate;
    },

    renderDevices: function () {
        const container = document.getElementById('mqtt-devices-content');
        if (!container) return;

        if (this.devices.length === 0) {
            container.innerHTML = '<div class="no-issues">No devices detected yet.</div>';
            return;
        }

        // Sort by last seen
        const sorted = [...this.devices].sort((a, b) => new Date(b.last_seen) - new Date(a.last_seen));

        container.innerHTML = sorted.map(dev => `
            <div class="device-card" style="padding: 0.8rem; border-bottom: 1px solid rgba(0, 255, 249, 0.1); margin-bottom: 0.5rem; background: rgba(0,0,0,0.2);">
                <div style="display: flex; justify-content: space-between; align-items: start;">
                    <strong style="color: var(--neon-cyan);">${dev.name}</strong>
                    <span style="font-size: 0.7rem; opacity: 0.6;">${dev.message_count} msgs</span>
                </div>
                <div style="font-size: 0.75rem; margin-top: 0.3rem; word-break: break-all;">
                    <span style="opacity: 0.5;">Last:</span> ${dev.last_topic}
                </div>
                <div style="font-size: 0.7rem; margin-top: 0.2rem; text-align: right; opacity: 0.5;">
                    ${this.formatTime(dev.last_seen)}
                </div>
            </div>
        `).join('');
    },

    renderFlux: function () {
        const container = document.getElementById('mqtt-flux-content');
        if (!container) return;

        let filtered = this.messageBuffer;
        if (this.filter) {
            filtered = this.messageBuffer.filter(m =>
                m.topic.toLowerCase().includes(this.filter) ||
                m.payload.toLowerCase().includes(this.filter)
            );
        }

        if (filtered.length === 0) {
            container.innerHTML = '<div class="no-issues">No messages matching filter.</div>';
            return;
        }

        container.innerHTML = filtered.map(msg => `
            <div class="flux-line" style="margin-bottom: 0.2rem; border-left: 2px solid var(--neon-magenta); padding-left: 0.5rem;">
                <span style="color: #888; font-size: 0.75rem;">[${this.formatTimeOnly(msg.timestamp)}]</span>
                <span style="color: var(--neon-magenta); font-weight: bold;">${msg.topic}</span>
                <span style="color: var(--neon-cyan); margin-left: 0.5rem;">${this.escapeHtml(msg.payload)}</span>
            </div>
        `).join('');
    },

    formatTime: function (ts) {
        if (!ts) return 'never';
        const date = new Date(ts);
        return date.toLocaleString();
    },

    formatTimeOnly: function (ts) {
        const date = new Date(ts);
        return date.toLocaleTimeString();
    },

    escapeHtml: function (unsafe) {
        return unsafe
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    },

    openLogsModal: function () {
        document.getElementById('mqtt-logs-modal').style.display = 'flex';
        this.renderLogsModalContent();
    },

    closeLogsModal: function () {
        document.getElementById('mqtt-logs-modal').style.display = 'none';
    },

    renderLogsModalContent: function () {
        const container = document.getElementById('mqtt-modal-log-content');
        if (!container) return;

        // Show only the last 30 messages
        const last30 = this.messageBuffer.slice(0, 30);

        if (last30.length === 0) {
            container.innerHTML = '<div style="text-align: center; margin-top: 2rem; opacity: 0.5;">No messages yet.</div>';
            return;
        }

        container.innerHTML = last30.map(msg => `
            <div style="border-bottom: 1px solid rgba(255, 0, 128, 0.1); padding: 0.5rem 0;">
                <span style="color: #888; font-size: 0.8rem;">[${this.formatTimeOnly(msg.timestamp)}]</span>
                <span style="color: var(--neon-magenta); font-weight: bold;">${msg.topic}</span>
                <div style="color: var(--neon-cyan); margin-left: 1rem; word-break: break-all;">${this.escapeHtml(msg.payload)}</div>
            </div>
        `).join('');
    }
};

// Hook into the global WebSocket message handling
window.addEventListener('kusanagi-ws-message', (e) => {
    if (MqttManager) {
        MqttManager.handleWsMessage(e.detail);
    }
});
