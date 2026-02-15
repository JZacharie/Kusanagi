/**
 * Security Dashboard Module - Trivy Integration
 * Note: Polling is handled by TabManager (tab-aware)
 */
const SecurityDashboard = {
    init() {
        console.log('✅ Security Dashboard initialized (no internal polling)');
    },

    // Alias pour TabManager
    loadSecurityData() {
        return this.fetchAndRender();
    },

    async fetchAndRender() {
        try {
            // Use apiFetch to get unwrapped data from the standard envelope
            const vulns = await api.get('/api/security/vulnerabilities');

            // Also fetch available reports for the selector
            try {
                const reportsData = await api.get('/api/security/reports');
                this.renderReportSelector(reportsData || []);
            } catch (e) {
                console.warn('Failed to fetch reports list:', e);
            }

            this.renderStats(vulns);
            this.renderVulnerabilities(vulns);
        } catch (error) {
            console.error('Failed to fetch Trivy data:', error);
            this.renderSecurityError('Trivy security service unavailable. Please check Trivy server configuration.');
        }
    },

    renderSecurityError(message) {
        // Update stats to show N/A
        document.getElementById('security-critical-vulns').textContent = '-';
        document.getElementById('security-high-vulns').textContent = '-';
        const mediumEl = document.getElementById('security-medium-vulns');
        const lowEl = document.getElementById('security-low-vulns');
        const totalEl = document.getElementById('security-total-vulns');
        if (mediumEl) mediumEl.textContent = '-';
        if (lowEl) lowEl.textContent = '-';
        if (totalEl) totalEl.textContent = '-';

        // Render error in vulnerabilities container
        const container = document.getElementById('security-vulns-content');
        if (container) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 2rem; text-align: center;">
                    <span style="font-size: 2rem;">🛡️</span>
                    <p>Security data unavailable</p>
                    <p style="color: var(--neon-orange); margin-top: 1rem; font-size: 0.9rem;">⚠️ ${message}</p>
                    <button onclick="SecurityDashboard.fetchAndRender()" class="cyber-btn" style="margin-top: 1rem;">Retry</button>
                </div>
            `;
        }
    },

    renderStats(vulns) {
        document.getElementById('security-critical-vulns').textContent = vulns.critical || '0';
        document.getElementById('security-high-vulns').textContent = vulns.high || '0';
        const mediumEl = document.getElementById('security-medium-vulns');
        const lowEl = document.getElementById('security-low-vulns');
        const totalEl = document.getElementById('security-total-vulns');
        if (mediumEl) mediumEl.textContent = vulns.medium || '0';
        if (lowEl) lowEl.textContent = vulns.low || '0';
        if (totalEl) totalEl.textContent = vulns.total || '0';
    },

    renderReportSelector(reports) {
        const container = document.getElementById('security-report-selector');
        if (!container) return;

        if (!reports || reports.length === 0) {
            container.innerHTML = '<p style="color: var(--neon-orange); font-size: 0.9rem;">No cached reports available</p>';
            return;
        }

        const selector = `
            <div style="margin-bottom: 1rem;">
                <label for="report-select" style="margin-right: 0.5rem; color: var(--neon-cyan);">📊 Select Report:</label>
                <select id="report-select" class="cyber-select" onchange="SecurityDashboard.loadReport(this.value)" style="background: rgba(0,0,0,0.5); color: var(--neon-cyan); border: 1px solid var(--neon-cyan); border-radius: 4px; padding: 0.5rem; font-family: 'JetBrains Mono', monospace;">
                    <option value="">Current Scan</option>
                    ${reports.map(r => `
                        <option value="${r.report_id}">${r.report_id} (${new Date(r.timestamp).toLocaleString()})</option>
                    `).join('')}
                </select>
            </div>
        `;

        container.innerHTML = selector;
    },

    async loadReport(reportId) {
        if (!reportId) {
            // Reload current scan
            this.fetchAndRender();
            return;
        }

        try {
            const vulns = await api.get(`/api/security/reports/${reportId}`);
            this.renderStats(vulns);
            this.renderVulnerabilities(vulns);
        } catch (error) {
            console.error('Failed to load report:', error);
            this.renderSecurityError(`Failed to load report: ${reportId}`);
        }
    },

    renderVulnerabilities(vulns) {
        const container = document.getElementById('security-vulns-content');
        document.getElementById('security-vuln-count').textContent = vulns.images ? vulns.images.length : 0;

        if (!vulns.images || vulns.images.length === 0) {
            container.innerHTML = '<div class="no-issues">No vulnerability reports found</div>';
            return;
        }

        const table = `
            <table class="issues-table">
                <thead>
                    <tr>
                        <th>Image</th>
                        <th>Namespace</th>
                        <th>🔴 Critical</th>
                        <th>🟠 High</th>
                        <th>🟡 Medium</th>
                        <th>🟢 Low</th>
                        <th>Last Scan</th>
                    </tr>
                </thead>
                <tbody>
                    ${vulns.images.map(img => `
                        <tr>
                            <td><code title="${img.image}">${this.truncate(img.image, 40)}</code></td>
                            <td>${img.namespace}</td>
                            <td><span class="status-badge ${img.critical_count > 0 ? 'unhealthy' : 'healthy'}">${img.critical_count}</span></td>
                            <td><span class="status-badge ${img.high_count > 0 ? 'warning' : 'healthy'}">${img.high_count}</span></td>
                            <td><span class="status-badge ${img.medium_count > 0 ? 'info' : 'healthy'}">${img.medium_count || 0}</span></td>
                            <td><span class="status-badge healthy">${img.low_count || 0}</span></td>
                            <td>${this.formatTime(img.last_scan)}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;

        container.innerHTML = table;
    },

    truncate(text, length) {
        if (text.length <= length) return text;
        return text.substring(0, length) + '...';
    },

    formatTime(timestamp) {
        if (!timestamp) return 'N/A';
        try {
            const date = new Date(timestamp);
            return date.toLocaleString();
        } catch (e) {
            return timestamp;
        }
    }
};

// Global initializer
window.SecurityDashboard = SecurityDashboard;
