// Security Dashboard Module
const SecurityDashboard = {
    refreshInterval: null,

    init() {
        this.fetchAndRender();
        if (this.refreshInterval) clearInterval(this.refreshInterval);
        this.refreshInterval = setInterval(() => this.fetchAndRender(), 30000);
        console.log('✅ Security Dashboard initialized');
    },

    async fetchAndRender() {
        try {
            // Fetch all data in parallel
            const [vulnsRes, policiesRes, fenceRes, violationsRes] = await Promise.all([
                fetch('/api/security/vulnerabilities'),
                fetch('/api/security/policies'),
                fetch('/api/security/fence'),
                fetch('/api/security/policies/violations')
            ]);

            // Check if any request failed
            if (!vulnsRes.ok || !policiesRes.ok || !fenceRes.ok || !violationsRes.ok) {
                throw new Error('Security service unavailable');
            }

            const [vulns, policies, fence, violations] = await Promise.all([
                vulnsRes.json(),
                policiesRes.json(),
                fenceRes.json(),
                violationsRes.json()
            ]);

            this.renderStats(vulns, policies, fence, violations);
            this.renderVulnerabilities(vulns);
            this.renderPolicies(policies);
            this.renderViolations(violations);
        } catch (error) {
            console.error('Failed to fetch Security data:', error);
            this.renderSecurityError('Security data unavailable. The security service may not be configured.');
        }
    },

    renderSecurityError(message) {
        // Update stats to show N/A
        document.getElementById('security-critical-vulns').textContent = '-';
        document.getElementById('security-high-vulns').textContent = '-';
        document.getElementById('security-total-policies').textContent = '-';
        document.getElementById('security-violations-count').textContent = '-';
        document.getElementById('fence-status-text').textContent = 'N/A';

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

        // Render error in policies container
        const policiesContainer = document.getElementById('security-policies-content');
        if (policiesContainer) {
            policiesContainer.innerHTML = '<div class="no-issues">Security policies unavailable</div>';
        }

        // Render error in violations container
        const violationsContainer = document.getElementById('security-violations-content');
        if (violationsContainer) {
            violationsContainer.innerHTML = '<div class="no-issues">Policy violations unavailable</div>';
        }
    },

    renderStats(vulns, policies, fence, violations) {
        document.getElementById('security-critical-vulns').textContent = vulns.critical || '0';
        document.getElementById('security-high-vulns').textContent = vulns.high || '0';
        document.getElementById('security-total-policies').textContent = policies.total_policies || '0';
        document.getElementById('security-violations-count').textContent = (violations && violations.total_violations) || '0';

        const fenceStatusText = document.getElementById('fence-status-text');
        const fenceStatusBox = document.getElementById('fence-status-box');

        if (fenceStatusText && fence) {
            fenceStatusText.textContent = fence.status.toUpperCase();
            fenceStatusBox.className = `stat-box ${fence.status === 'healthy' ? 'healthy' : 'unhealthy'}`;
            fenceStatusBox.title = `Pods: ${fence.pods} in namespace ${fence.namespace}`;
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
                            <td>${this.formatTime(img.last_scan)}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;

        container.innerHTML = table;
    },

    renderPolicies(policies) {
        const container = document.getElementById('security-policies-content');
        document.getElementById('security-policies-count').textContent = policies.policies ? policies.policies.length : 0;

        if (!policies.policies || policies.policies.length === 0) {
            container.innerHTML = '<div class="no-issues">No network policies found</div>';
            return;
        }

        const table = `
            <table class="issues-table">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Namespace</th>
                        <th>Matched Pods</th>
                        <th>Ingress Rules</th>
                        <th>Egress Rules</th>
                    </tr>
                </thead>
                <tbody>
                    ${policies.policies.map(policy => `
                        <tr>
                            <td><strong>${policy.name}</strong></td>
                            <td>${policy.namespace}</td>
                            <td>${policy.endpoints_matched}</td>
                            <td>${policy.ingress_rules}</td>
                            <td>${policy.egress_rules}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;

        container.innerHTML = table;
    },

    renderViolations(violations) {
        const container = document.getElementById('security-violations-content');

        if (!violations.violations || violations.violations.length === 0) {
            container.innerHTML = '<div class="no-issues">No policy violations found ✓</div>';
            return;
        }

        const table = `
            <table class="issues-table">
                <thead>
                    <tr>
                        <th>Policy</th>
                        <th>Resource</th>
                        <th>Namespace</th>
                        <th>Severity</th>
                        <th>Message</th>
                    </tr>
                </thead>
                <tbody>
                    ${violations.violations.map(v => `
                        <tr>
                            <td><strong title="${v.rule}">${v.policy}</strong></td>
                            <td><code>${v.resource}</code></td>
                            <td>${v.namespace}</td>
                            <td><span class="status-badge ${v.severity === 'high' ? 'unhealthy' : 'warning'}">${v.severity}</span></td>
                            <td class="error-message" title="${v.message}">${this.truncate(v.message, 60)}</td>
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
