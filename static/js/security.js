/**
 * Security Dashboard Module - Enhanced Trivy Integration
 * Features: Charts, filtering, CSV export, detailed views
 */
const SecurityDashboard = {
    data: null,
    filteredData: null,
    chart: null,
    namespaces: new Set(),

    init() {
        console.log('✅ Security Dashboard initialized');
        this.initChart();
    },

    initChart() {
        const canvas = document.getElementById('vuln-chart');
        if (!canvas) return;

        // Simple bar chart using canvas
        this.chart = {
            canvas: canvas,
            draw: (data) => {
                const ctx = canvas.getContext('2d');
                const width = canvas.width = 280;
                const height = canvas.height = 180;

                // Clear
                ctx.fillStyle = '#0a0a0f';
                ctx.fillRect(0, 0, width, height);

                if (!data || data.total === 0) {
                    ctx.fillStyle = '#666';
                    ctx.font = '14px JetBrains Mono';
                    ctx.textAlign = 'center';
                    ctx.fillText('No data available', width / 2, height / 2);
                    return;
                }

                const max = Math.max(data.critical, data.high, data.medium, data.low, 1);
                const barWidth = 50;
                const gap = 20;
                const startX = 20;
                const startY = height - 40;

                const bars = [
                    { label: 'Crit', value: data.critical, color: '#ff4444' },
                    { label: 'High', value: data.high, color: '#ff8800' },
                    { label: 'Med', value: data.medium, color: '#ffdd00' },
                    { label: 'Low', value: data.low, color: '#44ff44' }
                ];

                bars.forEach((bar, i) => {
                    const x = startX + i * (barWidth + gap);
                    const barHeight = (bar.value / max) * 100;

                    // Draw bar
                    ctx.fillStyle = bar.color;
                    ctx.fillRect(x, startY - barHeight, barWidth, barHeight);

                    // Draw value on top
                    ctx.fillStyle = '#fff';
                    ctx.font = '12px JetBrains Mono';
                    ctx.textAlign = 'center';
                    ctx.fillText(bar.value.toString(), x + barWidth / 2, startY - barHeight - 5);

                    // Draw label
                    ctx.fillStyle = '#888';
                    ctx.fillText(bar.label, x + barWidth / 2, startY + 15);
                });
            }
        };
    },

    loadSecurityData() {
        return this.fetchAndRender();
    },

    async fetchAndRender() {
        try {
            // Show loading
            this.renderLoading();

            // Fetch vulnerabilities
            const vulns = await api.get('/api/security/vulnerabilities');
            this.data = vulns;
            this.filteredData = { ...vulns };

            // Fetch available reports
            try {
                const reportsData = await api.get('/api/security/reports');
                this.renderReportSelector(reportsData || []);
            } catch (e) {
                console.warn('Failed to fetch reports list:', e);
            }

            // Extract namespaces for filter
            this.extractNamespaces(vulns.images || []);

            // Update UI
            this.renderStats(vulns);
            this.renderChart(vulns);
            this.renderVulnerabilities(vulns);
            this.renderByNamespace(vulns);
            this.renderTopRisk(vulns);
            this.updateLastUpdated();

        } catch (error) {
            console.error('Failed to fetch security data:', error);
            this.renderSecurityError('Security service unavailable. Please check configuration.');
        }
    },

    refreshAll() {
        this.fetchAndRender();
    },

    renderLoading() {
        const containers = [
            'security-vulns-content',
            'security-by-namespace',
            'security-top-risk'
        ];
        containers.forEach(id => {
            const el = document.getElementById(id);
            if (el) el.innerHTML = '<div class="loading">Loading...</div>';
        });
    },

    updateLastUpdated() {
        const el = document.getElementById('last-updated');
        if (el) {
            el.textContent = `Last updated: ${new Date().toLocaleTimeString()}`;
        }
    },

    extractNamespaces(images) {
        this.namespaces.clear();
        images.forEach(img => {
            if (img.namespace) this.namespaces.add(img.namespace);
        });

        // Populate namespace filter
        const select = document.getElementById('filter-namespace');
        if (select) {
            const currentValue = select.value;
            select.innerHTML = '<option value="all">All Namespaces</option>';
            Array.from(this.namespaces).sort().forEach(ns => {
                select.innerHTML += `<option value="${ns}">${ns}</option>`;
            });
            select.value = currentValue || 'all';
        }
    },

    applyFilters() {
        if (!this.data) return;

        const severityFilter = document.getElementById('filter-severity')?.value || 'all';
        const namespaceFilter = document.getElementById('filter-namespace')?.value || 'all';
        const searchFilter = document.getElementById('filter-search')?.value?.toLowerCase() || '';

        let filtered = [...(this.data.images || [])];

        // Filter by namespace
        if (namespaceFilter !== 'all') {
            filtered = filtered.filter(img => img.namespace === namespaceFilter);
        }

        // Filter by search
        if (searchFilter) {
            filtered = filtered.filter(img =>
                img.image?.toLowerCase().includes(searchFilter) ||
                img.namespace?.toLowerCase().includes(searchFilter)
            );
        }

        // Filter by severity (show only images with at least that severity)
        if (severityFilter !== 'all') {
            filtered = filtered.filter(img => {
                switch (severityFilter) {
                    case 'critical': return img.critical_count > 0;
                    case 'high': return img.critical_count > 0 || img.high_count > 0;
                    case 'medium': return img.critical_count > 0 || img.high_count > 0 || img.medium_count > 0;
                    default: return true;
                }
            });
        }

        this.filteredData = {
            ...this.data,
            images: filtered
        };

        this.renderVulnerabilities(this.filteredData);

        // Update filtered count
        const countEl = document.getElementById('filtered-count');
        if (countEl && this.data.images) {
            countEl.textContent = `Showing ${filtered.length} of ${this.data.images.length}`;
        }
    },

    renderStats(vulns) {
        const stats = {
            critical: vulns.critical || 0,
            high: vulns.high || 0,
            medium: vulns.medium || 0,
            low: vulns.low || 0,
            total: vulns.total || 0
        };

        Object.entries(stats).forEach(([key, value]) => {
            const el = document.getElementById(`security-${key}-vulns`);
            if (el) {
                el.textContent = value;
                // Add animation for changes
                el.style.transform = 'scale(1.2)';
                setTimeout(() => el.style.transform = 'scale(1)', 200);
            }
        });

        // Update document title with critical count
        if (stats.critical > 0) {
            document.title = `(${stats.critical} Critical) Security - Kusanagi`;
        } else {
            document.title = 'Security - Kusanagi';
        }
    },

    renderChart(vulns) {
        if (this.chart) {
            this.chart.draw(vulns);
        }
    },

    renderSecurityError(message) {
        // Reset stats
        ['critical', 'high', 'medium', 'low', 'total'].forEach(key => {
            const el = document.getElementById(`security-${key}-vulns`);
            if (el) el.textContent = '-';
        });

        // Render error
        const container = document.getElementById('security-vulns-content');
        if (container) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 3rem; text-align: center;">
                    <span style="font-size: 3rem;">🛡️</span>
                    <h3 style="margin: 1rem 0; color: var(--neon-cyan);">Security Center</h3>
                    <p style="color: var(--neon-orange); margin-bottom: 1rem;">⚠️ ${message}</p>
                    <button onclick="SecurityDashboard.refreshAll()" class="cyber-btn" style="margin-top: 1rem;">
                        🔄 Retry
                    </button>
                </div>
            `;
        }

        if (this.chart) this.chart.draw({ total: 0 });
    },

    renderReportSelector(reports) {
        const container = document.getElementById('security-report-selector');
        if (!container) return;

        if (!reports || reports.length === 0) {
            container.innerHTML = '';
            return;
        }

        container.innerHTML = `
            <div style="background: rgba(0,0,0,0.3); border: 1px solid var(--neon-cyan); border-radius: 8px; padding: 0.75rem; display: flex; align-items: center; gap: 1rem;">
                <span style="color: var(--neon-cyan); font-size: 0.9rem;">📊 Report History:</span>
                <select id="report-select" class="cyber-input small" onchange="SecurityDashboard.loadReport(this.value)"
                    style="flex: 1; background: rgba(0,0,0,0.5); color: var(--neon-cyan); border: 1px solid var(--neon-cyan);">
                    <option value="">Current Scan</option>
                    ${reports.map(r => `
                        <option value="${r.report_id || r.category + '/' + r.name}">
                            ${r.report_id || r.name} (${new Date(r.timestamp || r.date).toLocaleString()})
                        </option>
                    `).join('')}
                </select>
            </div>
        `;
    },

    async loadReport(reportId) {
        if (!reportId) {
            this.fetchAndRender();
            return;
        }

        try {
            this.renderLoading();
            const vulns = await api.get(`/api/security/reports/${reportId}`);
            this.data = vulns;
            this.filteredData = { ...vulns };

            this.renderStats(vulns);
            this.renderChart(vulns);
            this.renderVulnerabilities(vulns);
            this.renderByNamespace(vulns);
            this.renderTopRisk(vulns);
            this.updateLastUpdated();

        } catch (error) {
            console.error('Failed to load report:', error);
            this.renderSecurityError(`Failed to load report: ${reportId}`);
        }
    },

    renderVulnerabilities(vulns) {
        const container = document.getElementById('security-vulns-content');
        const countEl = document.getElementById('security-vuln-count');

        if (!container) return;

        const images = vulns.images || [];
        if (countEl) countEl.textContent = images.length;

        if (images.length === 0) {
            container.innerHTML = `
                <div class="no-issues" style="padding: 3rem; text-align: center;">
                    <span style="font-size: 2rem;">✅</span>
                    <p>No vulnerabilities found matching your filters</p>
                </div>
            `;
            return;
        }

        // Sort by risk score (critical * 10 + high * 5 + medium * 2 + low)
        const sortedImages = [...images].sort((a, b) => {
            const scoreA = a.critical_count * 10 + a.high_count * 5 + a.medium_count * 2 + a.low_count;
            const scoreB = b.critical_count * 10 + b.high_count * 5 + b.medium_count * 2 + b.low_count;
            return scoreB - scoreA;
        });

        const table = `
            <table class="issues-table" style="font-size: 0.9rem;">
                <thead>
                    <tr>
                        <th style="text-align: left;">Image</th>
                        <th>Namespace</th>
                        <th style="text-align: center;">🔴</th>
                        <th style="text-align: center;">🟠</th>
                        <th style="text-align: center;">🟡</th>
                        <th style="text-align: center;">🟢</th>
                        <th>Score</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    ${sortedImages.map(img => {
                        const riskScore = img.critical_count * 10 + img.high_count * 5 + img.medium_count * 2 + img.low_count;
                        let riskClass = 'healthy';
                        if (img.critical_count > 0) riskClass = 'unhealthy';
                        else if (img.high_count > 0) riskClass = 'warning';
                        else if (img.medium_count > 0) riskClass = 'info';

                        return `
                            <tr class="vuln-row" data-image="${img.image}" style="cursor: pointer;" onclick="SecurityDashboard.toggleDetails('${this.escapeId(img.image)}')">
                                <td style="max-width: 300px; overflow: hidden; text-overflow: ellipsis;">
                                    <code title="${img.image}" style="font-size: 0.8rem;">${this.truncate(img.image, 45)}</code>
                                    <div style="font-size: 0.7rem; color: #888; margin-top: 2px;">${this.formatTime(img.last_scan)}</div>
                                </td>
                                <td><span class="status-badge info">${img.namespace}</span></td>
                                <td style="text-align: center;">${img.critical_count > 0 ? `<span class="status-badge unhealthy">${img.critical_count}</span>` : '-'}</td>
                                <td style="text-align: center;">${img.high_count > 0 ? `<span class="status-badge warning">${img.high_count}</span>` : '-'}</td>
                                <td style="text-align: center;">${img.medium_count > 0 ? `<span class="status-badge info">${img.medium_count}</span>` : '-'}</td>
                                <td style="text-align: center;">${img.low_count > 0 ? `<span class="status-badge healthy">${img.low_count}</span>` : '-'}</td>
                                <td><span class="status-badge ${riskClass}">${riskScore}</span></td>
                                <td>
                                    <button class="cyber-btn small" onclick="event.stopPropagation(); SecurityDashboard.scanImage('${img.image}')" title="Rescan">
                                        🔄
                                    </button>
                                </td>
                            </tr>
                            <tr id="details-${this.escapeId(img.image)}" style="display: none; background: rgba(0,0,0,0.3);">
                                <td colspan="8" style="padding: 1rem;">
                                    <div style="font-size: 0.85rem;">
                                        <strong>Full Image:</strong> <code>${img.image}</code><br>
                                        <strong>Digest:</strong> <code style="color: #888;">${img.digest || 'N/A'}</code><br>
                                        ${img.os ? `<strong>OS:</strong> ${img.os}<br>` : ''}
                                        ${img.scanner_version ? `<strong>Scanner:</strong> ${img.scanner_version}<br>` : ''}
                                    </div>
                                </td>
                            </tr>
                        `;
                    }).join('')}
                </tbody>
            </table>
        `;

        container.innerHTML = table;
    },

    renderByNamespace(vulns) {
        const container = document.getElementById('security-by-namespace');
        if (!container) return;

        const images = vulns.images || [];
        if (images.length === 0) {
            container.innerHTML = '<div class="no-issues">No data</div>';
            return;
        }

        // Group by namespace
        const byNs = {};
        images.forEach(img => {
            const ns = img.namespace || 'unknown';
            if (!byNs[ns]) {
                byNs[ns] = { count: 0, critical: 0, high: 0, medium: 0, low: 0 };
            }
            byNs[ns].count++;
            byNs[ns].critical += img.critical_count || 0;
            byNs[ns].high += img.high_count || 0;
            byNs[ns].medium += img.medium_count || 0;
            byNs[ns].low += img.low_count || 0;
        });

        const sorted = Object.entries(byNs).sort((a, b) => (b[1].critical + b[1].high) - (a[1].critical + a[1].high));

        container.innerHTML = `
            <table class="issues-table" style="font-size: 0.85rem;">
                <thead>
                    <tr>
                        <th>Namespace</th>
                        <th style="text-align: center;">Images</th>
                        <th style="text-align: center;">🔴</th>
                        <th style="text-align: center;">🟠</th>
                    </tr>
                </thead>
                <tbody>
                    ${sorted.map(([ns, data]) => `
                        <tr>
                            <td><span class="status-badge info">${ns}</span></td>
                            <td style="text-align: center;">${data.count}</td>
                            <td style="text-align: center;">${data.critical > 0 ? `<span class="status-badge unhealthy">${data.critical}</span>` : '-'}</td>
                            <td style="text-align: center;">${data.high > 0 ? `<span class="status-badge warning">${data.high}</span>` : '-'}</td>
                        </tr>
                    `).join('')}
                </tbody>
            </table>
        `;
    },

    renderTopRisk(vulns) {
        const container = document.getElementById('security-top-risk');
        if (!container) return;

        const images = vulns.images || [];
        if (images.length === 0) {
            container.innerHTML = '<div class="no-issues">No data</div>';
            return;
        }

        // Sort by risk and take top 5
        const topRisk = [...images]
            .sort((a, b) => (b.critical_count * 10 + b.high_count * 5) - (a.critical_count * 10 + a.high_count * 5))
            .slice(0, 5);

        container.innerHTML = `
            <div style="padding: 0.5rem;">
                ${topRisk.map((img, i) => {
                    const riskScore = img.critical_count * 10 + img.high_count * 5;
                    let riskColor = '#44ff44';
                    if (img.critical_count > 0) riskColor = '#ff4444';
                    else if (img.high_count > 0) riskColor = '#ff8800';
                    else if (img.medium_count > 0) riskColor = '#ffdd00';

                    return `
                        <div style="display: flex; align-items: center; gap: 0.75rem; padding: 0.75rem; border-bottom: 1px solid rgba(255,255,255,0.1); ${i === 0 ? 'background: rgba(255,0,0,0.1);' : ''}">
                            <span style="font-size: 1.2rem;">${i === 0 ? '🔥' : i < 3 ? '⚠️' : '⚡'}</span>
                            <div style="flex: 1; min-width: 0;">
                                <div style="font-size: 0.8rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;" title="${img.image}">
                                    ${this.truncate(img.image, 35)}
                                </div>
                                <div style="font-size: 0.7rem; color: #888;">${img.namespace}</div>
                            </div>
                            <div style="text-align: right;">
                                <div style="font-size: 1rem; font-weight: bold; color: ${riskColor};">${riskScore}</div>
                                <div style="font-size: 0.7rem; color: #888;">risk score</div>
                            </div>
                        </div>
                    `;
                }).join('')}
            </div>
        `;
    },

    toggleDetails(imageId) {
        const row = document.getElementById(`details-${imageId}`);
        if (row) {
            row.style.display = row.style.display === 'none' ? 'table-row' : 'none';
        }
    },

    escapeId(str) {
        return str.replace(/[^a-zA-Z0-9]/g, '_');
    },

    async scanImage(imageName) {
        // Could trigger a rescan via API
        console.log('Rescan requested for:', imageName);
        alert(`Rescan requested for: ${imageName}\n(Feature would trigger Trivy rescan)`);
    },

    exportCSV() {
        if (!this.filteredData || !this.filteredData.images) {
            alert('No data to export');
            return;
        }

        const images = this.filteredData.images;
        const headers = ['Image', 'Namespace', 'Critical', 'High', 'Medium', 'Low', 'Last Scan'];
        const rows = images.map(img => [
            img.image,
            img.namespace,
            img.critical_count,
            img.high_count,
            img.medium_count,
            img.low_count,
            img.last_scan
        ]);

        const csv = [headers.join(','), ...rows.map(r => r.join(','))].join('\n');
        const blob = new Blob([csv], { type: 'text/csv' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `security-report-${new Date().toISOString().split('T')[0]}.csv`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    },

    truncate(text, length) {
        if (!text) return 'N/A';
        if (text.length <= length) return text;
        return text.substring(0, length) + '...';
    },

    formatTime(timestamp) {
        if (!timestamp) return 'N/A';
        try {
            const date = new Date(timestamp);
            const now = new Date();
            const diff = now - date;
            const hours = Math.floor(diff / (1000 * 60 * 60));

            if (hours < 1) return 'Just now';
            if (hours < 24) return `${hours}h ago`;
            if (hours < 168) return `${Math.floor(hours / 24)}d ago`;
            return date.toLocaleDateString();
        } catch (e) {
            return timestamp;
        }
    }
};

// Global export
window.SecurityDashboard = SecurityDashboard;
