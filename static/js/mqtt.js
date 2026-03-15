/**
 * MQTT Manager
 * Note: Polling is handled by TabManager (tab-aware)
 */
const MqttManager = {
    devices: [],
    messages: [],
    topics: {},
    filter: '',
    isInitialized: false,
    messageBuffer: [],
    lastUpdate: null,

    init: function () {
        if (this.isInitialized) return;
        console.log('✅ MqttManager initialized (no internal polling)');
        this.setDefaultValues();
        this.isInitialized = true;
    },

    _setText: function (id, text) {
        const el = document.getElementById(id);
        if (el) el.textContent = text;
    },

    setDefaultValues: function () {
        this._setText('mqtt-device-count', '0');
        this._setText('mqtt-device-table-count', '0');
        this._setText('mqtt-topic-count', '0');
        this._setText('mqtt-topic-table-count', '0');
        this._setText('mqtt-total-msg', '0');
        this._setText('mqtt-msg-rate', '0');
        this._setText('mqtt-last-update', '-');
    },

    // Alias pour TabManager
    loadData: function () {
        return this.fetchInitialData();
    },

    fetchInitialData: async function () {
        try {
            const [devices, messages] = await Promise.all([
                api.get('/api/mqtt/devices'),
                api.get('/api/mqtt/messages')
            ]);

            this.devices = Array.isArray(devices) ? devices : [];
            this.messageBuffer = Array.isArray(messages) ? messages : [];
            this.lastUpdate = new Date();

            // Extract topic statistics
            this.analyzeTopics();

            this.render();
        } catch (error) {
            console.error('Failed to fetch MQTT data:', error);
            this.renderError('Failed to fetch MQTT data. Is the MQTT broker configured?');
        }
    },

    // Note: Real-time WebSocket updates not implemented in backend
    // Polling via TabManager (every 30s) is sufficient

    refresh: async function () {
        return this.fetchInitialData();
    },

    analyzeTopics: function () {
        this.topics = {};
        this.messageBuffer.forEach(msg => {
            const topic = msg.topic || 'unknown';
            if (!this.topics[topic]) {
                this.topics[topic] = {
                    name: topic,
                    count: 0,
                    lastPayload: '',
                    lastSeen: null
                };
            }
            this.topics[topic].count++;
            this.topics[topic].lastPayload = msg.payload;
            this.topics[topic].lastSeen = msg.timestamp;
        });
    },

    applyFilter: function () {
        const input = document.getElementById('mqtt-filter-input');
        if (input) {
            this.filter = input.value.toLowerCase();
            this.render();
        }
    },

    clearFlux: function () {
        this.messageBuffer = [];
        this.analyzeTopics();
        this.render();
    },

    render: function () {
        this.renderStats();
        this.renderDevices();
        this.renderFlux();
        this.renderTopics();
    },

    renderStats: function () {
        this._setText('mqtt-device-count', this.devices.length);
        this._setText('mqtt-device-table-count', this.devices.length);

        const topicCount = Object.keys(this.topics).length;
        this._setText('mqtt-topic-count', topicCount);
        this._setText('mqtt-topic-table-count', topicCount);

        this._setText('mqtt-total-msg', this.messageBuffer.length);

        // Simple rate calculation (messages in last minute)
        const now = new Date();
        const minAgo = new Date(now - 60000);
        const rate = this.messageBuffer.filter(m => {
            const msgTime = new Date(m.timestamp);
            return msgTime > minAgo;
        }).length;
        this._setText('mqtt-msg-rate', rate);

        // Last update time
        if (this.lastUpdate) {
            const timeStr = this.lastUpdate.toLocaleTimeString();
            this._setText('mqtt-last-update', timeStr);
        }
    },

    renderDevices: function () {
        const container = document.getElementById('mqtt-devices-content');
        if (!container) return;

        if (this.devices.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 2rem; text-align: center;">
                    <span style="font-size: 2rem;">📱</span>
                    <p style="margin-top: 1rem;">No devices connected yet.</p>
                    <p style="color: #888; font-size: 0.8rem;">Devices will appear when they publish messages.</p>
                </div>
            `;
            return;
        }

        // Sort by last seen (most recent first)
        const sorted = [...this.devices].sort((a, b) => b.last_seen - a.last_seen);

        container.innerHTML = sorted.map(dev => `
            <div class="device-card" style="padding: 0.8rem; border-bottom: 1px solid rgba(0, 255, 249, 0.1); margin-bottom: 0.5rem; background: rgba(0,0,0,0.2); border-radius: 4px;">
                <div style="display: flex; justify-content: space-between; align-items: start;">
                    <strong style="color: var(--neon-cyan);">${this.escapeHtml(dev.name)}</strong>
                    <span style="font-size: 0.7rem; background: rgba(0,255,255,0.2); padding: 0.1rem 0.4rem; border-radius: 3px;">${dev.message_count} msgs</span>
                </div>
                <div style="font-size: 0.75rem; margin-top: 0.3rem; word-break: break-all; color: #aaa;">
                    <span style="opacity: 0.5;">Last topic:</span> ${this.escapeHtml(dev.last_topic)}
                </div>
                <div style="font-size: 0.7rem; margin-top: 0.2rem; text-align: right; opacity: 0.5; color: #666;">
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
                (m.topic || '').toLowerCase().includes(this.filter) ||
                (m.payload || '').toLowerCase().includes(this.filter)
            );
        }

        if (filtered.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 2rem; text-align: center;">
                    <span style="font-size: 2rem;">📡</span>
                    <p style="margin-top: 1rem;">No messages to display.</p>
                    ${this.filter ? '<p style="color: #888; font-size: 0.8rem;">Try clearing the filter.</p>' : '<p style="color: #888; font-size: 0.8rem;">Messages will appear here when received.</p>'}
                </div>
            `;
            return;
        }

        // Show last 100 messages only for performance
        const recent = filtered.slice(0, 100);

        container.innerHTML = recent.map(msg => {
            const topic = msg.topic || 'unknown';
            const payload = msg.payload || '';
            const timestamp = msg.timestamp || 0;

            // Do not truncate payloads (as requested by user)
            const displayPayload = payload;

            return `
            <div class="flux-line" style="margin-bottom: 0.3rem; border-left: 2px solid var(--neon-magenta); padding-left: 0.5rem; font-size: 0.8rem;">
                <span style="color: #888; font-size: 0.7rem;">[${this.formatTimeOnly(timestamp)}]</span>
                <span style="color: var(--neon-magenta); font-weight: bold; cursor: pointer;" onclick="MqttManager.filterByTopic('${this.escapeHtml(topic)}')">${this.escapeHtml(topic)}</span>
                <span style="color: var(--neon-cyan); margin-left: 0.5rem; word-break: break-all;">${this.escapeHtml(displayPayload)}</span>
            </div>
        `}).join('');
    },

    renderTopics: function () {
        const container = document.getElementById('mqtt-topics-content');
        if (!container) return;

        const topicList = Object.values(this.topics);

        if (topicList.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 2rem; text-align: center;">
                    <span style="font-size: 2rem;">🏷️</span>
                    <p style="margin-top: 1rem;">No topics yet.</p>
                    <p style="color: #888; font-size: 0.8rem;">Topics will appear as messages are received.</p>
                </div>
            `;
            return;
        }

        // Sort by message count (descending)
        const sorted = topicList.sort((a, b) => b.count - a.count);

        container.innerHTML = sorted.map(topic => `
            <div class="topic-card" style="padding: 0.6rem; border-bottom: 1px solid rgba(168, 85, 247, 0.1); margin-bottom: 0.3rem; background: rgba(0,0,0,0.2); border-radius: 4px; cursor: pointer;"
                 onclick="MqttManager.filterByTopic('${this.escapeHtml(topic.name)}')"
                 onmouseover="this.style.background='rgba(168,85,247,0.1)'" 
                 onmouseout="this.style.background='rgba(0,0,0,0.2)'">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 0.75rem; color: var(--neon-purple); font-weight: bold; word-break: break-all; flex: 1;">${this.escapeHtml(topic.name)}</span>
                    <span style="font-size: 0.7rem; background: rgba(168,85,247,0.3); padding: 0.1rem 0.4rem; border-radius: 3px; margin-left: 0.5rem;">${topic.count}</span>
                </div>
                <div style="font-size: 0.65rem; color: #666; margin-top: 0.2rem;">
                    Last: ${this.formatTime(topic.lastSeen)}
                </div>
            </div>
        `).join('');
    },

    filterByTopic: function (topic) {
        const input = document.getElementById('mqtt-filter-input');
        if (input) input.value = topic;
        this.filter = topic.toLowerCase();
        this.render();
    },

    renderError: function (message) {
        const containers = ['mqtt-devices-content', 'mqtt-flux-content', 'mqtt-topics-content'];
        containers.forEach(id => {
            const el = document.getElementById(id);
            if (el) {
                el.innerHTML = `
                    <div style="padding: 2rem; text-align: center;">
                        <span style="font-size: 2rem;">❌</span>
                        <p style="color: #ff4444; margin-top: 1rem;">${message}</p>
                        <button onclick="MqttManager.refresh()" class="cyber-btn" style="margin-top: 1rem;">🔄 Retry</button>
                    </div>
                `;
            }
        });
    },

    formatTime: function (ts) {
        if (!ts) return 'never';
        try {
            // Handle both milliseconds and seconds timestamps
            const timestamp = ts > 1000000000000 ? ts : ts * 1000;
            const date = new Date(timestamp);
            return date.toLocaleString();
        } catch (e) {
            return 'invalid';
        }
    },

    formatTimeOnly: function (ts) {
        if (!ts) return '--:--:--';
        try {
            const timestamp = ts > 1000000000000 ? ts : ts * 1000;
            const date = new Date(timestamp);
            return date.toLocaleTimeString();
        } catch (e) {
            return '--:--:--';
        }
    },

    escapeHtml: function (unsafe) {
        if (!unsafe) return '';
        return unsafe
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;")
            .replace(/"/g, "&quot;")
            .replace(/'/g, "&#039;");
    },

    openLogsModal: function () {
        const modal = document.getElementById('mqtt-logs-modal');
        if (modal) modal.style.display = 'flex';
        this.renderLogsModalContent();
    },

    closeLogsModal: function () {
        const modal = document.getElementById('mqtt-logs-modal');
        if (modal) modal.style.display = 'none';
    },

    renderLogsModalContent: function () {
        const container = document.getElementById('mqtt-modal-log-content');
        if (!container) return;

        const last30 = this.messageBuffer.slice(0, 30);

        if (last30.length === 0) {
            container.innerHTML = '<div style="text-align: center; margin-top: 2rem; opacity: 0.5;">No messages yet.</div>';
            return;
        }

        container.innerHTML = last30.map(msg => `
            <div style="border-bottom: 1px solid rgba(255, 0, 128, 0.1); padding: 0.5rem 0;">
                <span style="color: #888; font-size: 0.8rem;">[${this.formatTimeOnly(msg.timestamp)}]</span>
                <span style="color: var(--neon-magenta); font-weight: bold;">${this.escapeHtml(msg.topic || 'unknown')}</span>
                <div style="color: var(--neon-cyan); margin-left: 1rem; word-break: break-all; font-size: 0.85rem;">${this.escapeHtml(msg.payload || '')}</div>
            </div>
        `).join('');
    }
};

// MQTT Manager is ready
console.log('📡 MqttManager loaded');

// Expose to window for TabManager
window.MqttManager = MqttManager;
