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
            const [vulns, policies, fence] = await Promise.all([
                fetch('/api/security/vulnerabilities').then(r => r.json()),
                fetch('/api/security/policies').then(r => r.json()),
                fetch('/api/security/fence').then(r => r.json())
            ]);

            this.renderStats(vulns, policies, fence);
            this.renderVulnerabilities(vulns);
            this.renderPolicies(policies);
        } catch (error) {
            console.error('Failed to fetch Security data:', error);
            const container = document.getElementById('security-vulns-content');
            if (container) {
                container.innerHTML = `<div class="error">Failed to load Security data: ${error.message}</div>`;
            }
        }
    },

    renderStats(vulns, policies, fence) {
        document.getElementById('security-critical-vulns').textContent = vulns.critical || '0';
        document.getElementById('security-high-vulns').textContent = vulns.high || '0';
        document.getElementById('security-total-policies').textContent = policies.total_policies || '0';

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
